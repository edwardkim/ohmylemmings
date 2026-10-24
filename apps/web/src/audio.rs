//! Device-free WASM PCM bridge. JS owns AudioContext and scheduled nodes.
use super::{Source, WebGame};
use lemmings_game::audio::{AudioOutput, decode_adlib, demo_image};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct WebAudio {
    output: AudioOutput,
    generation: Option<u64>,
    rate: u32,
    volume: u8,
    mix: [u8; 2],
}
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl WebAudio {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    pub fn new(rate: f64) -> Result<Self, String> {
        if !rate.is_finite() || rate.fract() != 0.0 || !(8000.0..=192000.0).contains(&rate) {
            return Err("audio sample rate must be integer 8000..=192000".into());
        }
        Ok(Self {
            output: AudioOutput::new(demo_image(), rate as u32)?,
            generation: None,
            rate: rate as u32,
            volume: 40,
            mix: [100; 2],
        })
    }
    /// Consume the single sound queue only after a replacement audio bank validates.
    /// The returned flag requires stopping all previously scheduled device nodes.
    pub fn prepare(&mut self, game: &mut WebGame, active: bool) -> Result<bool, String> {
        let generation = game.run.sound.generation();
        if self.generation != Some(generation) {
            let image = match &game.source {
                Source::Synthetic(_) => demo_image(),
                Source::Original(files) => decode_adlib(
                    files
                        .get("ADLIB.DAT")
                        .ok_or("ADLIB.DAT: select this file to enable original sound")?,
                )
                .map_err(|e| format!("ADLIB.DAT: {e}"))?,
            };
            let mut next = AudioOutput::new(image, self.rate)?;
            next.set_volume(self.volume)?;
            next.set_mix(self.mix[0], self.mix[1])?;
            let batch = game.run.sound.drain();
            next.synchronize(&batch, active)?;
            self.output = next;
            self.generation = Some(generation);
            return Ok(true);
        }
        self.output.synchronize(&game.run.sound.drain(), active)
    }
    pub fn set_volume(&mut self, volume: f64) -> Result<(), String> {
        if !volume.is_finite() || volume.fract() != 0.0 || !(0.0..=100.0).contains(&volume) {
            return Err("volume must be integer 0..=100".into());
        }
        self.output.set_volume(volume as u8)?;
        self.volume = volume as u8;
        Ok(())
    }
    pub fn set_mix(&mut self, music: f64, effects: f64) -> Result<(), String> {
        for value in [music, effects] {
            if !value.is_finite() || value.fract() != 0.0 || !(0.0..=100.0).contains(&value) {
                return Err("music/effects volume must be integer 0..=100".into());
            }
        }
        self.output.set_mix(music as u8, effects as u8)?;
        self.mix = [music as u8, effects as u8];
        Ok(())
    }
    pub fn suspend(&mut self) {
        self.output.suspend();
    }
    pub fn render(&mut self, frames: f64) -> Result<Vec<i16>, String> {
        if !frames.is_finite() || frames.fract() != 0.0 || !(0.0..=8192.0).contains(&frames) {
            return Err("PCM frames must be integer 0..=8192".into());
        }
        let mut output = vec![0; frames as usize];
        self.output.render(&mut output)?;
        Ok(output)
    }
    pub fn state(&self) -> String {
        format!(
            "{{\"generation\":{},\"sample_rate\":{},\"volume\":{},\"frames\":{}}}",
            self.generation
                .map_or_else(|| "null".into(), |n| n.to_string()),
            self.rate,
            self.volume,
            self.output.frames()
        )
    }
}
