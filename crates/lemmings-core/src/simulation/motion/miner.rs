use super::*;

/// Caller-owned 16×13 MSB-first erase masks: two frames right, then two left.
/// The DOS original right one-way mining bug is enabled by default. It affects
/// the foot check during mining, not the directional assignment check.
#[derive(Clone, Copy, Debug)]
pub struct MiningContext<'a> {
    pub masks: &'a [u8; 104],
    pub objects: BashingContext<'a>,
    pub one_way_right_bug: bool,
}
impl<'a> MiningContext<'a> {
    pub const fn new(masks: &'a [u8; 104], objects: BashingContext<'a>) -> Self {
        Self {
            masks,
            objects,
            one_way_right_bug: true,
        }
    }
}
impl Actor {
    /// Consume a Miner only after current action, front object and foot steel
    /// accept it. Selection priority remains the caller's stable-ID decision.
    pub fn assign_miner(&mut self, remaining: &mut u16, context: MiningContext<'_>) -> bool {
        let s = self.state;
        if *remaining == 0
            || s.removed.is_some()
            || !matches!(
                s.action,
                MotionAction::Walking
                    | MotionAction::Digging
                    | MotionAction::Bashing
                    | MotionAction::Building
                    | MotionAction::Shrugging
            )
            || context.objects.assignment_steel(s)
            || context.objects.assignment_blocked(s)
        {
            return false;
        }
        let Some(y) = s.y.checked_add(1) else {
            return false;
        };
        transition(&mut self.state, MotionAction::Mining, self.rules);
        self.state.y = y;
        *remaining -= 1;
        true
    }

    /// Advance all implemented jobs. Arithmetic is planned before committing
    /// this actor's state/pixels; prior actors' successful changes are retained.
    pub fn advance_with_jobs(
        &mut self,
        terrain: &mut TerrainMaskMut<'_>,
        context: MiningContext<'_>,
    ) -> Result<MotionOutcome, MotionError> {
        self.require_basic_context()?;
        if self.state.action == MotionAction::Building && self.state.removed.is_none() {
            return self.advance_building(terrain);
        }
        if self.state.removed.is_some() || self.state.action != MotionAction::Mining {
            return self.advance_with_skills(terrain, context.objects);
        }
        let mut s = self.state;
        s.end_of_animation = s.frame == 23;
        s.frame = (s.frame + 1) % 24;
        let mut out = MotionOutcome::default();
        if matches!(s.frame, 1 | 2) {
            let offset = i32::from(s.frame == 2);
            let left =
                s.x.checked_add(s.direction.delta() * offset)
                    .and_then(|x| x.checked_sub(8))
                    .ok_or(MotionError::CoordinateOverflow)?;
            let top =
                s.y.checked_add(offset)
                    .and_then(|y| y.checked_sub(13))
                    .ok_or(MotionError::CoordinateOverflow)?;
            left.checked_add(15)
                .ok_or(MotionError::CoordinateOverflow)?;
            top.checked_add(12).ok_or(MotionError::CoordinateOverflow)?;
            let frame = usize::from(s.frame - 1)
                + if s.direction == MotionDirection::Left {
                    2
                } else {
                    0
                };
            let bytes = &context.masks[frame * 26..frame * 26 + 26];
            for y in 0..13 {
                for x in 0..16 {
                    if bytes[y * 2 + x / 8] & (128 >> (x % 8)) != 0 {
                        terrain.set_solid(left + x as i32, top + y as i32, false);
                    }
                }
            }
        } else if matches!(s.frame, 3 | 15) {
            out.check_objects = true;
            for _ in 0..2 {
                s.x =
                    s.x.checked_add(s.direction.delta())
                        .ok_or(MotionError::CoordinateOverflow)?;
                if !(0..=1647).contains(&s.x) {
                    transition(&mut s, MotionAction::Walking, self.rules);
                    s.direction = s.direction.reverse();
                    self.state = s;
                    return Ok(out);
                }
            }
            if s.frame == 3 {
                s.y = add_y(s.y, 1)?;
            }
            if s.y > 163 && s.frame == 3 {
                remove(&mut s, &mut out, RemovalReason::BottomBoundary);
            } else if !pixel(terrain.as_read_only(), s.x, i64::from(s.y), 0) {
                transition(&mut s, MotionAction::Falling, self.rules);
            } else if context
                .objects
                .blocked(i64::from(s.x), i64::from(s.y), s.direction)
                || (context.one_way_right_bug
                    && context.objects.object_map.map_or_else(
                        || {
                            context
                                .objects
                                .one_way_right
                                .is_some_and(|m| m.is_solid(s.x, s.y))
                        },
                        |map| map.read(i64::from(s.x), i64::from(s.y)) == 136,
                    ))
            {
                transition(&mut s, MotionAction::Walking, self.rules);
                s.direction = s.direction.reverse();
            }
        } else if s.frame == 0 {
            s.y = add_y(s.y, 1)?;
            out.check_objects = true;
            if s.y > 163 {
                remove(&mut s, &mut out, RemovalReason::BottomBoundary);
            }
        }
        self.state = s;
        Ok(out)
    }
}

/// Strict ID-ordered shared terrain updates for all currently implemented jobs.
pub fn step_actors_with_jobs(
    actors: &mut [Actor],
    terrain: &mut TerrainMaskMut<'_>,
    context: MiningContext<'_>,
    emit: impl FnMut(ActorState, MotionOutcome),
) -> Result<ActorTick, MotionError> {
    super::basher::step_mutating(actors, |a| a.advance_with_jobs(terrain, context), emit)
}
