use super::*;
use crate::ObjectMapMut;

/// Existing jobs plus a caller-owned, 16x22 MSB-first explosion erase mask.
#[derive(Clone, Copy, Debug)]
pub struct BomberContext<'a> {
    pub motion: MiningContext<'a>,
    pub explosion_mask: &'a [u8; 44],
}
impl<'a> BomberContext<'a> {
    pub const fn new(motion: MiningContext<'a>, explosion_mask: &'a [u8; 44]) -> Self {
        Self {
            motion,
            explosion_mask,
        }
    }
}
impl ActorState {
    /// Display values follow the original 79-tick counter, independently of wall time.
    pub const fn countdown_digit(self) -> Option<u8> {
        match self.explosion_timer {
            0 => None,
            1..=16 => Some(1),
            17..=32 => Some(2),
            33..=48 => Some(3),
            49..=64 => Some(4),
            _ => Some(5),
        }
    }
}
impl Actor {
    // CheckUpdateNuking bypasses inventory/AssignBomber's OhNo restriction.
    pub(crate) fn arm_nuke(&mut self) {
        self.state.explosion_timer = 79;
    }
    pub(super) fn require_basic_context(&self) -> Result<(), MotionError> {
        if self.state.particle_timer != 0
            || (self.state.removed.is_none()
                && (self.state.explosion_timer != 0
                    || matches!(
                        self.state.action,
                        MotionAction::OhNo | MotionAction::Exploding
                    )))
        {
            Err(MotionError::BomberContextRequired)
        } else {
            Ok(())
        }
    }
    /// Assignment changes only the timer and successful inventory count.
    pub fn assign_bomber(&mut self, remaining: &mut u16) -> bool {
        if *remaining == 0
            || self.state.removed.is_some()
            || self.state.explosion_timer != 0
            || matches!(
                self.state.action,
                MotionAction::Splatting | MotionAction::OhNo | MotionAction::Exploding
            )
        {
            return false;
        }
        self.state.explosion_timer = 79;
        *remaining -= 1;
        true
    }
    /// CheckLemmings order: particles, removed guard, countdown (expiry skips
    /// motion), action, then interactive objects. Errors preserve this actor.
    pub fn advance_with_bomber(
        &mut self,
        terrain: &mut TerrainMaskMut<'_>,
        context: BomberContext<'_>,
        map: &mut ObjectMapMut<'_>,
    ) -> Result<MotionOutcome, MotionError> {
        let mut next = *self;
        if next.state.particle_timer > 0 {
            next.state.particle_timer -= 1;
            next.state.particle_frame = (next.state.particle_frame + 1).min(50);
        }
        if next.state.removed.is_some() {
            *self = next;
            return Ok(MotionOutcome::default());
        }
        if next.state.explosion_timer > 0 {
            next.state.explosion_timer -= 1;
            if next.state.explosion_timer == 0 {
                let action = if matches!(
                    next.state.action,
                    MotionAction::Falling | MotionAction::Floating
                ) {
                    MotionAction::Exploding
                } else {
                    MotionAction::OhNo
                };
                transition(&mut next.state, action, next.rules);
                *self = next;
                return Ok(MotionOutcome::default());
            }
        }
        let mut out = MotionOutcome::default();
        match next.state.action {
            MotionAction::OhNo => {
                let s = &mut next.state;
                s.end_of_animation = s.frame == 15;
                if s.end_of_animation {
                    transition(s, MotionAction::Exploding, next.rules);
                } else {
                    s.frame += 1;
                    let mut dy = 0;
                    while dy < 3 && !pixel(terrain.as_read_only(), s.x, i64::from(s.y), dy) {
                        dy += 1;
                        s.y = add_y(s.y, 1)?;
                    }
                    if s.y > 163 {
                        remove(s, &mut out, RemovalReason::BottomBoundary);
                    } else {
                        out.check_objects = true;
                        next.apply_force(map.as_read_only());
                    }
                }
            }
            MotionAction::Exploding => {
                let s = &mut next.state;
                s.end_of_animation = true;
                if let Some(saved) = s.blocker_saved.take() {
                    for (i, x, y) in super::blocker::field(*s) {
                        map.write(x, y, saved[i]);
                    }
                }
                // Object gate is read after restoring a Blocker's original map.
                if !matches!(
                    map.as_read_only().read(i64::from(s.x), i64::from(s.y)),
                    133 | 137
                ) {
                    for y_index in 0..22 {
                        for x_index in 0..16 {
                            let (x, y) = (x_index, y_index);
                            if context.explosion_mask[y * 2 + x / 8] & (128 >> (x % 8)) != 0 {
                                let dx = i64::from(s.x) - 8 + x as i64;
                                let dy = i64::from(s.y) - 14 + y as i64;
                                if let (Ok(x), Ok(y)) = (i32::try_from(dx), i32::try_from(dy))
                                    && terrain.set_solid(x, y, false) == Some(true)
                                {
                                    out.explosion_erased[y_index] |= 1 << x_index;
                                }
                            }
                        }
                    }
                }
                remove(s, &mut out, RemovalReason::Exploded);
                s.particle_timer = 52;
                s.particle_frame = 0;
            }
            _ => {
                // The legacy job coordinator owns animation/terrain changes;
                // pass its timer-free view, then restore the countdown unless
                // that action entered Splatting (which cancels the timer).
                let timer = next.state.explosion_timer;
                next.state.explosion_timer = 0;
                out = next.advance_with_objects(terrain, context.motion, map)?;
                if !out.splat_started && next.state.action != MotionAction::Splatting {
                    next.state.explosion_timer = timer;
                }
            }
        }
        *self = next;
        Ok(out)
    }
}
/// All records, including removed explosion particles, progress in strict ID order.
pub fn step_actors_with_bomber(
    actors: &mut [Actor],
    terrain: &mut TerrainMaskMut<'_>,
    context: BomberContext<'_>,
    map: &mut ObjectMapMut<'_>,
    mut emit: impl FnMut(ActorState, MotionOutcome),
) -> Result<ActorTick, MotionError> {
    for (index, pair) in actors.windows(2).enumerate() {
        if pair[0].state.id >= pair[1].state.id {
            return Err(MotionError::ActorOrder { index: index + 1 });
        }
    }
    let mut tick = ActorTick::default();
    for actor in actors {
        let removed = actor.state.removed.is_some();
        let out = actor.advance_with_bomber(terrain, context, map)?;
        if removed {
            continue;
        }
        tick.active_before += 1;
        tick.removed += usize::from(out.removal.is_some());
        tick.rescued += usize::from(out.removal == Some(RemovalReason::Rescued));
        tick.checking_objects += usize::from(out.check_objects);
        tick.splats_started += usize::from(out.splat_started);
        emit(actor.state, out);
    }
    Ok(tick)
}
