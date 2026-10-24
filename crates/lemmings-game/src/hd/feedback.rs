//! Small cached aiming reticles. No simulation or host dependencies.
pub const SIZE: u32 = 84;
/// Impact pose in display pixels only; the logical foot and collision stay fixed.
pub fn impact_pose(rect: &mut [i32; 4], action: lemmings_core::MotionAction, frame: u16) {
    use lemmings_core::MotionAction;
    let old = *rect;
    match action {
        MotionAction::Digging if frame % 8 < 3 => {
            let squash = (3 - i32::from(frame % 8)) * 3;
            rect[1] += squash;
            rect[3] = (rect[3] - squash).max(1);
        }
        MotionAction::Splatting => {
            rect[2] = old[2] * 3 / 2;
            rect[3] = (old[3] / if frame < 3 { 3 } else { 5 }).max(1);
            rect[0] -= (rect[2] - old[2]) / 2;
            rect[1] += old[3] - rect[3];
        }
        _ => {}
    }
}

pub fn reticle(unavailable: bool) -> Vec<u8> {
    let mut rgba = vec![0; (SIZE * SIZE * 4) as usize];
    for y in 0..SIZE as i32 {
        for x in 0..SIZE as i32 {
            let (dx, dy) = (x - 42, y - 42);
            let distance = dx * dx + dy * dy;
            let ring = (34 * 34..=37 * 37).contains(&distance);
            let ticks = (dx.abs() <= 1 && (30..=41).contains(&dy.abs()))
                || (dy.abs() <= 1 && (30..=41).contains(&dx.abs()));
            let cross =
                unavailable && dx.abs() <= 12 && dy.abs() <= 12 && (dx.abs() - dy.abs()).abs() <= 1;
            if ring || ticks || cross {
                let at = (y as usize * SIZE as usize + x as usize) * 4;
                rgba[at..at + 4].copy_from_slice(if unavailable {
                    &[255, 125, 100, 255]
                } else {
                    &[116, 238, 220, 255]
                });
            }
        }
    }
    rgba
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fatal_impact_is_flattened_but_safe_falling_and_walkers_are_unchanged() {
        let original = [100, 100, 60, 90];
        for action in [
            lemmings_core::MotionAction::Walking,
            lemmings_core::MotionAction::Falling,
        ] {
            let mut rect = original;
            impact_pose(&mut rect, action, 0);
            assert_eq!(rect, original);
        }
        let mut rect = original;
        impact_pose(&mut rect, lemmings_core::MotionAction::Splatting, 0);
        assert_eq!(rect[1] + rect[3], 190);
        assert!(rect[3] < original[3] / 2 && rect[2] > original[2]);
        let mut digging = original;
        impact_pose(&mut digging, lemmings_core::MotionAction::Digging, 8);
        assert_eq!(digging[1] + digging[3], 190);
    }
}
