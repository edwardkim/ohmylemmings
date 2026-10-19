use super::*;

impl Actor {
    /// Assign by stable ID. Game.pas AssignBuilder checks the current animation's
    /// head against HEAD_MIN_Y=-5, then starts twelve bricks without moving.
    pub fn assign_builder(&mut self, remaining: &mut u16) -> bool {
        self.assign_builder_check(
            remaining,
            i64::from(self.state.y) - i64::from(self.state.action.foot_y()) < -5,
        )
    }
    /// AssignBuilder's secondary branch checks HEAD_MIN_Y on the primary,
    /// even when the secondary's head lies above it.
    pub fn assign_builder_secondary(&mut self, remaining: &mut u16, primary: ActorState) -> bool {
        self.assign_builder_check(
            remaining,
            i64::from(primary.y) - i64::from(primary.action.foot_y()) < -5,
        )
    }
    fn assign_builder_check(&mut self, remaining: &mut u16, head_above: bool) -> bool {
        let s = self.state;
        if *remaining == 0
            || s.removed.is_some()
            || head_above
            || !matches!(
                s.action,
                MotionAction::Walking
                    | MotionAction::Digging
                    | MotionAction::Bashing
                    | MotionAction::Mining
                    | MotionAction::Shrugging
            )
        {
            return false;
        }
        transition(&mut self.state, MotionAction::Building, self.rules);
        *remaining -= 1;
        true
    }

    /// Game.pas LayBrick/HandleBuilding: frame9 lays six cells, and frame10
    /// repeats the brick when nine remain. Frame0 climbs one and moves two, with ordered
    /// foot/head/boundary checks. All arithmetic precedes this actor's edits.
    pub(super) fn advance_building(
        &mut self,
        terrain: &mut TerrainMaskMut<'_>,
    ) -> Result<MotionOutcome, MotionError> {
        let mut s = self.state;
        s.end_of_animation = s.frame == 15;
        s.frame = (s.frame + 1) % 16;
        let lays = s.frame == 9 || (s.frame == 10 && s.bricks_left == 9);
        let mut out = MotionOutcome {
            check_objects: !lays,
            ..MotionOutcome::default()
        };
        if lays {
            let left = if s.direction == MotionDirection::Left {
                s.x.checked_sub(4).ok_or(MotionError::CoordinateOverflow)?
            } else {
                s.x
            };
            left.checked_add(5).ok_or(MotionError::CoordinateOverflow)?;
            let y = s.y.checked_sub(1).ok_or(MotionError::CoordinateOverflow)?;
            for (index, x) in (left..=left + 5).enumerate() {
                if terrain.set_solid(x, y, true).is_some_and(|change| change) {
                    out.added_pixels[index] = Some((x, y));
                }
            }
        } else if s.frame == 0 {
            s.x =
                s.x.checked_add(s.direction.delta())
                    .ok_or(MotionError::CoordinateOverflow)?;
            s.y = add_y(s.y, -1)?;
            if s.x <= 0 || s.x > 1647 || pixel(terrain.as_read_only(), s.x, i64::from(s.y) - 1, -1)
            {
                stop(&mut s, self.rules);
            } else {
                s.x =
                    s.x.checked_add(s.direction.delta())
                        .ok_or(MotionError::CoordinateOverflow)?;
                if pixel(terrain.as_read_only(), s.x, i64::from(s.y) - 1, -1) {
                    stop(&mut s, self.rules);
                } else {
                    s.bricks_left -= 1;
                    if s.bricks_left == 0 {
                        transition(&mut s, MotionAction::Shrugging, self.rules);
                        top(&mut s, self.rules);
                    } else if pixel(
                        terrain.as_read_only(),
                        s.x.checked_add(s.direction.delta() * 2)
                            .ok_or(MotionError::CoordinateOverflow)?,
                        i64::from(s.y) - 9,
                        -9,
                    ) || s.x <= 0
                        || s.x > 1647
                    {
                        stop(&mut s, self.rules);
                    } else if i64::from(s.y) - i64::from(s.action.foot_y()) < -5 {
                        transition(&mut s, MotionAction::Walking, self.rules);
                        top(&mut s, self.rules);
                    }
                }
            }
        }
        self.state = s;
        Ok(out)
    }
}

fn stop(s: &mut ActorState, rules: MotionRules) {
    transition(s, MotionAction::Walking, rules);
    s.direction = s.direction.reverse();
    top(s, rules);
}
