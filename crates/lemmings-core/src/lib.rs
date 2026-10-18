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
    Actor, ActorState, ActorTick, EntrancePoint, MotionAction, MotionDirection, MotionError,
    MotionOutcome, MotionRules, RemovalReason, step_actors,
};
