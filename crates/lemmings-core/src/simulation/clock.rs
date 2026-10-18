use core::{fmt, time::Duration};

/// Logical control calls per second, following the compatibility profile.
pub const TICKS_PER_SECOND: u32 = 17;
const PHASE_DENOMINATOR: u128 = 1_000_000_000;

/// Adapter-owned integer time accumulator, separate from simulation snapshots.
/// No elapsed time is clamped and no due calls are silently dropped. The caller
/// must process every returned call, including calls while gameplay is paused.
/// Splitting the same total nanoseconds into render frames preserves the result.
///
/// ```
/// use core::time::Duration;
/// use lemmings_core::FixedTickClock;
/// let mut clock = FixedTickClock::new();
/// assert_eq!(clock.advance(Duration::from_nanos(58_823_529))?, 0);
/// assert_eq!(clock.advance(Duration::from_nanos(1))?, 1);
/// assert_eq!(clock.phase_numerator(), 10);
/// # Ok::<(), lemmings_core::TickOverflow>(())
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FixedTickClock {
    phase: u32,
}

impl FixedTickClock {
    pub const fn new() -> Self {
        Self { phase: 0 }
    }

    /// Fraction of the next call, in billionths of a tick; always below 10^9.
    pub const fn phase_numerator(&self) -> u32 {
        self.phase
    }

    /// Accumulate a supplied duration, returning all whole due control calls.
    /// Rejects a result above u64::MAX without changing the fractional phase.
    pub fn advance(&mut self, elapsed: Duration) -> Result<u64, TickOverflow> {
        // Duration::MAX is less than 2^94 nanoseconds; multiplying by 17 fits u128.
        let scaled = elapsed.as_nanos() * u128::from(TICKS_PER_SECOND) + u128::from(self.phase);
        let due = u64::try_from(scaled / PHASE_DENOMINATOR).map_err(|_| TickOverflow)?;
        self.phase = (scaled % PHASE_DENOMINATOR) as u32;
        Ok(due)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TickOverflow;
impl fmt::Display for TickOverflow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("elapsed duration produces more than u64::MAX control calls")
    }
}
impl core::error::Error for TickOverflow {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_and_zero_time_have_no_due_calls() {
        let mut clock = FixedTickClock::new();
        assert_eq!(clock, FixedTickClock::default());
        assert_eq!(clock.advance(Duration::ZERO), Ok(0));
        assert_eq!(clock.phase_numerator(), 0);
    }

    #[test]
    fn rational_tick_boundary_preserves_subnanosecond_fraction() {
        let mut clock = FixedTickClock::new();
        assert_eq!(clock.advance(Duration::from_nanos(58_823_529)), Ok(0));
        assert_eq!(clock.phase_numerator(), 999_999_993);
        assert_eq!(clock.advance(Duration::from_nanos(1)), Ok(1));
        assert_eq!(clock.phase_numerator(), 10);
        assert_eq!(clock.advance(Duration::from_nanos(941_176_470)), Ok(16));
        assert_eq!(clock.phase_numerator(), 0);
    }

    #[test]
    fn seconds_and_long_frames_return_every_call_without_a_catchup_cap() {
        let mut clock = FixedTickClock::new();
        assert_eq!(clock.advance(Duration::from_secs(1)), Ok(17));
        assert_eq!(clock.advance(Duration::from_secs(60 * 60)), Ok(61_200));
        assert_eq!(clock.phase_numerator(), 0);
    }

    #[test]
    fn repeated_sixty_millisecond_frames_do_not_round_the_logical_rate() {
        let mut clock = FixedTickClock::new();
        let due: u64 = (0..1000)
            .map(|_| clock.advance(Duration::from_millis(60)).unwrap())
            .sum();
        assert_eq!(due, 1020);
        assert_eq!(clock.phase_numerator(), 0);
    }

    #[test]
    fn all_two_frame_millisecond_partitions_match_exact_total_time() {
        let mut cases = 0;
        for total in 0..=1000 {
            for first in 0..=total {
                let mut clock = FixedTickClock::new();
                let due = clock.advance(Duration::from_millis(first)).unwrap()
                    + clock.advance(Duration::from_millis(total - first)).unwrap();
                assert_eq!(due, total * 17 / 1000);
                assert_eq!(
                    u64::from(clock.phase_numerator()),
                    total * 17 % 1000 * 1_000_000
                );
                cases += 1;
            }
        }
        assert_eq!(cases, 501_501);
    }

    #[test]
    fn largest_whole_seconds_with_representable_due_count_succeeds() {
        let mut clock = FixedTickClock::new();
        let seconds = u64::MAX / 17;
        assert_eq!(
            clock.advance(Duration::from_secs(seconds)),
            Ok(seconds * 17)
        );
        assert_eq!(clock.phase_numerator(), 0);
    }

    #[test]
    fn duration_overflow_preserves_phase_and_allows_recovery() {
        let mut clock = FixedTickClock::new();
        clock.advance(Duration::from_nanos(1)).unwrap();
        let before = clock;
        assert_eq!(clock.advance(Duration::MAX), Err(TickOverflow));
        assert_eq!(clock, before);
        assert_eq!(
            clock.advance(Duration::from_secs(u64::MAX / 17 + 1)),
            Err(TickOverflow)
        );
        assert_eq!(clock, before);
        assert_eq!(clock.advance(Duration::from_nanos(58_823_529)), Ok(1));
        assert_eq!(clock.phase_numerator(), 10);
    }
}
