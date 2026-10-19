//! Deterministic timing, immutable boundary inputs and startup scheduling.
mod clock;
mod input;
mod motion;
mod objects;
mod startup;
mod stepper;

pub use clock::{FixedTickClock, TICKS_PER_SECOND, TickOverflow};
pub use input::{InputCommand, InputOrderError, InputSchedule, TimedInput};
pub use stepper::{InvalidStepState, StepEvent, StepOutcome, StepOverflow, StepState, TickStepper};

pub use startup::{
    SpawnRequest, StartupCommand, StartupConfig, StartupConfigError, StartupEvent,
    StartupScheduler, StartupState,
};

pub use motion::{
    Actor, ActorState, ActorTick, BashingContext, EntrancePoint, MiningContext, MotionAction,
    MotionDirection, MotionError, MotionOutcome, MotionRules, RemovalReason, step_actors,
    step_actors_mut, step_actors_with_jobs, step_actors_with_objects, step_actors_with_skills,
};

pub use objects::{
    DOS_OBJECT_MAP_HEIGHT, DOS_OBJECT_MAP_ORIGIN, DOS_OBJECT_MAP_WIDTH, OBJECT_BLOCKER,
    OBJECT_FORCE_LEFT, OBJECT_FORCE_RIGHT, OBJECT_NONE, ObjectMap, ObjectMapError, ObjectMapMut,
};
