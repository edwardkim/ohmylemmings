//! Deterministic DOS-original AdLib calls to mono signed 16-bit PCM.
//!
//! The fixed Nuked-OPL3 OPL2 engine has no allocation, floating point or device I/O.
//! Timing provenance and limits: mydocs/tech_insights/issue-76_opl_pcm.md.
#![no_std]
use core::fmt;
use lemmings_opl::Opl2;
use lemmings_resources::{AdlibImage, AdlibPlayer, AdlibState, AdlibStreamError, AdlibWrite};

/// Bound per render request; large streams can be delivered in arbitrary chunks.
pub const MAX_PCM_FRAMES: usize = 8192;
pub const MIN_SAMPLE_RATE: u32 = 8000;
pub const MAX_SAMPLE_RATE: u32 = 192000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PcmError {
    SampleRate(u32),
    Rate { numerator: u32, denominator: u32 },
    Frames { requested: usize },
    Scratch { required: usize },
    Stream(AdlibStreamError),
    FrameOverflow,
}
impl fmt::Display for PcmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid AdLib PCM request: {self:?}")
    }
}
impl core::error::Error for PcmError {}
impl From<AdlibStreamError> for PcmError {
    fn from(error: AdlibStreamError) -> Self {
        Self::Stream(error)
    }
}

/// Exact driver calls per second, independent of game ticks and output chunks.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AudioRate {
    numerator: u32,
    denominator: u32,
}
impl AudioRate {
    /// Pinned DOS VGA refresh/IRQ path: one AdLib call per VGA IRQ.
    pub const DOS_VGA: Self = Self {
        numerator: 2190197,
        denominator: 31250,
    };
    /// Pinned DOS -x path: PIT 1193182 Hz / divisor 16384, one call per IRQ.
    pub const DOS_UNLOCKED: Self = Self {
        numerator: 596591,
        denominator: 8192,
    };
    /// Accept 1..=1000 calls/second and reduce the fraction. Lower bounds ensure
    /// every driver interval contains at least eight samples at supported rates.
    pub fn new(numerator: u32, denominator: u32) -> Result<Self, PcmError> {
        if denominator == 0
            || numerator < denominator
            || u64::from(numerator) > u64::from(denominator) * 1000
        {
            return Err(PcmError::Rate {
                numerator,
                denominator,
            });
        }
        let mut a = numerator;
        let mut b = denominator;
        while b != 0 {
            let r = a % b;
            a = b;
            b = r;
        }
        Ok(Self {
            numerator: numerator / a,
            denominator: denominator / a,
        })
    }
    pub const fn numerator(self) -> u32 {
        self.numerator
    }
    pub const fn denominator(self) -> u32 {
        self.denominator
    }
    fn interval(self, sample_rate: u32, remainder: &mut u64) -> u32 {
        let total = *remainder + u64::from(sample_rate) * u64::from(self.denominator);
        *remainder = total % u64::from(self.numerator);
        (total / u64::from(self.numerator)) as u32
    }
}
/// Sample count and retained scheduling phase; no device time or game clock.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PcmPosition {
    pub frames: u64,
    pub interval_remainder: u64,
    pub remaining_frames: u32,
}

/// Borrowed driver and owned OPL2 state. Register writes occur before the first
/// sample of their interval. The nth interval ends at floor(n * rate * den/num).
/// The fixed OPL pipeline's left signal is delivered as mono, saturating to
/// i16 without adaptive normalization. Serialize samples using `i16::to_le_bytes`.
pub struct AdlibPcm<'a> {
    image: AdlibImage<'a>,
    driver: AdlibPlayer<'a>,
    chip: Opl2,
    sample_rate: u32,
    call_rate: AudioRate,
    position: PcmPosition,
}
impl<'a> AdlibPcm<'a> {
    pub fn new(
        image: AdlibImage<'a>,
        sample_rate: u32,
        call_rate: AudioRate,
    ) -> Result<Self, PcmError> {
        if !(MIN_SAMPLE_RATE..=MAX_SAMPLE_RATE).contains(&sample_rate) {
            return Err(PcmError::SampleRate(sample_rate));
        }
        Ok(Self {
            image,
            driver: AdlibPlayer::new(image),
            chip: Opl2::new(sample_rate).expect("validated sample rate"),
            sample_rate,
            call_rate,
            position: PcmPosition::default(),
        })
    }
    pub const fn position(&self) -> PcmPosition {
        self.position
    }
    pub const fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    pub const fn call_rate(&self) -> AudioRate {
        self.call_rate
    }
    pub const fn driver_state(&self) -> AdlibState {
        self.driver.state()
    }
    /// Queue controls for the next driver boundary; do not discard queued PCM in
    /// a device here. The platform owns device queues/generations separately.
    pub fn request_music(&mut self, index: usize) -> Result<(), PcmError> {
        self.driver.request_music(index)?;
        Ok(())
    }
    pub fn request_effect(&mut self, index: usize) -> Result<(), PcmError> {
        self.driver.request_effect(index)?;
        Ok(())
    }
    pub fn request_stop(&mut self) {
        self.driver.request_stop();
    }
    /// Start a fresh driver, OPL chip, resampler and sample phase. Invalid music
    /// indices preserve the whole previous stream, including synthesizer history.
    pub fn reset_music(&mut self, index: usize) -> Result<(), PcmError> {
        self.image.music(index).map_err(AdlibStreamError::from)?;
        let mut next = Self::new(self.image, self.sample_rate, self.call_rate)?;
        next.request_music(index)?;
        *self = next;
        Ok(())
    }
    /// Fill any chunk up to MAX_PCM_FRAMES. `scratch` is caller-owned register
    /// workspace, not PCM output. Validate all future VM calls and workspace
    /// capacity before touching PCM, the driver, phase or synthesizer.
    /// `command_budget` applies separately to each bounded driver call.
    pub fn render(
        &mut self,
        pcm: &mut [i16],
        scratch: &mut [AdlibWrite],
        command_budget: usize,
    ) -> Result<(), PcmError> {
        if pcm.len() > MAX_PCM_FRAMES {
            return Err(PcmError::Frames {
                requested: pcm.len(),
            });
        }
        let end = self
            .position
            .frames
            .checked_add(pcm.len() as u64)
            .ok_or(PcmError::FrameOverflow)?;
        let mut driver = self.driver;
        let mut phase = self.position.interval_remainder;
        let mut remaining = self.position.remaining_frames as usize;
        let mut requested = pcm.len();
        while requested > remaining {
            requested -= remaining;
            let count = driver.advance_silent(command_budget)?;
            if count > scratch.len() {
                return Err(PcmError::Scratch { required: count });
            }
            remaining = self.call_rate.interval(self.sample_rate, &mut phase) as usize;
        }
        for sample in pcm {
            if self.position.remaining_frames == 0 {
                // Preflight ran this same immutable VM image/control sequence.
                // No external callbacks or mutable asset aliases can change it.
                let count = self
                    .driver
                    .step(scratch, command_budget)
                    .expect("preflight validated immutable ADLIB calls");
                for write in &scratch[..count] {
                    self.chip.write_reg(write.register, write.value);
                }
                self.position.remaining_frames = self
                    .call_rate
                    .interval(self.sample_rate, &mut self.position.interval_remainder);
            }
            *sample = mono_i16(self.chip.render_frame().0);
            self.position.remaining_frames -= 1;
        }
        self.position.frames = end;
        Ok(())
    }
}
fn mono_i16(sample: i32) -> i16 {
    sample.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mono_conversion_clips_both_bounds_and_preserves_in_range() {
        for (input, want) in [
            (i32::MIN, i16::MIN),
            (-32769, i16::MIN),
            (-32768, i16::MIN),
            (-1, -1),
            (0, 0),
            (32767, i16::MAX),
            (32768, i16::MAX),
            (i32::MAX, i16::MAX),
        ] {
            assert_eq!(mono_i16(input), want);
        }
    }
    #[test]
    fn rational_interval_carries_without_cumulative_rounding() {
        for rate in [8000, 44100, 48000, 49716, 96000, 192000] {
            let mut rem = 0;
            let mut frames = 0_u64;
            for n in 1..=10000_u64 {
                frames += u64::from(AudioRate::DOS_VGA.interval(rate, &mut rem));
                assert_eq!(frames, n * u64::from(rate) * 31250 / 2190197);
            }
            assert!(rem < 2190197);
        }
    }
}
