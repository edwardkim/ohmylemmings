//! No-allocation OPL2 interface to the fixed Nuked-OPL3 integer state machine.
//! Derived library: LGPL-2.1-or-later. Full source/license and relink instructions
//! accompany this crate; see README.md and issue-76_opl_pcm.md for provenance.
#![no_std]
// Keep upstream implementation style separate from the small project adapter.
// unsafe_code remains forbidden by the inherited workspace lint.
#[allow(clippy::all, dead_code)]
mod core;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SampleRateError(pub u32);
impl ::core::fmt::Display for SampleRateError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        write!(f, "unsupported OPL sample rate {}", self.0)
    }
}
impl ::core::error::Error for SampleRateError {}
#[derive(Clone)]
pub struct Opl2 {
    chip: core::Chip,
}
impl Opl2 {
    /// OPL3 power-on state is OPL2. Only the low register bank is exposed; extension
    /// modes and floating-point stereo panning are unavailable.
    pub fn new(sample_rate: u32) -> Result<Self, SampleRateError> {
        if !(8000..=192000).contains(&sample_rate) {
            return Err(SampleRateError(sample_rate));
        }
        Ok(Self {
            chip: core::Chip::new(sample_rate),
        })
    }
    pub fn write_reg(&mut self, register: u8, value: u8) {
        // Preserve upstream's two-native-sample register spacing so a key-off
        // followed by key-on is observed by the envelope generator. The fixed
        // 1024-entry ring/overflow policy and clock are owned by the integer core.
        self.chip.write_reg_buffered(u16::from(register), value);
    }
    /// Output-only OPL2 channel gains. Synthesis state and buffered writes keep
    /// advancing at zero gain. Unity preserves the original integer PCM exactly.
    pub fn set_channel_gains(&mut self, gains: [u8; 9]) {
        for (target, gain) in self.chip.channel_gain[..9].iter_mut().zip(gains) {
            *target = gain.min(100);
        }
    }
    /// Signed stereo pipeline values widened from the core's saturated i16 output.
    /// The native OPL pipeline can introduce a small left/right sample delay even
    /// in OPL2 mode; consumers explicitly choose the left signal for mono PCM.
    pub fn render_frame(&mut self) -> (i32, i32) {
        let mut frame = [0; 2];
        self.chip.generate_resampled(&mut frame);
        (i32::from(frame[0]), i32::from(frame[1]))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn simultaneous_key_off_on_restarts_each_note_at_every_supported_rate() {
        for rate in [8000, 22050, 44100, 48000, 49716, 96000, 192000] {
            let mut chip = Opl2::new(rate).unwrap();
            for (register, value) in [
                (0x20, 1),
                (0x23, 1),
                (0x40, 0x3f),
                (0x43, 0),
                (0x60, 0xf4),
                (0x63, 0xf4),
                (0x80, 0x44),
                (0x83, 0x44),
                (0xc0, 0),
                (0xa0, 0x98),
            ] {
                chip.write_reg(register, value);
            }
            let mut first = 0.0;
            for note in 0..10 {
                chip.write_reg(0xb0, 0x11);
                chip.write_reg(0xb0, 0x31);
                let (mut sum, mut squares) = (0_i64, 0_i64);
                let count = rate / 10;
                for frame in 0..rate / 2 {
                    let sample = i64::from(chip.render_frame().0);
                    if frame < count {
                        sum += sample;
                        squares += sample * sample;
                    }
                }
                let variance =
                    squares as f64 / f64::from(count) - (sum as f64 / f64::from(count)).powi(2);
                if note == 0 {
                    first = variance;
                }
                assert!(
                    variance > 1_000_000.0 && variance > first * 0.25,
                    "note {note} at {rate}Hz lost attack: variance {variance}, first {first}"
                );
            }
        }
    }
    #[test]
    fn sample_rate_bounds_prevent_zero_resampler_divisors() {
        for rate in [0, 1, 7999, 192001, u32::MAX] {
            assert!(Opl2::new(rate).is_err());
        }
        for rate in [8000, 44100, 48000, 49716, 96000, 192000] {
            let mut chip = Opl2::new(rate).unwrap();
            for _ in 0..64 {
                assert_eq!(chip.render_frame(), (0, 0));
            }
        }
    }
    #[test]
    fn arbitrary_low_bank_writes_and_clone_preserve_deterministic_samples() {
        let mut chip = Opl2::new(44100).unwrap();
        let mut seed = 1_u32;
        for _ in 0..10000 {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            chip.write_reg(seed as u8, (seed >> 8) as u8);
            chip.render_frame();
        }
        let mut clone = chip.clone();
        for _ in 0..1000 {
            assert_eq!(chip.render_frame(), clone.render_frame());
        }
    }
    #[test]
    fn pending_and_overflowed_register_bursts_survive_cloning() {
        for count in [1, 1023, 1024, 1025, 4096] {
            let mut chip = Opl2::new(192000).unwrap();
            for n in 0..count {
                chip.write_reg((n % 256) as u8, (n / 7) as u8);
            }
            let mut copy = chip.clone();
            for _ in 0..12000 {
                assert_eq!(chip.render_frame(), copy.render_frame());
            }
        }
    }
}
