use super::*;

/// Caller-owned, MSB-first masks: four 16×10 frames facing right, then four
/// facing left (20 bytes/frame). A stored one erases terrain. Restrictions are
/// object-map views, not pixel protection; use mutually exclusive object cells.
#[derive(Clone, Copy, Debug)]
pub struct BashingContext<'a> {
    pub masks: &'a [u8; 160],
    pub object_map: Option<crate::ObjectMap<'a>>,
    pub steel: TerrainMask<'a>,
    pub one_way_left: Option<TerrainMask<'a>>,
    pub one_way_right: Option<TerrainMask<'a>>,
}
impl<'a> BashingContext<'a> {
    pub const fn new(masks: &'a [u8; 160], steel: TerrainMask<'a>) -> Self {
        Self {
            masks,
            object_map: None,
            steel,
            one_way_left: None,
            one_way_right: None,
        }
    }
    pub(super) fn is_steel(self, x: i32, y: i32) -> bool {
        self.object_map.map_or_else(
            || self.steel.is_solid(x, y),
            |map| map.read(i64::from(x), i64::from(y)) == 137,
        )
    }
    pub(super) fn assignment_steel(self, s: ActorState) -> bool {
        if self.object_map.is_some() {
            s.object_below == 137
        } else {
            self.is_steel(s.x, s.y)
        }
    }
    pub(super) fn assignment_blocked(self, s: ActorState) -> bool {
        if self.object_map.is_some() {
            s.object_in_front == 137
                || (s.object_in_front == 135 && s.direction != MotionDirection::Left)
                || (s.object_in_front == 136 && s.direction != MotionDirection::Right)
        } else {
            self.blocked(
                i64::from(s.x) + i64::from(s.direction.delta()) * 8,
                i64::from(s.y) - 8,
                s.direction,
            )
        }
    }
    pub(super) fn blocked(self, x: i64, y: i64, direction: MotionDirection) -> bool {
        let (Ok(x), Ok(y)) = (i32::try_from(x), i32::try_from(y)) else {
            return false;
        };
        if let Some(map) = self.object_map {
            let value = map.read(i64::from(x), i64::from(y));
            return value == 137
                || (value == 135 && direction != MotionDirection::Left)
                || (value == 136 && direction != MotionDirection::Right);
        }
        self.steel.is_solid(x, y)
            || (direction != MotionDirection::Left
                && self.one_way_left.is_some_and(|m| m.is_solid(x, y)))
            || (direction != MotionDirection::Right
                && self.one_way_right.is_some_and(|m| m.is_solid(x, y)))
    }
}

impl Actor {
    /// Assign by stable ID after caller selection. Supported replaceable jobs
    /// are Walking, Digging and Mining; rejected requests preserve inventory and state.
    pub fn assign_basher(&mut self, remaining: &mut u16, context: BashingContext<'_>) -> bool {
        let s = self.state;
        if *remaining == 0
            || s.removed.is_some()
            || !matches!(
                s.action,
                MotionAction::Walking
                    | MotionAction::Digging
                    | MotionAction::Mining
                    | MotionAction::Building
                    | MotionAction::Shrugging
            )
            || context.assignment_blocked(s)
        {
            return false;
        }
        transition(&mut self.state, MotionAction::Bashing, self.rules);
        *remaining -= 1;
        true
    }

    /// Mutable job dispatch. Each actor commits state/pixels only after checked
    /// arithmetic succeeds. Movement frames permit subsequent object checks;
    /// masking/stationary frames do not. See #32 for the reference contract.
    pub fn advance_with_skills(
        &mut self,
        terrain: &mut TerrainMaskMut<'_>,
        context: BashingContext<'_>,
    ) -> Result<MotionOutcome, MotionError> {
        self.require_basic_context()?;
        if self.state.removed.is_some() || self.state.action != MotionAction::Bashing {
            return self.advance_digging(terrain, |x, y| context.is_steel(x, y));
        }
        let mut s = self.state;
        s.end_of_animation = s.frame == 31;
        s.frame = (s.frame + 1) % 32;
        let index = s.frame % 16;
        let moving = (11..=15).contains(&index);
        let mut mask = None;
        if moving {
            s.x =
                s.x.checked_add(s.direction.delta())
                    .ok_or(MotionError::CoordinateOverflow)?;
            if !(0..=1647).contains(&s.x) {
                transition(&mut s, MotionAction::Walking, self.rules);
                s.direction = s.direction.reverse();
            } else {
                let mut dy = 0;
                while dy < 3 && !pixel(terrain.as_read_only(), s.x, i64::from(s.y), dy) {
                    dy += 1;
                    s.y = add_y(s.y, 1)?;
                }
                if dy == 3 {
                    transition(&mut s, MotionAction::Falling, self.rules);
                } else if context.blocked(
                    i64::from(s.x) + i64::from(s.direction.delta()) * 8,
                    i64::from(s.y) - 8,
                    s.direction,
                ) {
                    transition(&mut s, MotionAction::Walking, self.rules);
                    s.direction = s.direction.reverse();
                }
            }
        } else if (2..=5).contains(&index) {
            let left = s.x.checked_sub(8).ok_or(MotionError::CoordinateOverflow)?;
            let top = s.y.checked_sub(10).ok_or(MotionError::CoordinateOverflow)?;
            left.checked_add(15)
                .ok_or(MotionError::CoordinateOverflow)?;
            top.checked_add(9).ok_or(MotionError::CoordinateOverflow)?;
            let frame = usize::from(index - 2)
                + if s.direction == MotionDirection::Left {
                    4
                } else {
                    0
                };
            let bytes = &context.masks[frame * 20..frame * 20 + 20];
            let erased = |x: i64, y: i64| {
                let dx = x - i64::from(left);
                let dy = y - i64::from(top);
                (0..16).contains(&dx)
                    && (0..10).contains(&dy)
                    && mask_bit(bytes, dx as usize, dy as usize)
            };
            if s.frame == 5
                && !(0..4).any(|n| {
                    let x = i64::from(s.x) + i64::from(s.direction.delta()) * (8 + n);
                    let y = i64::from(s.y) - 6;
                    !erased(x, y)
                        && i32::try_from(x).is_ok_and(|x| {
                            i32::try_from(y).is_ok_and(|y| terrain.as_read_only().is_solid(x, y))
                        })
                })
            {
                transition(&mut s, MotionAction::Walking, self.rules);
            }
            mask = Some((left, top, bytes));
        }
        if let Some((left, top, bytes)) = mask {
            for y in 0..10 {
                for x in 0..16 {
                    if mask_bit(bytes, x, y) {
                        terrain.set_solid(left + x as i32, top + y as i32, false);
                    }
                }
            }
        }
        self.state = s;
        Ok(MotionOutcome {
            check_objects: moving,
            ..MotionOutcome::default()
        })
    }
}
fn mask_bit(bytes: &[u8], x: usize, y: usize) -> bool {
    bytes[y * 2 + x / 8] & (128 >> (x % 8)) != 0
}

/// Strict ID order over shared mutable terrain, including both implemented
/// jobs through Bashing. Order errors precede all changes; later failures keep earlier commits.
pub fn step_actors_with_skills(
    actors: &mut [Actor],
    terrain: &mut TerrainMaskMut<'_>,
    context: BashingContext<'_>,
    emit: impl FnMut(ActorState, MotionOutcome),
) -> Result<ActorTick, MotionError> {
    step_mutating(actors, |a| a.advance_with_skills(terrain, context), emit)
}
pub(super) fn step_mutating(
    actors: &mut [Actor],
    mut advance: impl FnMut(&mut Actor) -> Result<MotionOutcome, MotionError>,
    mut emit: impl FnMut(ActorState, MotionOutcome),
) -> Result<ActorTick, MotionError> {
    for (index, pair) in actors.windows(2).enumerate() {
        if pair[0].state.id >= pair[1].state.id {
            return Err(MotionError::ActorOrder { index: index + 1 });
        }
    }
    let mut tick = ActorTick::default();
    for actor in actors {
        if actor.state.removed.is_some() {
            continue;
        }
        let out = advance(actor)?;
        tick.active_before += 1;
        tick.removed += usize::from(out.removal.is_some());
        tick.checking_objects += usize::from(out.check_objects);
        tick.splats_started += usize::from(out.splat_started);
        emit(actor.state, out);
    }
    Ok(tick)
}
