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
        self.chip.write_reg(u16::from(register), value);
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
}
