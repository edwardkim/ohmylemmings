use super::SpawnRequest;
use crate::{TerrainMask, TerrainMaskMut};
mod basher;
mod blocker;
pub use blocker::step_actors_with_objects;
mod bomber;
mod builder;
mod climber;
mod floater;
mod hazards;
pub use bomber::{BomberContext, step_actors_with_bomber};
pub use hazards::{TrapState, TrapStates, advance_traps, step_actors_with_hazards};
mod digger;
mod miner;
pub use basher::{BashingContext, step_actors_with_skills};
use core::fmt;
pub use digger::step_actors_mut;
pub use miner::{MiningContext, step_actors_with_jobs};

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

/// Implemented actions, independent of sprite/resource enums.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MotionAction {
    Walking,
    Jumping,
    Falling,
    Splatting,
    Digging,
    Bashing,
    Mining,
    Building,
    Shrugging,
    Blocking,
    OhNo,
    Exploding,
    Climbing,
    Hoisting,
    Floating,
    Exiting,
    Drowning,
    Vaporizing,
}
impl MotionAction {
    pub const fn frame_count(self) -> u8 {
        match self {
            Self::Walking
            | Self::Shrugging
            | Self::Climbing
            | Self::Hoisting
            | Self::Floating
            | Self::Exiting => 8,
            Self::Jumping => 1,
            Self::Falling => 4,
            Self::Splatting
            | Self::Digging
            | Self::Building
            | Self::Blocking
            | Self::OhNo
            | Self::Drowning => 16,
            Self::Vaporizing => 14,
            Self::Exploding => 1,
            Self::Bashing => 32,
            Self::Mining => 24,
        }
    }
    fn loops(self) -> bool {
        matches!(
            self,
            Self::Walking | Self::Falling | Self::Climbing | Self::Floating
        )
    }
    fn foot_y(self) -> i32 {
        match self {
            Self::Building | Self::Mining | Self::Exiting => 13,
            Self::Digging | Self::Climbing | Self::Hoisting => 12,
            Self::Exploding => 25,
            Self::Floating => 16,
            Self::Vaporizing => 14,
            _ => 10,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RemovalReason {
    BottomBoundary,
    SplattingFinished,
    Exploded,
    Rescued,
    Drowned,
    Vaporized,
    Trapped,
}

/// Stable actor record. Coordinates are the foot, not sprite top-left. Removed
/// records retain their last action/position; explosion particle counters may advance.
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
    pub bricks_left: u8,
    pub blocker_saved: Option<[u8; 9]>,
    pub is_climber: bool,
    pub is_floater: bool,
    /// Original16-stage table cursor; retained after leaving Floating.
    pub float_index: u8,
    /// DOS AssignClimber while shrugging changes action, preserving Once animation.
    pub walking_once: bool,
    pub explosion_timer: u8,
    pub particle_timer: u8,
    pub particle_frame: u8,
    pub object_below: u8,
    pub object_in_front: u8,
    pub removed: Option<RemovalReason>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MotionRules {
    pub faller_starts_with_three: bool,
    pub assign_climber_shrugger_bug: bool,
}
impl Default for MotionRules {
    fn default() -> Self {
        Self {
            faller_starts_with_three: true,
            assign_climber_shrugger_bug: true,
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
    InvalidBricks,
    ObjectMapRequired,
    InvalidBlockerState,
    InvalidBomberState,
    InvalidClimberState,
    InvalidFloaterState,
    BomberContextRequired,
    InvalidTrapFrames,
    MissingTrap(u8),
}
impl fmt::Display for MotionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MutableTerrainRequired => {
                f.write_str("terrain-changing jobs require mutable terrain")
            }
            Self::InvalidFloaterState => f.write_str("invalid floater ability or table index"),
            Self::InvalidClimberState => {
                f.write_str("climbing/hoisting or stale walking animation requires climber ability")
            }
            Self::InvalidBomberState => f.write_str("invalid bomber timer or particle state"),
            Self::BomberContextRequired => {
                f.write_str("bomber progression requires bomber context")
            }
            Self::InvalidNewDigger => f.write_str("new digger must be digging at frame zero"),
            Self::ObjectMapRequired => f.write_str("blocking requires its mutable object map"),
            Self::InvalidBlockerState => f.write_str(
                "saved field requires Blocking, OhNo or Exploding; Blocking needs a saved field",
            ),
            Self::InvalidTrapFrames => {
                f.write_str("trap frame count must be nonzero and first frame in range")
            }
            Self::MissingTrap(index) => write!(f, "object map references missing DOS trap {index}"),
            Self::InvalidBricks => f.write_str("only building retains one to twelve bricks"),
            Self::Frame { action, frame } => write!(f, "frame {frame} is outside {action:?}"),
            Self::Direction => f.write_str("stopped direction is invalid for a moving action"),
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
    /// Actual empty-to-solid changes from a builder, in left-to-right order.
    /// Hosts may use these coordinates to maintain terrain material/color.
    pub added_pixels: [Option<(i32, i32)>; 6],
    /// Actual explosion solid-to-empty changes:22 rows, bit0 is the left pixel.
    pub explosion_erased: [u16; 22],
}

/// Caller-owned actor. No allocation, rendering or I/O. Trigger execution is
/// separate; supported skills accept caller-provided terrain/object views.
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
                bricks_left: 0,
                blocker_saved: None,
                is_climber: false,
                is_floater: false,
                float_index: 0,
                walking_once: false,
                explosion_timer: 0,
                particle_timer: 0,
                particle_frame: 0,
                object_below: 0,
                object_in_front: 0,
                removed: None,
            },
            rules,
        )
    }
    /// Restore a basic state with valid frame/direction/counter/removal fields.
    /// Arbitrary i32 coordinates are accepted; later arithmetic errors are
    /// explicit and leave the complete record unchanged.
    pub fn from_state(state: ActorState, rules: MotionRules) -> Result<Self, MotionError> {
        if state.float_index > 15
            || (!state.is_floater && state.float_index != 0)
            || (state.action == MotionAction::Floating && !state.is_floater)
        {
            return Err(MotionError::InvalidFloaterState);
        }
        if (matches!(
            state.action,
            MotionAction::Climbing | MotionAction::Hoisting
        ) && !state.is_climber)
            || (state.walking_once && (!state.is_climber || state.action != MotionAction::Walking))
        {
            return Err(MotionError::InvalidClimberState);
        }
        if (state.action == MotionAction::Blocking && state.blocker_saved.is_none())
            || (state.blocker_saved.is_some()
                && !matches!(
                    state.action,
                    MotionAction::Blocking
                        | MotionAction::OhNo
                        | MotionAction::Exploding
                        | MotionAction::Exiting
                        | MotionAction::Drowning
                        | MotionAction::Vaporizing
                ))
        {
            return Err(MotionError::InvalidBlockerState);
        }
        if state.explosion_timer > 79
            || state.particle_timer > 52
            || state.particle_frame > 50
            // Nuke can re-arm OhNo; its timer persists into Exploding/removal.
            || (state.action == MotionAction::Splatting && state.explosion_timer != 0)
            || ((state.particle_timer != 0 || state.particle_frame != 0)
                && state.removed != Some(RemovalReason::Exploded))
        {
            return Err(MotionError::InvalidBomberState);
        }
        if (state.action == MotionAction::Building && !(1..=12).contains(&state.bricks_left))
            || (state.action != MotionAction::Building && state.bricks_left != 0)
        {
            return Err(MotionError::InvalidBricks);
        }
        if state.new_digger && (state.action != MotionAction::Digging || state.frame != 0) {
            return Err(MotionError::InvalidNewDigger);
        }
        if state.frame >= state.action.frame_count() {
            return Err(MotionError::Frame {
                action: state.action,
                frame: state.frame,
            });
        }
        if (state.action == MotionAction::Splatting && state.direction != MotionDirection::Stopped)
            || (state.direction == MotionDirection::Stopped
                && !matches!(
                    state.action,
                    MotionAction::Splatting
                        | MotionAction::Exiting
                        | MotionAction::Drowning
                        | MotionAction::Vaporizing
                        | MotionAction::OhNo
                        | MotionAction::Exploding
                ))
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
                        MotionAction::Walking
                            | MotionAction::Falling
                            | MotionAction::Digging
                            | MotionAction::Mining
                            | MotionAction::OhNo
                            | MotionAction::Floating
                    )
            }
            Some(RemovalReason::Exploded) => {
                state.action == MotionAction::Exploding
                    && state.frame == 0
                    && state.end_of_animation
                    && state.blocker_saved.is_none()
            }
            Some(RemovalReason::SplattingFinished) => {
                state.action == MotionAction::Splatting
                    && state.frame == 15
                    && state.end_of_animation
            }
            Some(RemovalReason::Drowned) => {
                state.action == MotionAction::Drowning
                    && state.frame == 15
                    && state.end_of_animation
            }
            Some(RemovalReason::Vaporized) => {
                state.action == MotionAction::Vaporizing
                    && state.frame == 13
                    && state.end_of_animation
            }
            Some(RemovalReason::Trapped) => {
                state.object_below < 16
                    && !matches!(
                        state.action,
                        MotionAction::Exiting
                            | MotionAction::Drowning
                            | MotionAction::Vaporizing
                            | MotionAction::Exploding
                    )
            }
            Some(RemovalReason::Rescued) => {
                state.action == MotionAction::Exiting && state.frame == 7 && state.end_of_animation
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
        self.require_basic_context()?;
        let mut s = self.state;
        if s.removed.is_some() {
            return Ok((s, MotionOutcome::default()));
        }
        if s.action != MotionAction::Floating {
            if s.frame < s.action.frame_count() - 1 {
                s.frame += 1;
                s.end_of_animation = false;
            } else {
                s.end_of_animation = true;
                if s.action.loops() && !s.walking_once {
                    s.frame = 0;
                }
            }
        }
        let mut result = MotionOutcome {
            check_objects: true,
            ..MotionOutcome::default()
        };
        match s.action {
            MotionAction::Drowning => {
                result.check_objects = false;
                if s.end_of_animation {
                    remove(&mut s, &mut result, RemovalReason::Drowned);
                } else if !i32::try_from(i64::from(s.x) + 8 * i64::from(s.direction.delta()))
                    .is_ok_and(|x| terrain.is_solid(x, s.y))
                {
                    s.x =
                        s.x.checked_add(s.direction.delta())
                            .ok_or(MotionError::CoordinateOverflow)?;
                }
            }
            MotionAction::Vaporizing => {
                result.check_objects = false;
                if s.end_of_animation {
                    remove(&mut s, &mut result, RemovalReason::Vaporized);
                }
            }
            MotionAction::Exiting => {
                result.check_objects = false;
                if s.end_of_animation {
                    remove(&mut s, &mut result, RemovalReason::Rescued);
                }
            }
            MotionAction::Walking => {
                s.x =
                    s.x.checked_add(s.direction.delta())
                        .ok_or(MotionError::CoordinateOverflow)?;
                if !(0..=1647).contains(&s.x) {
                    s.direction = s.direction.reverse();
                    s.walking_once = false;
                } else if pixel(terrain, s.x, i64::from(s.y), 0) {
                    let (mut dy, mut new_y) = (0, s.y);
                    while dy <= 6 && pixel(terrain, s.x, i64::from(new_y) - 1, -dy - 1) {
                        dy += 1;
                        new_y = add_y(new_y, -1)?;
                    }
                    if dy > 6 {
                        if s.is_climber {
                            transition(&mut s, MotionAction::Climbing, self.rules);
                        } else {
                            s.direction = s.direction.reverse();
                            s.walking_once = false;
                        }
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
            MotionAction::Climbing | MotionAction::Hoisting => {
                climber::advance(&mut s, terrain, self.rules, &mut result)?;
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
            MotionAction::Floating => {
                floater::advance(&mut s, terrain, self.rules, &mut result)?;
            }
            MotionAction::Falling => {
                if s.is_floater && s.fallen > 16 {
                    transition(&mut s, MotionAction::Floating, self.rules);
                    return Ok((s, result));
                }
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
            MotionAction::Digging
            | MotionAction::Bashing
            | MotionAction::Mining
            | MotionAction::Building => {
                return Err(MotionError::MutableTerrainRequired);
            }
            MotionAction::Blocking => return Err(MotionError::ObjectMapRequired),
            MotionAction::OhNo | MotionAction::Exploding => {
                return Err(MotionError::BomberContextRequired);
            }
            MotionAction::Shrugging => {
                result.check_objects = s.end_of_animation;
                if s.end_of_animation {
                    transition(&mut s, MotionAction::Walking, self.rules);
                }
            }
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
    if action == MotionAction::Floating {
        s.float_index = 0;
    }
    s.walking_once = false;
    if !matches!(
        action,
        MotionAction::OhNo
            | MotionAction::Exploding
            | MotionAction::Exiting
            | MotionAction::Drowning
            | MotionAction::Vaporizing
    ) {
        s.blocker_saved = None;
    }
    s.new_digger = action == MotionAction::Digging;
    s.bricks_left = if action == MotionAction::Building {
        12
    } else {
        0
    };
    s.frame = 0;
    s.end_of_animation = false;
    s.fallen = if action == MotionAction::Falling && rules.faller_starts_with_three {
        3
    } else {
        0
    };
    if action == MotionAction::Splatting {
        s.direction = MotionDirection::Stopped;
        s.explosion_timer = 0;
    }
}
fn top(s: &mut ActorState, rules: MotionRules) {
    // Use the action after any transition, matching Game.pas's FrameTopDy.
    let foot = s.action.foot_y();
    if i64::from(s.y) - i64::from(foot) < -5 {
        s.y = foot - 7;
        s.direction = s.direction.reverse();
        s.walking_once = false;
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
    /// Rescues are also removals, emitted only on the Exiting completion tick.
    pub rescued: usize,
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
        tick.rescued += usize::from(outcome.removal == Some(RemovalReason::Rescued));
        tick.checking_objects += usize::from(outcome.check_objects);
        tick.splats_started += usize::from(outcome.splat_started);
        emit(actor.state, outcome);
    }
    Ok(tick)
}

#[cfg(test)]
mod tests;
