//! Deterministic timing, immutable boundary inputs and startup scheduling.
mod clock;
mod input;
mod motion;
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
    Actor, ActorState, ActorTick, EntrancePoint, MotionAction, MotionDirection, MotionError,
    MotionOutcome, MotionRules, RemovalReason, step_actors, step_actors_mut,
};
