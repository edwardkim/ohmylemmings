//! Bounded game-to-audio commands. Consumption never advances the simulation.
use lemmings_core::SoundCue;
use lemmings_resources::{AdlibCommand, DosMusic, SoundMappingError, adlib_effect_for_cue};
use lemmings_session::SessionSound;
use std::collections::VecDeque;
use std::fmt::Write;

pub const SOUND_EVENT_CAPACITY: usize = 1024;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SoundEvent {
    pub sequence: u64,
    pub source: SessionSound,
    pub command: Option<AdlibCommand>,
    pub raw_trap: Option<u8>,
    pub mapping_error: Option<SoundMappingError>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SoundBatch {
    pub generation: u64,
    /// A new generation requires a fresh driver/chip/phase and device queue.
    pub reset: bool,
    pub dropped: u64,
    pub sequence_exhausted: bool,
    pub music_index: u8,
    pub music_playing: bool,
    pub events: Vec<SoundEvent>,
}
pub struct SoundTimeline {
    generation: u64,
    reset_pending: bool,
    next_sequence: u64,
    dropped: u64,
    sequence_exhausted: bool,
    events: VecDeque<SoundEvent>,
    music: DosMusic,
    music_special: u16,
    music_index: u8,
    music_playing: bool,
    trap_sounds: [u8; 16],
}
impl SoundTimeline {
    pub fn new(music_special: u16, trap_sounds: [u8; 16]) -> Result<Self, SoundMappingError> {
        let mut music = DosMusic::default();
        let music_index = music.select(music_special, false)?;
        Ok(Self {
            generation: 0,
            reset_pending: true,
            next_sequence: 0,
            dropped: 0,
            sequence_exhausted: false,
            events: VecDeque::new(),
            music,
            music_special,
            music_index,
            music_playing: false,
            trap_sounds,
        })
    }
    pub const fn generation(&self) -> u64 {
        self.generation
    }
    pub const fn music_index(&self) -> u8 {
        self.music_index
    }
    /// Set a freshly built replacement's generation. Does not mutate the old run.
    pub fn fresh_generation_after(&mut self, previous: u64) -> Result<(), &'static str> {
        let generation = previous.checked_add(1).ok_or("sound generation overflow")?;
        self.generation = generation;
        self.reset_pending = true;
        self.next_sequence = 0;
        self.dropped = 0;
        self.sequence_exhausted = false;
        self.events.clear();
        self.music_playing = false;
        Ok(())
    }
    /// Continue the DOS music cursor; only a successful Continue advances it.
    /// Explicit Restart/manual selection hold the cursor and use the first
    /// normal song, matching the original non-advance path.
    pub fn continue_from(
        &mut self,
        previous: &Self,
        advance: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut music = previous.music;
        let index = music.select(self.music_special, advance)?;
        self.fresh_generation_after(previous.generation)?;
        self.music = music;
        self.music_index = index;
        Ok(())
    }
    pub(crate) fn report_dropped(&mut self, count: u64) {
        self.dropped = self.dropped.saturating_add(count);
    }
    pub fn collect(&mut self, frame: &[SessionSound]) {
        for source in frame {
            let raw_trap = match source.cue {
                SoundCue::Trap { slot } => self.trap_sounds.get(usize::from(slot)).copied(),
                _ => None,
            };
            let result = match source.cue {
                SoundCue::MusicStart => {
                    self.music_playing = true;
                    Ok(Some(AdlibCommand::Music(self.music_index)))
                }
                SoundCue::LevelFinished => {
                    self.music_playing = false;
                    Ok(Some(AdlibCommand::Stop))
                }
                cue => adlib_effect_for_cue(cue, raw_trap.unwrap_or(0)),
            };
            let (command, mapping_error) = match result {
                Ok(command) => (command, None),
                Err(error) => (None, Some(error)),
            };
            if self.next_sequence == u64::MAX {
                self.sequence_exhausted = true;
                self.dropped = self.dropped.saturating_add(1);
                continue;
            }
            let event = SoundEvent {
                sequence: self.next_sequence,
                source: *source,
                command,
                raw_trap,
                mapping_error,
            };
            self.next_sequence += 1;
            if self.events.len() == SOUND_EVENT_CAPACITY {
                self.events.pop_front();
                self.dropped = self.dropped.saturating_add(1);
            }
            self.events.push_back(event);
        }
    }
    pub fn drain(&mut self) -> SoundBatch {
        SoundBatch {
            generation: self.generation,
            reset: std::mem::take(&mut self.reset_pending),
            dropped: std::mem::take(&mut self.dropped),
            sequence_exhausted: self.sequence_exhausted,
            music_index: self.music_index,
            music_playing: self.music_playing,
            events: self.events.drain(..).collect(),
        }
    }
    pub fn diagnostic_json(&self) -> String {
        format!(
            "{{\"generation\":{},\"queued\":{},\"dropped\":{},\"next_sequence\":{},\"music\":{},\"playing\":{},\"reset\":{},\"sequence_exhausted\":{}}}",
            self.generation,
            self.events.len(),
            self.dropped,
            self.next_sequence,
            self.music_index,
            self.music_playing,
            self.reset_pending,
            self.sequence_exhausted
        )
    }
    pub fn drain_json(&mut self) -> String {
        self.drain().to_json()
    }
}
impl SoundBatch {
    pub fn to_json(&self) -> String {
        let mut out = format!(
            "{{\"generation\":{},\"reset\":{},\"dropped\":{},\"sequence_exhausted\":{},\"music\":{},\"playing\":{},\"events\":[",
            self.generation,
            self.reset,
            self.dropped,
            self.sequence_exhausted,
            self.music_index,
            self.music_playing
        );
        for (i, e) in self.events.iter().enumerate() {
            if i != 0 {
                out.push(',');
            }
            let optional = |v: Option<u64>| v.map_or_else(|| "null".into(), |v| v.to_string());
            let command = match e.command {
                Some(AdlibCommand::Music(index)) => format!("[\"music\",{index}]"),
                Some(AdlibCommand::Effect(index)) => format!("[\"effect\",{index}]"),
                Some(AdlibCommand::Stop) => "[\"stop\"]".into(),
                None => "null".into(),
            };
            let s = e.source;
            let object_slot = match s.cue {
                SoundCue::Trap { slot } => Some(u64::from(slot)),
                _ => None,
            };
            let cue = format!("{:?}", s.cue); // Static ASCII enum names and integer fields only.
            let error = e.mapping_error.map_or_else(
                || "null".into(),
                |v| format!("{v:?}").escape_debug().to_string(),
            );
            let error = if e.mapping_error.is_some() {
                format!("\"{error}\"")
            } else {
                error
            };
            write!(out,"{{\"sequence\":{},\"boundary_sequence\":{},\"control_step\":{},\"iteration\":{},\"input_sequence\":{},\"actor\":{},\"cue\":{cue:?},\"command\":{command},\"object_slot\":{},\"raw_trap\":{},\"mapping_error\":{error}}}",
                e.sequence,s.sequence,s.control_step,s.iteration,
                optional(s.input_sequence.map(u64::from)),optional(s.actor_id.map(u64::from)),
                optional(object_slot),optional(e.raw_trap.map(u64::from))).expect("String formatting");
        }
        out.push_str("]}");
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn event(cue: SoundCue) -> SessionSound {
        SessionSound {
            control_step: 0,
            iteration: 1,
            sequence: 0,
            input_sequence: None,
            actor_id: Some(7),
            cue,
        }
    }
    #[test]
    fn invalid_raw_trap_and_slot_remain_explicit_and_do_not_poison_later_commands() {
        let mut raw = [0; 16];
        raw[0] = 255;
        let mut sound = SoundTimeline::new(0, raw).unwrap();
        sound.collect(&[
            event(SoundCue::Trap { slot: 0 }),
            event(SoundCue::Trap { slot: 16 }),
            event(SoundCue::Trap { slot: 1 }),
            event(SoundCue::SkillButtonSelect),
        ]);
        let batch = sound.drain();
        assert_eq!(
            batch.events[0].mapping_error,
            Some(SoundMappingError::TrapSound(255))
        );
        assert_eq!(
            batch.events[1].mapping_error,
            Some(SoundMappingError::TrapSlot(16))
        );
        assert_eq!(batch.events[2].command, None);
        assert_eq!(batch.events[2].mapping_error, None);
        assert_eq!(batch.events[3].command, Some(AdlibCommand::Effect(0)));
        assert_eq!(batch.dropped, 0);
    }
    #[test]
    fn generation_overflow_preserves_the_replacement_and_old_timeline() {
        let mut old = SoundTimeline::new(0, [0; 16]).unwrap();
        old.generation = u64::MAX;
        let mut replacement = SoundTimeline::new(0, [0; 16]).unwrap();
        replacement.collect(&[event(SoundCue::SkillButtonSelect)]);
        let before = replacement.diagnostic_json();
        let previous = old.diagnostic_json();
        assert!(replacement.continue_from(&old, true).is_err());
        assert_eq!(replacement.diagnostic_json(), before);
        assert_eq!(old.diagnostic_json(), previous);
    }
    #[test]
    fn sequence_exhaustion_reports_loss_and_never_reuses_an_event_id() {
        let mut sound = SoundTimeline::new(0, [0; 16]).unwrap();
        sound.next_sequence = u64::MAX - 1;
        sound.collect(&[event(SoundCue::SkillButtonSelect); 2]);
        let batch = sound.drain();
        assert_eq!(batch.events.len(), 1);
        assert_eq!(batch.events[0].sequence, u64::MAX - 1);
        assert_eq!(batch.dropped, 1);
        assert!(batch.sequence_exhausted);
        sound.fresh_generation_after(0).unwrap();
        sound.collect(&[event(SoundCue::SkillButtonSelect)]);
        let batch = sound.drain();
        assert_eq!(batch.events[0].sequence, 0);
        assert!(!batch.sequence_exhausted);
    }
}
