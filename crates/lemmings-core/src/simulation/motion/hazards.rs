use super::*;
use crate::ObjectMapMut;

/// DOS trap metadata and mutable animation, owned by the caller. Non-trap
/// object slots are None; indices match compact non-entrance ObjectInfos.
pub type TrapStates = [Option<TrapState>; 16];
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TrapState {
    frame_count: u8,
    frame: u8,
    triggered: bool,
    continuous: bool,
}
impl TrapState {
    pub fn new(frame_count: u8, first_frame: u8) -> Result<Self, MotionError> {
        if frame_count == 0 || first_frame >= frame_count {
            return Err(MotionError::InvalidTrapFrames);
        }
        Ok(Self {
            frame_count,
            frame: first_frame,
            triggered: false,
            continuous: false,
        })
    }
    pub const fn with_continuous(mut self, continuous: bool) -> Self {
        self.continuous = continuous;
        self
    }
    pub const fn continuous(self) -> bool {
        self.continuous
    }
    pub const fn frame(self) -> u8 {
        self.frame
    }
    pub const fn frame_count(self) -> u8 {
        self.frame_count
    }
    pub const fn triggered(self) -> bool {
        self.triggered
    }
}
/// Call once after actors and Nuke, only on logical updates. DOS uses the
/// corrected zero start frame; the same tick's object update displays frame1.
pub fn advance_traps(traps: &mut TrapStates) {
    for trap in traps.iter_mut().flatten() {
        if trap.triggered || trap.continuous {
            if trap.frame == trap.frame_count - 1 {
                trap.frame = 0;
                trap.triggered = false;
            } else {
                trap.frame += 1;
            }
        }
    }
}
/// Eight jobs/Bomber followed by raw trap arbitration in stable ID order.
/// Missing trap definitions, including saved Blocker map references, and ID
/// order errors are rejected before any actor/terrain/map/trap changes.
/// Like the other mutable batches, later arithmetic errors retain prior commits.
pub fn step_actors_with_hazards(
    actors: &mut [Actor],
    terrain: &mut TerrainMaskMut<'_>,
    context: BomberContext<'_>,
    map: &mut ObjectMapMut<'_>,
    traps: &mut TrapStates,
    mut emit: impl FnMut(ActorState, MotionOutcome),
) -> Result<ActorTick, MotionError> {
    for (index, pair) in actors.windows(2).enumerate() {
        if pair[0].state.id >= pair[1].state.id {
            return Err(MotionError::ActorOrder { index: index + 1 });
        }
    }
    let codes = map.as_read_only().as_bytes().iter().copied().chain(
        actors
            .iter()
            .filter_map(|a| a.state.blocker_saved)
            .flatten(),
    );
    for code in codes {
        if code < 128 && !traps.get(usize::from(code)).is_some_and(Option::is_some) {
            return Err(MotionError::MissingTrap(code));
        }
    }
    let mut tick = ActorTick::default();
    for actor in actors {
        let removed = actor.state.removed.is_some();
        let mut out = actor.advance_with_bomber(terrain, context, map)?;
        if removed {
            continue;
        }
        if out.check_objects && actor.state.y < 160 && actor.state.object_below < 128 {
            let index = usize::from(actor.state.object_below);
            let trap = traps[index].as_mut().expect("prevalidated trap reference");
            if !trap.triggered {
                trap.triggered = true;
                trap.frame = 0;
                remove(&mut actor.state, &mut out, RemovalReason::Trapped);
            }
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
