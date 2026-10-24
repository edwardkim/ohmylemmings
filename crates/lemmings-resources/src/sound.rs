//! Verified original VGALEMMI INT61 call-site routing, not Lemmix WAV ordinals.
//! Code-relative sites and source-execution proof: issue-78_game_sound.md.
use core::fmt;
use lemmings_core::SoundCue;

/// Zero-based resource indices. Applying an invalid externally constructed index
/// is checked by AdlibPlayer/AdlibPcm before changing state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdlibCommand {
    Music(u8),
    Effect(u8),
    Stop,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SoundMappingError {
    TrapSound(u8),
    TrapSlot(u8),
    SpecialMusic(u16),
}
impl fmt::Display for SoundMappingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unsupported DOS sound mapping: {self:?}")
    }
}
impl core::error::Error for SoundMappingError {}

/// Map logical effects to the actual original one-based AL IDs, then convert
/// once to resource indices. Trap IDs come from the exact GROUND object slot.
/// Zero is silent; malformed nonzero IDs remain explicit errors.
pub fn adlib_effect_for_cue(
    cue: SoundCue,
    trap_raw: u8,
) -> Result<Option<AdlibCommand>, SoundMappingError> {
    let raw = match cue {
        SoundCue::SkillButtonSelect => 1,     // 3f77
        SoundCue::EntranceOpening => 2,       // 03a7
        SoundCue::LetsGo => 3,                // 038e
        SoundCue::AssignSkill(_) => 4,        // shared 3206 for all successful skills
        SoundCue::AssignmentBlocked(_) => 18, // front steel 3295; no inventory consumed
        SoundCue::OhNo => 5,                  // 2e59
        SoundCue::BuilderWarning => 18,       // 26d0
        SoundCue::HitsSteel => 10,            // 24cb/2a65/2d08
        SoundCue::Explosion => 12,            // 2d65/2ead
        SoundCue::Vaporizing => 13,           // 30a0
        SoundCue::Yippee => 16,               // 2f9c
        SoundCue::Drowning => 17,             // 3049
        SoundCue::Splat => 8,                 // 229b
        SoundCue::Trap { slot } => {
            if slot >= 16 {
                return Err(SoundMappingError::TrapSlot(slot));
            }
            trap_raw
        } // 2fe9..2ff5
        SoundCue::MusicStart | SoundCue::LevelFinished | SoundCue::DigStroke => return Ok(None),
    };
    match raw {
        0 => Ok(None),
        1..=18 => Ok(Some(AdlibCommand::Effect(raw - 1))),
        _ => Err(SoundMappingError::TrapSound(raw)),
    }
}

/// Original normal-level music cursor, independent of displayed level number.
/// The EXE keeps the cursor on failure/retry but selects the FIRST normal song
/// on that path (AL still contains special=0). Success advances/selects the
/// cursor. Special selectors do not consume a normal-cycle slot.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DosMusic {
    cursor: u8,
}
impl DosMusic {
    pub const fn cursor(self) -> u8 {
        self.cursor
    }
    pub fn select(
        &mut self,
        special: u16,
        advance_after_success: bool,
    ) -> Result<u8, SoundMappingError> {
        const NORMAL: [u8; 17] = [
            6, 7, 8, 10, 11, 4, 12, 13, 14, 15, 5, 16, 17, 18, 19, 20, 21,
        ];
        const SPECIAL: [u8; 4] = [1, 9, 3, 2];
        if special > 4 {
            return Err(SoundMappingError::SpecialMusic(special));
        }
        if special != 0 {
            return Ok(SPECIAL[usize::from(special - 1)] - 1);
        }
        if advance_after_success {
            self.cursor = (self.cursor + 1) % 17;
        }
        let selected = if advance_after_success {
            self.cursor
        } else {
            0
        };
        Ok(NORMAL[usize::from(selected)] - 1)
    }
}
