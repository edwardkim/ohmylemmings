use super::*;

impl Actor {
    /// Assign to a live walker, basher or miner, consuming one skill only on success. The caller
    /// selects a stable actor ID; cursor priority and other jobs are separate.
    /// Steel is a caller-provided object-map view, not per-pixel protection.
    pub fn assign_digger(&mut self, remaining: &mut u16, steel: TerrainMask<'_>) -> bool {
        if *remaining == 0
            || self.state.removed.is_some()
            || !matches!(
                self.state.action,
                MotionAction::Walking
                    | MotionAction::Bashing
                    | MotionAction::Mining
                    | MotionAction::Building
                    | MotionAction::Shrugging
            )
            || steel.is_solid(self.state.x, self.state.y)
        {
            return false;
        }
        transition(&mut self.state, MotionAction::Digging, self.rules);
        *remaining -= 1;
        true
    }

    /// One tick for basic motion/Digging; use `advance_with_jobs` for all jobs.
    /// Each actor's arithmetic is checked before
    /// committing its state or pixels. Earlier actors in a batch see/leave their
    /// edits in ID order. Steel is checked at the new foot after a supported row
    /// is removed; holes take precedence over steel. See #31's reference contract.
    pub fn advance_mut(
        &mut self,
        terrain: &mut TerrainMaskMut<'_>,
        steel: TerrainMask<'_>,
    ) -> Result<MotionOutcome, MotionError> {
        if self.state.removed.is_some() {
            return Ok(MotionOutcome::default());
        }
        if self.state.action != MotionAction::Digging {
            return self.advance(terrain.as_read_only());
        }
        let mut s = self.state;
        let left = s.x.checked_sub(4).ok_or(MotionError::CoordinateOverflow)?;
        s.x.checked_add(4).ok_or(MotionError::CoordinateOverflow)?;
        // At most three rows of nine cells. Plan before applying; no allocation.
        let mut deleted = [(0, 0); 27];
        let mut count = 0;
        let mut row = |y: i32| {
            let mut support = false;
            for x in left..=left + 8 {
                let point = (x, y.max(0));
                if terrain.as_read_only().is_solid(point.0, point.1)
                    && !deleted[..count].contains(&point)
                {
                    support = true;
                    deleted[count] = point;
                    count += 1;
                }
            }
            support
        };
        if s.new_digger {
            row(s.y.checked_sub(2).ok_or(MotionError::CoordinateOverflow)?);
            row(s.y.checked_sub(1).ok_or(MotionError::CoordinateOverflow)?);
            s.new_digger = false;
        } else {
            s.frame = (s.frame + 1) % 16;
        }
        let mut out = MotionOutcome::default();
        if s.frame == 0 || s.frame == 8 {
            let y = s.y;
            s.y = add_y(s.y, 1)?;
            if s.y > 163 {
                remove(&mut s, &mut out, RemovalReason::BottomBoundary);
            } else {
                if !row(y) {
                    transition(&mut s, MotionAction::Falling, self.rules);
                } else if steel.is_solid(s.x, s.y) {
                    transition(&mut s, MotionAction::Walking, self.rules);
                }
                out.check_objects = true;
            }
        }
        for &(x, y) in &deleted[..count] {
            terrain.set_solid(x, y, false);
        }
        self.state = s;
        Ok(out)
    }
}

/// Update a strict ID-ordered batch over changing terrain. Order errors are
/// rejected before any edit/callback. A later actor's arithmetic error preserves
/// that actor and its pixels; earlier successful updates remain committed.
/// Use `step_actors` for the legacy immutable batch's all-or-nothing preflight.
pub fn step_actors_mut(
    actors: &mut [Actor],
    terrain: &mut TerrainMaskMut<'_>,
    steel: TerrainMask<'_>,
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
        let out = actor.advance_mut(terrain, steel)?;
        tick.active_before += 1;
        tick.removed += usize::from(out.removal.is_some());
        tick.checking_objects += usize::from(out.check_objects);
        tick.splats_started += usize::from(out.splat_started);
        emit(actor.state, out);
    }
    Ok(tick)
}
