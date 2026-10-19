use super::{Actor, MotionAction};
use core::fmt;

/// BtnNuke/CheckUpdateNuking state, separate from the immutable level total.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NukeState {
    pub is_nuked: bool,
    pub assigning: bool,
    pub next_index: u32,
    /// NukeGlitch changes the eventual percentage denominator, not rescue logic.
    pub max_lemmings: u32,
}
impl NukeState {
    pub(super) const fn new(total: u32) -> Self {
        Self {
            is_nuked: false,
            assigning: false,
            next_index: 0,
            max_lemmings: total,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NukeActorError {
    ActorCount { expected: u32, actual: usize },
    ActorId { index: usize },
}
impl fmt::Display for NukeActorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ActorCount { expected, actual } => {
                write!(f, "nuke needs {expected} released records, got {actual}")
            }
            Self::ActorId { index } => write!(f, "nuke actor at {index} has a different stable ID"),
        }
    }
}
impl core::error::Error for NukeActorError {}

pub(super) fn update(
    state: &mut NukeState,
    released: u32,
    actors: &mut [Actor],
) -> Result<Option<u32>, NukeActorError> {
    if !state.is_nuked || !state.assigning {
        return Ok(None);
    }
    if actors.len() as u64 != u64::from(released) {
        return Err(NukeActorError::ActorCount {
            expected: released,
            actual: actors.len(),
        });
    }
    for (index, actor) in actors.iter().enumerate() {
        if u64::from(actor.state().id) != index as u64 {
            return Err(NukeActorError::ActorId { index });
        }
    }
    // DOS EXE213a tests a running timer before EXE2140's removed guard.
    // Lemmix skips all removed records; DOS spends a slot on a removed timer.
    while state.next_index < released
        && actors[state.next_index as usize].state().removed.is_some()
        && actors[state.next_index as usize].state().explosion_timer == 0
    {
        state.next_index += 1;
    }
    if state.next_index >= released {
        state.assigning = false;
        return Ok(None);
    }
    let actor = &mut actors[state.next_index as usize];
    state.next_index += 1;
    if actor.state().explosion_timer == 0
        && !matches!(
            actor.state().action,
            MotionAction::Splatting | MotionAction::Exploding
        )
    {
        actor.arm_nuke();
        Ok(Some(actor.state().id))
    } else {
        Ok(None)
    }
}
