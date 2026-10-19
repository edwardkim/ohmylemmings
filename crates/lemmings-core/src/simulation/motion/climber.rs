use super::*;
impl Actor {
    /// Permanent ability; original DOS shrugger bug changes only the action.
    pub fn assign_climber(&mut self, remaining: &mut u16) -> bool {
        let s = &mut self.state;
        if *remaining == 0
            || s.removed.is_some()
            || s.is_climber
            || matches!(
                s.action,
                MotionAction::Blocking | MotionAction::Splatting | MotionAction::Exploding
            )
        {
            return false;
        }
        s.is_climber = true;
        *remaining -= 1;
        if s.action == MotionAction::Shrugging && self.rules.assign_climber_shrugger_bug {
            s.action = MotionAction::Walking;
            s.walking_once = true;
        }
        true
    }
}
pub(super) fn advance(
    s: &mut ActorState,
    terrain: TerrainMask<'_>,
    rules: MotionRules,
    out: &mut MotionOutcome,
) -> Result<(), MotionError> {
    if s.action == MotionAction::Climbing {
        if s.frame <= 3 {
            if !pixel(terrain, s.x, i64::from(s.y) - 7 - i64::from(s.frame), 0) {
                s.y = add_y(s.y, 2 - i32::from(s.frame))?;
                transition(s, MotionAction::Hoisting, rules);
                top(s, rules);
            }
        } else {
            s.y = add_y(s.y, -1)?;
            if i64::from(s.y) - 12 < -5
                || i32::try_from(i64::from(s.x) - i64::from(s.direction.delta()))
                    .is_ok_and(|x| pixel(terrain, x, i64::from(s.y) - 8, -8))
            {
                s.direction = s.direction.reverse();
                transition(s, MotionAction::Falling, rules);
                s.x =
                    s.x.checked_add(s.direction.delta() * 2)
                        .ok_or(MotionError::CoordinateOverflow)?;
            }
        }
    } else {
        out.check_objects = false;
        if s.frame <= 4 {
            s.y = add_y(s.y, -2)?;
            top(s, rules);
            out.check_objects = true;
        } else if s.end_of_animation {
            transition(s, MotionAction::Walking, rules);
            top(s, rules);
            out.check_objects = true;
        }
    }
    Ok(())
}
