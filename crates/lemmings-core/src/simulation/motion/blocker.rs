use super::*;
use crate::{OBJECT_BLOCKER, OBJECT_FORCE_LEFT, OBJECT_FORCE_RIGHT, ObjectMap, ObjectMapMut};

pub(super) fn field(s: ActorState) -> impl Iterator<Item = (usize, i64, i64)> {
    [-6, -2, 2]
        .into_iter()
        .enumerate()
        .flat_map(move |(row, dy)| {
            [-4, 0, 4].into_iter().enumerate().map(move |(column, dx)| {
                (row * 3 + column, i64::from(s.x) + dx, i64::from(s.y) + dy)
            })
        })
}
impl Actor {
    /// Game.pas AssignBlocker/Transition: test all nine cells before consuming
    /// one skill, save their raw codes, then replace the three columns.
    pub fn assign_blocker(&mut self, remaining: &mut u16, map: &mut ObjectMapMut<'_>) -> bool {
        let s = self.state;
        if *remaining == 0
            || s.removed.is_some()
            || !matches!(
                s.action,
                MotionAction::Walking
                    | MotionAction::Shrugging
                    | MotionAction::Building
                    | MotionAction::Bashing
                    | MotionAction::Mining
                    | MotionAction::Digging
            )
        {
            return false;
        }
        let mut saved = [0; 9];
        for (i, x, y) in field(s) {
            let code = map.as_read_only().read(x, y);
            if matches!(
                code,
                OBJECT_FORCE_LEFT | OBJECT_BLOCKER | OBJECT_FORCE_RIGHT
            ) {
                return false;
            }
            saved[i] = code;
        }
        transition(&mut self.state, MotionAction::Blocking, self.rules);
        self.state.blocker_saved = Some(saved);
        for (i, x, y) in field(s) {
            map.write(
                x,
                y,
                [OBJECT_FORCE_LEFT, OBJECT_BLOCKER, OBJECT_FORCE_RIGHT][i % 3],
            );
        }
        *remaining -= 1;
        true
    }
    /// Apply exit, water, fire and force effects. The caller gates this with the action's
    /// check_objects result, exactly after HandleLemming in CheckLemmings.
    pub(super) fn apply_force(&mut self, map: ObjectMap<'_>) {
        let s = &mut self.state;
        if s.removed.is_some() {
            return;
        }
        let code = map.read(i64::from(s.x), i64::from(s.y));
        s.object_below = code;
        s.object_in_front = map.read(
            i64::from(s.x) + i64::from(s.direction.delta()) * 8,
            i64::from(s.y) - 8,
        );
        // CheckForInteractiveObjects excludes only Falling from DOM_EXIT.
        // A landing splat may enter with direction zero; Floating is allowed.
        if code == crate::OBJECT_EXIT && s.action != MotionAction::Falling {
            transition(s, MotionAction::Exiting, self.rules);
            return;
        }
        if s.y < 160 && (code == crate::OBJECT_WATER || code == crate::OBJECT_FIRE) {
            transition(
                s,
                if code == crate::OBJECT_WATER {
                    MotionAction::Drowning
                } else {
                    MotionAction::Vaporizing
                },
                self.rules,
            );
            return;
        }
        if (code == OBJECT_FORCE_LEFT && s.direction == MotionDirection::Right)
            || (code == OBJECT_FORCE_RIGHT && s.direction == MotionDirection::Left)
        {
            s.direction = s.direction.reverse();
            s.walking_once = false;
        }
    }
    /// All current jobs plus Blocker. Restrictions use the same mutable object
    /// map that Blocker fields overwrite, and earlier actors' restores are seen
    /// by later actors. Unsupported interactive effects are not executed here.
    pub fn advance_with_objects(
        &mut self,
        terrain: &mut TerrainMaskMut<'_>,
        context: MiningContext<'_>,
        map: &mut ObjectMapMut<'_>,
    ) -> Result<MotionOutcome, MotionError> {
        self.require_basic_context()?;
        if self.state.removed.is_some() {
            return Ok(MotionOutcome::default());
        }
        if self.state.action == MotionAction::Blocking {
            let mut s = self.state;
            s.end_of_animation = s.frame == 15;
            s.frame = (s.frame + 1) % 16;
            if !pixel(terrain.as_read_only(), s.x, i64::from(s.y), 0) {
                let saved = s.blocker_saved.expect("validated blocker has saved map");
                for (i, x, y) in field(s) {
                    map.write(x, y, saved[i]);
                }
                transition(&mut s, MotionAction::Walking, self.rules);
            }
            self.state = s;
            return Ok(MotionOutcome::default());
        }
        let objects = BashingContext {
            object_map: Some(map.as_read_only()),
            ..context.objects
        };
        let out = self.advance_with_jobs(terrain, MiningContext { objects, ..context })?;
        if out.check_objects {
            self.apply_force(map.as_read_only());
        }
        Ok(out)
    }
}

pub fn step_actors_with_objects(
    actors: &mut [Actor],
    terrain: &mut TerrainMaskMut<'_>,
    context: MiningContext<'_>,
    map: &mut ObjectMapMut<'_>,
    emit: impl FnMut(ActorState, MotionOutcome),
) -> Result<ActorTick, MotionError> {
    super::basher::step_mutating(
        actors,
        |a| a.advance_with_objects(terrain, context, map),
        emit,
    )
}
