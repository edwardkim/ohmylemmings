//! Native60Hz deadlines; rendering cost does not move the absolute grid.
//! Late slots are skipped in constant time. Simulation pacing is independent.
use std::time::{Duration, Instant};
#[derive(Clone, Copy)]
pub struct FrameBudget {
    pub scheduled: Instant,
    pub deadline: Instant,
}
pub struct FrameClock {
    epoch: Instant,
    period_units: u128,
    next: u128,
    visible: bool,
}
impl FrameClock {
    pub fn new(now: Instant, every: u64) -> Self {
        assert!((1..=10000).contains(&every));
        Self {
            epoch: now,
            period_units: u128::from(every) * 1_000_000_000,
            next: 0,
            visible: true,
        }
    }
    fn at(&self, slot: u128) -> Instant {
        let ns = (slot * self.period_units).div_ceil(60);
        self.epoch
            + Duration::new(
                u64::try_from(ns / 1_000_000_000).expect("display time fits Duration"),
                (ns % 1_000_000_000) as u32,
            )
    }
    pub fn reset(&mut self, now: Instant) {
        self.epoch = now;
        self.next = 0;
    }
    pub fn set_visible(&mut self, visible: bool, now: Instant) {
        if self.visible != visible {
            self.visible = visible;
            self.reset(now);
        }
    }
    pub fn due(&self, now: Instant) -> bool {
        self.visible && now >= self.at(self.next)
    }
    /// Forced initial/result/capture draws can occur without consuming a slot.
    pub fn begin(&mut self, now: Instant) -> Option<FrameBudget> {
        if !self.due(now) {
            return None;
        }
        let scheduled = self.at(self.next);
        self.next += 1;
        Some(FrameBudget {
            scheduled,
            deadline: self.at(self.next),
        })
    }
    /// Consume only scheduled frames. No repeated drawing of expired slots.
    pub fn finish(&mut self, now: Instant) -> u128 {
        let future =
            now.saturating_duration_since(self.epoch).as_nanos() * 60 / self.period_units + 1;
        let skipped = future.saturating_sub(self.next);
        self.next = self.next.max(future);
        skipped
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn processing_time_does_not_accumulate_in_sixty_hz_deadlines() {
        let base = Instant::now();
        let mut c = FrameClock::new(base, 1);
        for frame in 0..=3600u64 {
            let ns = (u128::from(frame) * 1_000_000_000).div_ceil(60) as u64;
            let at = base + Duration::from_nanos(ns);
            let budget = c.begin(at).expect("frame must be due on its absolute grid");
            assert_eq!(budget.scheduled, at);
            assert_eq!(
                budget.deadline,
                base + Duration::from_nanos(
                    ((u128::from(frame) + 1) * 1_000_000_000).div_ceil(60) as u64
                )
            );
            assert_eq!(c.finish(at + Duration::from_millis(2)), 0);
            assert!(!c.due(at + Duration::from_millis(15)));
        }
    }
    #[test]
    fn overdue_slots_are_skipped_once_without_a_catch_up_burst() {
        let base = Instant::now();
        let mut c = FrameClock::new(base, 1);
        c.begin(base).unwrap();
        assert_eq!(c.finish(base + Duration::from_millis(101)), 6);
        assert!(!c.due(base + Duration::from_millis(101)));
        assert!(c.due(base + Duration::from_nanos(116_666_667)));
    }
    #[test]
    fn forced_frames_do_not_postpone_grid_and_render_every_scales_sixty_hz() {
        let base = Instant::now();
        let mut c = FrameClock::new(base, 3);
        c.begin(base).unwrap();
        c.finish(base + Duration::from_millis(2));
        assert!(c.begin(base + Duration::from_millis(20)).is_none());
        assert!(!c.due(base + Duration::from_nanos(49_999_999)));
        assert!(c.due(base + Duration::from_millis(50)));
    }
    #[test]
    fn hide_restore_and_new_level_rebase_only_the_presentation_epoch() {
        let base = Instant::now();
        let mut c = FrameClock::new(base, 1);
        c.begin(base).unwrap();
        c.finish(base + Duration::from_millis(2));
        c.set_visible(false, base + Duration::from_secs(1));
        assert!(!c.due(base + Duration::from_secs(600)));
        c.set_visible(true, base + Duration::from_secs(600));
        assert!(c.begin(base + Duration::from_secs(600)).is_some());
        assert_eq!(
            c.finish(base + Duration::from_secs(600) + Duration::from_millis(2)),
            0
        );
        assert!(!c.due(base + Duration::from_secs(600) + Duration::from_millis(10)));
        c.reset(base + Duration::from_secs(601));
        assert_eq!(
            c.begin(base + Duration::from_secs(601)).unwrap().scheduled,
            base + Duration::from_secs(601)
        );
    }
}
