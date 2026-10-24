//! Device-independent owned PCM source. Never ticks or renders a game.
use crate::sound::SoundBatch;
use lemmings_audio::{AdlibPcm, AudioRate, MAX_PCM_FRAMES, PcmContinuation, PcmError};
use lemmings_resources::{ADLIB_IMAGE_SIZE, AdlibCommand, AdlibFile, AdlibImage, AdlibWrite};

pub fn decode_adlib(dat: &[u8]) -> Result<Vec<u8>, String> {
    let file = AdlibFile::parse(dat).map_err(|e| e.to_string())?;
    let mut bytes = vec![0; ADLIB_IMAGE_SIZE];
    file.decode_into(&mut bytes).map_err(|e| e.to_string())?;
    Ok(bytes)
}
/// Original-free practice tones, composed here; not a DOS tune or recording.
pub fn demo_image() -> Vec<u8> {
    fn word(b: &mut [u8], at: usize, value: usize) {
        b[at..at + 2].copy_from_slice(&(value as u16).to_le_bytes());
    }
    let mut b = vec![0; ADLIB_IMAGE_SIZE];
    for i in 0..21 {
        word(&mut b, 2926 + i * 2, 2968);
    }
    word(&mut b, 2968, 0x4300);
    word(&mut b, 2970, 3200 - 2926);
    b[2972] = 2;
    b[2973] = 1;
    word(&mut b, 2974, 3000 - 2926);
    word(&mut b, 3000, 3100 - 2926);
    word(&mut b, 3004, 3000 - 2926);
    let notes = [0xc1, 0xe3, 24, 28, 31, 28, 26, 29, 33, 29, 0x80];
    b[3100..3100 + notes.len()].copy_from_slice(&notes);
    let instrument = [
        0xf2, 0xf2, 0x05, 0x05, 0x21, 0x21, 0, 0, 0, 0x06, 0, 0, 0, 0, 1, 0,
    ];
    for (base, count) in [(3200, 31), (21731, 16)] {
        for i in 0..count {
            b[base + i * 16..base + (i + 1) * 16].copy_from_slice(&instrument);
        }
    }
    for i in 0..18 {
        let at = 22025 + i * 4;
        word(&mut b, 21989 + i * 2, at);
        b[at..at + 4].copy_from_slice(&[0xc1, 0xe2, 18 + i as u8, 0x85]);
    }
    for (i, f) in [343, 363, 385, 408, 432, 458, 485, 514, 544, 577, 611, 647]
        .into_iter()
        .enumerate()
    {
        word(&mut b, 2343 + i * 32, f);
    }
    for note in 0..96 {
        b[2727 + note] = 3 + (note / 12) as u8;
        b[2823 + note] = (note % 12) as u8;
    }
    b
}

pub struct AudioOutput {
    // Immutable private bytes and an opaque continuation avoid self-borrows,
    // leaked assets, unsafe code and a reset/replay on every output chunk.
    bytes: Vec<u8>,
    state: Option<PcmContinuation>,
    scratch: Vec<AdlibWrite>,
    rate: u32,
    volume: u8,
    mix: [u8; 2],
    active: bool,
    generation: Option<u64>,
    last_sequence: Option<u64>,
    frames: u64,
}
impl AudioOutput {
    pub fn new(bytes: Vec<u8>, rate: u32) -> Result<Self, String> {
        let image = AdlibImage::parse(&bytes).map_err(|e| e.to_string())?;
        let state = AdlibPcm::new(image, rate, AudioRate::DOS_VGA)
            .map_err(|e| e.to_string())?
            .into_continuation();
        Ok(Self {
            bytes,
            state: Some(state),
            scratch: vec![AdlibWrite::default(); 4096],
            rate,
            volume: 40,
            mix: [100; 2],
            active: false,
            generation: None,
            last_sequence: None,
            frames: 0,
        })
    }
    pub const fn frames(&self) -> u64 {
        self.frames
    }
    pub fn set_volume(&mut self, volume: u8) -> Result<(), String> {
        if volume > 100 {
            return Err("volume must be in 0..=100".into());
        }
        self.volume = volume;
        Ok(())
    }
    pub fn set_mix(&mut self, music: u8, effects: u8) -> Result<(), String> {
        if music > 100 || effects > 100 {
            return Err("music/effects volume must be in 0..=100".into());
        }
        self.mix = [music, effects];
        Ok(())
    }
    pub fn suspend(&mut self) {
        self.active = false;
    }
    fn reset(&mut self) {
        let image = AdlibImage::parse(&self.bytes).expect("private validated immutable image");
        self.state = Some(
            AdlibPcm::new(image, self.rate, AudioRate::DOS_VGA)
                .expect("validated rate")
                .into_continuation(),
        );
        self.frames = 0;
    }
    /// Return true when the platform must discard already scheduled device PCM.
    /// Activation/resume/generation/loss starts current music fresh, drops stale
    /// effects, and never admits wall-time catch-up into synthesis or gameplay.
    pub fn synchronize(&mut self, batch: &SoundBatch, active: bool) -> Result<bool, String> {
        if self.generation.is_some_and(|g| batch.generation < g) {
            return Err("stale audio generation".into());
        }
        let image = AdlibImage::parse(&self.bytes).expect("private validated immutable image");
        image
            .music(usize::from(batch.music_index))
            .map_err(|e| e.to_string())?;
        let same = self.generation == Some(batch.generation);
        let mut previous = if same { self.last_sequence } else { None };
        for event in &batch.events {
            if previous.is_some_and(|s| event.sequence <= s) {
                return Err("duplicate or unordered audio command".into());
            }
            previous = Some(event.sequence);
            match event.command {
                Some(AdlibCommand::Music(i)) => {
                    image.music(usize::from(i)).map_err(|e| e.to_string())?;
                }
                Some(AdlibCommand::Effect(i)) => {
                    image
                        .effect_offset(usize::from(i))
                        .map_err(|e| e.to_string())?;
                }
                _ => {}
            }
        }
        let reset = !same
            || batch.reset
            || batch.dropped != 0
            || batch.sequence_exhausted
            || (!self.active && active);
        let flush = reset
            || self.active != active
            || batch
                .events
                .iter()
                .any(|e| e.command == Some(AdlibCommand::Stop));
        if active {
            if reset {
                self.reset();
                if batch.music_playing {
                    self.apply(AdlibCommand::Music(batch.music_index))?;
                }
            } else {
                for event in &batch.events {
                    if let Some(command) = event.command {
                        self.apply(command)?;
                    }
                }
            }
        }
        self.generation = Some(batch.generation);
        self.last_sequence = previous;
        self.active = active;
        Ok(flush)
    }
    fn apply(&mut self, command: AdlibCommand) -> Result<(), String> {
        let image = AdlibImage::parse(&self.bytes).expect("private validated immutable image");
        let mut pcm = AdlibPcm::resume(image, self.state.take().expect("owned continuation"));
        let result = pcm.apply(command).map_err(|e| e.to_string());
        self.state = Some(pcm.into_continuation());
        result
    }
    pub fn render(&mut self, output: &mut [i16]) -> Result<(), String> {
        if output.len() > MAX_PCM_FRAMES {
            return Err(PcmError::Frames {
                requested: output.len(),
            }
            .to_string());
        }
        if !self.active {
            output.fill(0);
            return Ok(());
        }
        let image = AdlibImage::parse(&self.bytes).expect("private validated immutable image");
        let mut pcm = AdlibPcm::resume(image, self.state.take().expect("owned continuation"));
        pcm.set_mix(self.mix[0], self.mix[1]);
        let result = pcm
            .render(output, &mut self.scratch, 4096)
            .map_err(|e| e.to_string());
        self.frames = pcm.position().frames;
        self.state = Some(pcm.into_continuation());
        result?;
        for sample in output {
            *sample = (i32::from(*sample) * i32::from(self.volume) / 100) as i16;
        }
        Ok(())
    }
}
