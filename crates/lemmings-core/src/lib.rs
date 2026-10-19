#![no_std]
//! Domain types, checked terrain views and deterministic simulation scheduling.
//!
//! ```
//! use lemmings_core::{Difficulty, LevelId};
//!
//! let first = LevelId::new(Difficulty::Fun, 1)?;
//! assert_eq!(first.index(), 0);
//! assert_eq!(first.global_index(), 0);
//! assert!(LevelId::new(Difficulty::Fun, 0).is_err());
//! # Ok::<(), lemmings_core::InvalidLevelNumber>(())
//! ```

mod level;
mod simulation;
mod terrain;

pub use level::{Difficulty, InvalidLevelNumber, LevelId};

pub use terrain::{TerrainChange, TerrainMask, TerrainMaskError, TerrainMaskMut, TerrainRect};

pub use simulation::{
    FixedTickClock, InputCommand, InputOrderError, InputSchedule, InvalidStepState, StepEvent,
    StepOutcome, StepOverflow, StepState, TICKS_PER_SECOND, TickOverflow, TickStepper, TimedInput,
};

pub use simulation::{
    SpawnRequest, StartupCommand, StartupConfig, StartupConfigError, StartupEvent,
    StartupScheduler, StartupState,
};

pub use simulation::{
    Actor, ActorState, ActorTick, BashingContext, EntrancePoint, MiningContext, MotionAction,
    MotionDirection, MotionError, MotionOutcome, MotionRules, RemovalReason, step_actors,
    step_actors_mut, step_actors_with_jobs, step_actors_with_skills,
};

pub use simulation::{
    DOS_OBJECT_MAP_HEIGHT, DOS_OBJECT_MAP_ORIGIN, DOS_OBJECT_MAP_WIDTH, OBJECT_BLOCKER,
    OBJECT_EXIT, OBJECT_FIRE, OBJECT_FORCE_LEFT, OBJECT_FORCE_RIGHT, OBJECT_NONE, OBJECT_WATER,
    ObjectMap, ObjectMapError, ObjectMapMut, step_actors_with_objects,
};

pub use simulation::{BomberContext, step_actors_with_bomber};
pub use simulation::{NukeActorError, NukeState};

pub use simulation::{TrapState, TrapStates, advance_traps, step_actors_with_hazards};

pub use simulation::{
    CompletionError, CompletionProgress, CompletionRules, EndReason, LevelResult, percentage,
};
