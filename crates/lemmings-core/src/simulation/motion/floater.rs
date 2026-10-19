use super::*;
// Game.pas FloatParametersTable: opening0..7, then the steady8..15 loop.
const FLOAT_PARAMETERS: [(i32, u8); 16] = [
    (3, 1),
    (3, 2),
    (3, 3),
    (3, 5),
    (-1, 5),
    (0, 5),
    (1, 5),
    (1, 5),
    (2, 5),
    (2, 6),
    (2, 7),
    (2, 7),
    (2, 6),
    (2, 5),
    (2, 4),
    (2, 4),
];
impl Actor {
    /// Permanent ability only: assignment never changes the current action.
    pub fn assign_floater(&mut self, remaining: &mut u16) -> bool {
        let s = &mut self.state;
        if *remaining == 0
            || s.removed.is_some()
            || s.is_floater
            || matches!(
                s.action,
                MotionAction::Blocking | MotionAction::Splatting | MotionAction::Exploding
            )
        {
            return false;
        }
        s.is_floater = true;
        *remaining -= 1;
        true
    }
}
pub(super) fn advance(
    s: &mut ActorState,
    terrain: TerrainMask<'_>,
    rules: MotionRules,
    out: &mut MotionOutcome,
) -> Result<(), MotionError> {
    let (mut dy, frame) = FLOAT_PARAMETERS[usize::from(s.float_index)];
    s.frame = frame;
    s.float_index += 1;
    if s.float_index >= 16 {
        s.float_index = 8;
    }
    if dy <= 0 {
        s.y = add_y(s.y, dy)?;
    } else {
        let mut min_y = 0;
        while dy > 0 {
            if pixel(terrain, s.x, i64::from(s.y), min_y) {
                transition(s, MotionAction::Walking, rules);
                return Ok(());
            }
            s.y = add_y(s.y, 1)?;
            dy -= 1;
            min_y += 1;
        }
    }
    if s.y > 163 {
        remove(s, out, RemovalReason::BottomBoundary);
    }
    Ok(())
}
