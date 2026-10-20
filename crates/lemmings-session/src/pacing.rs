use std::time::Duration;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Pacing {
    Dos,
    Reference,
}
impl Pacing {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Dos => "dos-vga-5",
            Self::Reference => "reference-17",
        }
    }
    pub const fn rate(self) -> (u32, u32) {
        match self {
            // #30: pinned VGA video rate 2190197/31250; observed GUI PC2
            // waits five video frames per logical call. This is adapter pacing,
            // separate from the unchanged 17-call simulation game second.
            Self::Dos => (2_190_197, 156_250),
            Self::Reference => (17, 1),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlaybackClock {
    pacing: Pacing,
    phase: u64,
}
impl PlaybackClock {
    pub const fn new(pacing: Pacing) -> Self {
        Self { pacing, phase: 0 }
    }
    pub fn advance(&mut self, elapsed: Duration) -> Result<u64, lemmings_core::TickOverflow> {
        let (num, den) = self.pacing.rate();
        let denominator = u128::from(den) * 1_000_000_000;
        // Duration::MAX * this fixed numerator fits u128. Commit phase only
        // after the entire due count is representable; never clamp/drop debt.
        let scaled = elapsed.as_nanos() * u128::from(num) + u128::from(self.phase);
        let due = u64::try_from(scaled / denominator).map_err(|_| lemmings_core::TickOverflow)?;
        self.phase = (scaled % denominator) as u64;
        Ok(due)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn dos_profile_boundaries_follow_five_video_frames() {
        let mut clock = PlaybackClock::new(Pacing::Dos);
        assert_eq!(clock.advance(Duration::from_nanos(71_340_614)).unwrap(), 0);
        assert_eq!(clock.advance(Duration::from_nanos(1)).unwrap(), 1);
        assert_eq!(
            PlaybackClock::new(Pacing::Dos)
                .advance(Duration::from_secs(1))
                .unwrap(),
            14
        );
    }

    #[test]
    fn all_frame_partitions_return_the_same_due_calls() {
        for pacing in [Pacing::Dos, Pacing::Reference] {
            let (num, den) = pacing.rate();
            for total in 0..=1000_u64 {
                for first in 0..=total {
                    let mut clock = PlaybackClock::new(pacing);
                    let got = clock.advance(Duration::from_millis(first)).unwrap()
                        + clock.advance(Duration::from_millis(total - first)).unwrap();
                    assert_eq!(got, total * u64::from(num) / (1000 * u64::from(den)));
                }
            }
        }
    }

    #[test]
    fn long_frame_debt_is_returned_without_drop() {
        let mut clock = PlaybackClock::new(Pacing::Dos);
        assert_eq!(clock.advance(Duration::from_secs(3600)).unwrap(), 50_462);
        assert_eq!(clock.advance(Duration::ZERO).unwrap(), 0);
    }

    #[test]
    fn overflow_is_atomic_and_recoverable() {
        let mut clock = PlaybackClock::new(Pacing::Dos);
        clock.advance(Duration::from_nanos(1)).unwrap();
        let before = clock;
        assert!(clock.advance(Duration::MAX).is_err());
        assert_eq!(clock, before);
        assert_eq!(clock.advance(Duration::from_nanos(71_340_614)).unwrap(), 1);
    }

    #[test]
    fn reference_profile_keeps_existing_seventeen_hertz_contract() {
        let mut clock = PlaybackClock::new(Pacing::Reference);
        assert_eq!(clock.advance(Duration::from_nanos(58_823_529)).unwrap(), 0);
        assert_eq!(clock.advance(Duration::from_nanos(1)).unwrap(), 1);
        assert_eq!(
            clock.advance(Duration::from_nanos(941_176_470)).unwrap(),
            16
        );
    }
}
