//! Deterministic timing, immutable boundary inputs and a gameplay-neutral driver.
mod clock;
mod input;
mod stepper;

pub use clock::{FixedTickClock, TICKS_PER_SECOND, TickOverflow};
pub use input::{InputCommand, InputOrderError, InputSchedule, TimedInput};
pub use stepper::{InvalidStepState, StepEvent, StepOutcome, StepOverflow, StepState, TickStepper};
