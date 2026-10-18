use super::SpawnRequest;
use crate::{TerrainMask, TerrainMaskMut};
mod digger;
use core::fmt;
pub use digger::step_actors_mut;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MotionDirection {
    Left,
    Right,
    Stopped,
}
impl MotionDirection {
    pub const fn delta(self) -> i32 {
        match self {
            Self::Left => -1,
            Self::Right => 1,
            Self::Stopped => 0,
        }
    }
    fn reverse(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Stopped => Self::Stopped,
        }
    }
}

/// Basic unskilled actions, independent of sprite/resource enums.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MotionAction {
    Walking,
    Jumping,
    Falling,
    Splatting,
    Digging,
}
impl MotionAction {
    pub const fn frame_count(self) -> u8 {
        match self {
            Self::Walking => 8,
            Self::Jumping => 1,
            Self::Falling => 4,
            Self::Splatting | Self::Digging => 16,
        }
    }
    fn loops(self) -> bool {
        matches!(self, Self::Walking | Self::Falling)
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RemovalReason {
    BottomBoundary,
    SplattingFinished,
}

/// Stable actor record. Coordinates are the foot, not sprite top-left. Removed
/// records retain their last action/position and are never updated again.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActorState {
    pub id: u32,
    pub born_iteration: u64,
    pub x: i32,
    pub y: i32,
    pub direction: MotionDirection,
    pub action: MotionAction,
    pub frame: u8,
    pub end_of_animation: bool,
    pub fallen: u32,
    pub new_digger: bool,
    pub removed: Option<RemovalReason>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MotionRules {
    pub faller_starts_with_three: bool,
}
impl Default for MotionRules {
    fn default() -> Self {
        Self {
            faller_starts_with_three: true,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EntrancePoint {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MotionError {
    Frame { action: MotionAction, frame: u8 },
    Direction,
    InvalidFallen,
    InvalidRemoval,
    EntranceCount(usize),
    EntranceIndex(u8),
    CoordinateOverflow,
    FallCounterOverflow,
    ActorOrder { index: usize },
    MutableTerrainRequired,
    InvalidNewDigger,
}
impl fmt::Display for MotionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MutableTerrainRequired => f.write_str("digging requires mutable terrain"),
            Self::InvalidNewDigger => f.write_str("new digger must be digging at frame zero"),
            Self::Frame { action, frame } => write!(f, "frame {frame} is outside {action:?}"),
            Self::Direction => f.write_str("stopped direction must match splatting"),
            Self::InvalidFallen => f.write_str("only falling may retain a fall counter"),
            Self::InvalidRemoval => f.write_str("removal reason is inconsistent with actor state"),
            Self::EntranceCount(n) => write!(f, "entrance count {n} exceeds 4"),
            Self::EntranceIndex(i) => write!(f, "entrance index {i} is absent"),
            Self::CoordinateOverflow => f.write_str("actor coordinate overflow"),
            Self::FallCounterOverflow => f.write_str("actor fall counter overflow"),
            Self::ActorOrder { index } => {
                write!(f, "actor at index {index} is not in strict ID order")
            }
        }
    }
}
impl core::error::Error for MotionError {}

/// A new splat still permits a later object check (direct-drop exit ordering).
/// Removal is delayed until the splat animation finishes. Object behavior itself
/// belongs to a later gameplay phase.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MotionOutcome {
    pub check_objects: bool,
    pub removal: Option<RemovalReason>,
    pub splat_started: bool,
}

/// Caller-owned basic actor. No allocation, rendering, I/O, skills or triggers.
///
/// ```
/// use lemmings_core::{Actor, EntrancePoint, MotionRules, SpawnRequest, TerrainMask};
/// let request = SpawnRequest { actor_id: 7, entrance_index: 0, iteration: 54 };
/// let mut actor = Actor::from_spawn(request, &[EntrancePoint { x: 0, y: 0 }], MotionRules::default())?;
/// let cells = [0; 32 * 32];
/// let terrain = TerrainMask::new(32, 32, &cells).unwrap();
/// actor.advance(terrain)?;
/// assert_eq!((actor.state().x, actor.state().y, actor.state().fallen), (24, 17, 6));
/// # Ok::<(), lemmings_core::MotionError>(())
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Actor {
    state: ActorState,
    rules: MotionRules,
}
impl Actor {
    /// Create a right-facing faller at entrance+(24,14). The caller retains the
    /// original entrance order and must update this actor on the birth tick.
    pub fn from_spawn(
        request: SpawnRequest,
        entrances: &[EntrancePoint],
        rules: MotionRules,
    ) -> Result<Self, MotionError> {
        if entrances.len() > 4 {
            return Err(MotionError::EntranceCount(entrances.len()));
        }
        let p = entrances
            .get(usize::from(request.entrance_index))
            .ok_or(MotionError::EntranceIndex(request.entrance_index))?;
        Self::from_state(
            ActorState {
                id: request.actor_id,
                born_iteration: request.iteration,
                x: p.x.checked_add(24).ok_or(MotionError::CoordinateOverflow)?,
                y: p.y.checked_add(14).ok_or(MotionError::CoordinateOverflow)?,
                direction: MotionDirection::Right,
                action: MotionAction::Falling,
                frame: 0,
                end_of_animation: false,
                fallen: if rules.faller_starts_with_three { 3 } else { 0 },
                new_digger: false,
                removed: None,
            },
            rules,
        )
    }
    /// Restore a basic state with valid frame/direction/counter/removal fields.
    /// Arbitrary i32 coordinates are accepted; later arithmetic errors are
    /// explicit and leave the complete record unchanged.
    pub fn from_state(state: ActorState, rules: MotionRules) -> Result<Self, MotionError> {
        if state.new_digger && (state.action != MotionAction::Digging || state.frame != 0) {
            return Err(MotionError::InvalidNewDigger);
        }
        if state.frame >= state.action.frame_count() {
            return Err(MotionError::Frame {
                action: state.action,
                frame: state.frame,
            });
        }
        if (state.direction == MotionDirection::Stopped)
            != (state.action == MotionAction::Splatting)
        {
            return Err(MotionError::Direction);
        }
        if state.action != MotionAction::Falling && state.fallen != 0 {
            return Err(MotionError::InvalidFallen);
        }
        let valid_removal = match state.removed {
            None => true,
            Some(RemovalReason::BottomBoundary) => {
                state.y > 163
                    && matches!(
                        state.action,
                        MotionAction::Walking | MotionAction::Falling | MotionAction::Digging
                    )
            }
            Some(RemovalReason::SplattingFinished) => {
                state.action == MotionAction::Splatting
                    && state.frame == 15
                    && state.end_of_animation
            }
        };
        if !valid_removal {
            return Err(MotionError::InvalidRemoval);
        }
        Ok(Self { state, rules })
    }
    pub const fn state(&self) -> ActorState {
        self.state
    }
    pub const fn rules(&self) -> MotionRules {
        self.rules
    }

    /// Advance one logical tick. Terrain and actor state are preserved on error.
    /// An already removed actor is a no-op, with no repeated removal result.
    pub fn advance(&mut self, terrain: TerrainMask<'_>) -> Result<MotionOutcome, MotionError> {
        let (state, result) = self.preview(terrain)?;
        self.state = state;
        Ok(result)
    }
    fn preview(
        &self,
        terrain: TerrainMask<'_>,
    ) -> Result<(ActorState, MotionOutcome), MotionError> {
        let mut s = self.state;
        if s.removed.is_some() {
            return Ok((s, MotionOutcome::default()));
        }
        if s.frame < s.action.frame_count() - 1 {
            s.frame += 1;
            s.end_of_animation = false;
        } else {
            s.end_of_animation = true;
            if s.action.loops() {
                s.frame = 0;
            }
        }
        let mut result = MotionOutcome {
            check_objects: true,
            ..MotionOutcome::default()
        };
        match s.action {
            MotionAction::Walking => {
                s.x =
                    s.x.checked_add(s.direction.delta())
                        .ok_or(MotionError::CoordinateOverflow)?;
                if !(0..=1647).contains(&s.x) {
                    s.direction = s.direction.reverse();
                } else if pixel(terrain, s.x, i64::from(s.y), 0) {
                    let (mut dy, mut new_y) = (0, s.y);
                    while dy <= 6 && pixel(terrain, s.x, i64::from(new_y) - 1, -dy - 1) {
                        dy += 1;
                        new_y = add_y(new_y, -1)?;
                    }
                    if dy > 6 {
                        s.direction = s.direction.reverse();
                    } else {
                        if dy >= 3 {
                            transition(&mut s, MotionAction::Jumping, self.rules);
                            new_y = add_y(s.y, -2)?;
                        }
                        s.y = new_y;
                        top(&mut s, self.rules);
                    }
                } else {
                    let mut dy = 1;
                    while dy <= 3 {
                        s.y = add_y(s.y, 1)?;
                        if pixel(terrain, s.x, i64::from(s.y), dy) {
                            break;
                        }
                        dy += 1;
                    }
                    if dy > 3 {
                        s.y = add_y(s.y, 1)?;
                        transition(&mut s, MotionAction::Falling, self.rules);
                    }
                    if s.y > 163 {
                        remove(&mut s, &mut result, RemovalReason::BottomBoundary);
                    }
                }
            }
            MotionAction::Jumping => {
                let mut dy = 0;
                while dy < 2 && pixel(terrain, s.x, i64::from(s.y) - 1, -dy - 1) {
                    dy += 1;
                    s.y = add_y(s.y, -1)?;
                }
                if dy < 2 {
                    transition(&mut s, MotionAction::Walking, self.rules);
                }
                top(&mut s, self.rules);
            }
            MotionAction::Falling => {
                let mut dy = 0;
                while dy < 3 && !pixel(terrain, s.x, i64::from(s.y), dy) {
                    dy += 1;
                    s.y = add_y(s.y, 1)?;
                    if s.y > 163 {
                        remove(&mut s, &mut result, RemovalReason::BottomBoundary);
                        return Ok((s, result));
                    }
                }
                if dy == 3 {
                    s.fallen = s
                        .fallen
                        .checked_add(3)
                        .ok_or(MotionError::FallCounterOverflow)?;
                } else if s.fallen > 60 {
                    transition(&mut s, MotionAction::Splatting, self.rules);
                    result.splat_started = true;
                } else {
                    transition(&mut s, MotionAction::Walking, self.rules);
                }
            }
            MotionAction::Digging => return Err(MotionError::MutableTerrainRequired),
            MotionAction::Splatting => {
                result.check_objects = false;
                if s.end_of_animation {
                    remove(&mut s, &mut result, RemovalReason::SplattingFinished);
                }
            }
        }
        Ok((s, result))
    }
}
fn pixel(terrain: TerrainMask<'_>, x: i32, y: i64, minimum: i32) -> bool {
    i32::try_from(y.max(i64::from(minimum))).is_ok_and(|y| terrain.is_solid(x, y))
}
fn add_y(y: i32, delta: i32) -> Result<i32, MotionError> {
    y.checked_add(delta).ok_or(MotionError::CoordinateOverflow)
}
fn transition(s: &mut ActorState, action: MotionAction, rules: MotionRules) {
    if s.action == action {
        return;
    }
    s.action = action;
    s.new_digger = action == MotionAction::Digging;
    s.frame = 0;
    s.end_of_animation = false;
    s.fallen = if action == MotionAction::Falling && rules.faller_starts_with_three {
        3
    } else {
        0
    };
    if action == MotionAction::Splatting {
        s.direction = MotionDirection::Stopped;
    }
}
fn top(s: &mut ActorState, rules: MotionRules) {
    // All four supported animations have FootY=10. HEAD_MIN_Y=-5, so correction
    // is -5-2+10=3. Preserve the reference's turn and jumping-to-walking order.
    if s.y < 5 {
        s.y = 3;
        s.direction = s.direction.reverse();
        if s.action == MotionAction::Jumping {
            transition(s, MotionAction::Walking, rules);
        }
    }
}
fn remove(s: &mut ActorState, out: &mut MotionOutcome, reason: RemovalReason) {
    s.removed = Some(reason);
    out.removal = Some(reason);
    out.check_objects = false;
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ActorTick {
    pub active_before: usize,
    pub removed: usize,
    pub checking_objects: usize,
    pub splats_started: usize,
}

/// Advance a caller-owned, strictly ID-ordered slice over immutable terrain.
/// Two passes validate every next state before committing or emitting anything;
/// duplicate/reversed IDs or arithmetic errors preserve the entire slice.
/// Removed records stay in place and produce no callback. This helper does not
/// process object/skill interactions. Callback panics do not roll back effects.
///
/// ```
/// use lemmings_core::{Actor, ActorTick, EntrancePoint, MotionRules, SpawnRequest,
///                     TerrainMask, step_actors};
/// let request = SpawnRequest { actor_id: 0, entrance_index: 0, iteration: 54 };
/// let mut actors = [Actor::from_spawn(request, &[EntrancePoint { x: 0, y: 0 }], MotionRules::default())?];
/// let cells = [0; 32 * 32];
/// let terrain = TerrainMask::new(32, 32, &cells).unwrap();
/// let tick = step_actors(&mut actors, terrain, |_, _| {})?;
/// assert_eq!(tick.active_before, 1);
/// assert_eq!(actors[0].state().y, 17);
/// # Ok::<(), lemmings_core::MotionError>(())
/// ```
pub fn step_actors(
    actors: &mut [Actor],
    terrain: TerrainMask<'_>,
    mut emit: impl FnMut(ActorState, MotionOutcome),
) -> Result<ActorTick, MotionError> {
    for (i, pair) in actors.windows(2).enumerate() {
        if pair[0].state.id >= pair[1].state.id {
            return Err(MotionError::ActorOrder { index: i + 1 });
        }
    }
    for actor in actors.iter() {
        actor.preview(terrain)?;
    }
    let mut tick = ActorTick::default();
    for actor in actors {
        if actor.state.removed.is_some() {
            continue;
        }
        let outcome = actor
            .advance(terrain)
            .expect("validated immutable motion preflight");
        tick.active_before += 1;
        tick.removed += usize::from(outcome.removal.is_some());
        tick.checking_objects += usize::from(outcome.check_objects);
        tick.splats_started += usize::from(outcome.splat_started);
        emit(actor.state, outcome);
    }
    Ok(tick)
}

#[cfg(test)]
mod tests;
