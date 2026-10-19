use super::{
    InputCommand, InputSchedule, StepEvent, StepOutcome, StepOverflow, StepState, TickStepper,
};
use core::fmt;

/// Validated level-start inputs. Entrance indices refer to the caller's original
/// ordered entrance list. Zero entrances or a zero total produce no births.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StartupConfig {
    pub time_limit_minutes: u16,
    pub total_lemmings: u32,
    pub initial_release_rate: u8,
    pub entrance_count: u8,
    pub pause_glitch: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StartupConfigError {
    ReleaseRate(u8),
    EntranceCount(u8),
}
impl fmt::Display for StartupConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReleaseRate(v) => write!(f, "initial release rate {v} exceeds 99"),
            Self::EntranceCount(v) => write!(f, "entrance count {v} exceeds 4"),
        }
    }
}
impl core::error::Error for StartupConfigError {}

/// The scheduler accepts rates in initial_release_rate..=99. It does not clamp
/// invalid commands or reset an already-running spawn countdown.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StartupCommand {
    SetReleaseRate(u8),
    AssignDigger(u32),
    AssignBasher(u32),
    AssignMiner(u32),
    AssignBuilder(u32),
    AssignBlocker(u32),
    AssignBomber(u32),
    AssignClimber(u32),
    AssignFloater(u32),
}

/// Progress immediately before the next boundary. Read control/iteration/pause
/// counters separately through StartupScheduler::step_state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StartupState {
    pub seconds_remaining: u32,
    pub clock_frame: u8,
    pub glitch_pause_iterations: u8,
    pub release_rate: u8,
    pub spawn_countdown: u8,
    pub entrances_open: bool,
    pub released: u32,
}

/// A birth request, not an allocated actor or a terrain/coordinate update.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpawnRequest {
    pub actor_id: u32,
    pub entrance_index: u8,
    pub iteration: u64,
}

/// Boundary inputs are acknowledged in order. Startup markers and birth requests
/// precede Tick, allowing an application to update actors after births.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StartupEvent {
    Input {
        control_step: u64,
        sequence: u32,
        accepted: bool,
    },
    LetsGo,
    EntranceSound,
    EntrancesOpened,
    Spawn(SpawnRequest),
    Tick(u64),
}

/// Startup clock and birth scheduling over the shared deterministic stepper.
/// The clock saturates at zero; whole-game termination, actors, Nuke, music and
/// replay restoration are application/future gameplay duties. Callbacks must
/// finish normally; external effects are not rolled back after a panic.
///
/// ```
/// use lemmings_core::{InputSchedule, SpawnRequest, StartupCommand, StartupConfig,
///                     StartupEvent, StartupScheduler};
/// let config = StartupConfig { time_limit_minutes: 5, total_lemmings: 10,
///     initial_release_rate: 50, entrance_count: 1, pause_glitch: true };
/// let mut startup = StartupScheduler::new(config).unwrap();
/// let inputs = InputSchedule::<StartupCommand>::new(&[]).unwrap();
/// let mut birth = None;
/// for _ in 0..54 {
///     startup.step(&inputs, |e| if let StartupEvent::Spawn(s) = e { birth = Some(s); }).unwrap();
/// }
/// assert_eq!(birth, Some(SpawnRequest { actor_id: 0, entrance_index: 0, iteration: 54 }));
/// assert_eq!(startup.state().spawn_countdown, 28);
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StartupScheduler {
    config: StartupConfig,
    stepper: TickStepper,
    state: StartupState,
}

impl StartupScheduler {
    /// Validate release rate first, then entrance count; no allocation or I/O.
    pub const fn new(config: StartupConfig) -> Result<Self, StartupConfigError> {
        if config.initial_release_rate > 99 {
            return Err(StartupConfigError::ReleaseRate(config.initial_release_rate));
        }
        if config.entrance_count > 4 {
            return Err(StartupConfigError::EntranceCount(config.entrance_count));
        }
        Ok(Self {
            config,
            stepper: TickStepper::new(),
            state: StartupState {
                seconds_remaining: config.time_limit_minutes as u32 * 60,
                clock_frame: 0,
                glitch_pause_iterations: 0,
                release_rate: config.initial_release_rate,
                spawn_countdown: 20,
                entrances_open: false,
                released: 0,
            },
        })
    }

    pub const fn config(&self) -> StartupConfig {
        self.config
    }
    pub const fn state(&self) -> StartupState {
        self.state
    }
    pub const fn step_state(&self) -> StepState {
        self.stepper.state()
    }

    /// Process the boundary's inputs before clock/startup/birth updates. Emits
    /// accepted=false for an invalid rate while continuing other inputs/ticks.
    /// Counter overflow preserves both scheduler/stepper state and emits nothing.
    ///
    /// ```
    /// use lemmings_core::{InputCommand, InputSchedule, StartupCommand, StartupConfig,
    ///                     StartupScheduler, TimedInput};
    /// let config = StartupConfig { time_limit_minutes: 1, total_lemmings: 1,
    ///     initial_release_rate: 50, entrance_count: 1, pause_glitch: false };
    /// let mut startup = StartupScheduler::new(config).unwrap();
    /// let input = [TimedInput { control_step: 0, sequence: 0,
    ///     command: InputCommand::Custom(StartupCommand::SetReleaseRate(99)) }];
    /// startup.step(&InputSchedule::new(&input).unwrap(), |_| {}).unwrap();
    /// assert_eq!(startup.state().release_rate, 99);
    /// assert_eq!(startup.state().spawn_countdown, 20);
    /// ```
    pub fn step(
        &mut self,
        schedule: &InputSchedule<'_, StartupCommand>,
        emit: impl FnMut(StartupEvent),
    ) -> Result<StepOutcome, StepOverflow> {
        self.step_with_digger(schedule, |_| false, emit)
    }

    /// Process Digger requests in boundary input order, before spawning/motion.
    /// The caller owns actors, inventory and steel; plain `step` rejects skills.
    pub fn step_with_digger(
        &mut self,
        schedule: &InputSchedule<'_, StartupCommand>,
        mut assign: impl FnMut(u32) -> bool,
        emit: impl FnMut(StartupEvent),
    ) -> Result<StepOutcome, StepOverflow> {
        self.step_with_skills(
            schedule,
            |command| match command {
                StartupCommand::AssignDigger(id) => assign(id),
                _ => false,
            },
            emit,
        )
    }

    /// Delegate supported skill assignments in exact boundary/sequence order.
    /// Rates, pause and resume remain scheduler-owned; unsupported jobs reject.
    pub fn step_with_skills(
        &mut self,
        schedule: &InputSchedule<'_, StartupCommand>,
        mut assign: impl FnMut(StartupCommand) -> bool,
        mut emit: impl FnMut(StartupEvent),
    ) -> Result<StepOutcome, StepOverflow> {
        let before = self.stepper.state();
        let mut paused = before.paused;
        let config = self.config;
        let state = &mut self.state;
        self.stepper.step(schedule, |event| match event {
            StepEvent::Input(input) => {
                let accepted = match input.command {
                    InputCommand::Pause => {
                        if !paused
                            && config.pause_glitch
                            && before.iteration < 34
                            && state.glitch_pause_iterations == 0
                        {
                            state.glitch_pause_iterations = (34 - before.iteration) as u8;
                        }
                        paused = true;
                        true
                    }
                    InputCommand::Resume => {
                        paused = false;
                        true
                    }
                    InputCommand::Custom(
                        command @ (StartupCommand::AssignDigger(_)
                        | StartupCommand::AssignBasher(_)
                        | StartupCommand::AssignMiner(_)
                        | StartupCommand::AssignBuilder(_)
                        | StartupCommand::AssignBlocker(_)
                        | StartupCommand::AssignBomber(_)
                        | StartupCommand::AssignClimber(_)
                        | StartupCommand::AssignFloater(_)),
                    ) => assign(command),
                    InputCommand::Custom(StartupCommand::SetReleaseRate(rate)) => {
                        if (config.initial_release_rate..=99).contains(&rate) {
                            state.release_rate = rate;
                            true
                        } else {
                            false
                        }
                    }
                };
                emit(StartupEvent::Input {
                    control_step: input.control_step,
                    sequence: input.sequence,
                    accepted,
                });
            }
            StepEvent::Tick(iteration) => {
                if config.pause_glitch && state.glitch_pause_iterations > 0 {
                    state.glitch_pause_iterations -= 1;
                } else {
                    state.clock_frame += 1;
                    if state.clock_frame == 17 {
                        state.clock_frame = 0;
                        state.seconds_remaining = state.seconds_remaining.saturating_sub(1);
                    }
                }
                match iteration {
                    15 => emit(StartupEvent::LetsGo),
                    34 => emit(StartupEvent::EntranceSound),
                    35 => {
                        state.entrances_open = true;
                        emit(StartupEvent::EntrancesOpened);
                    }
                    _ => {}
                }
                if state.entrances_open {
                    state.spawn_countdown -= 1;
                    if state.spawn_countdown == 0 {
                        state.spawn_countdown = (99 - state.release_rate) / 2 + 4;
                        if state.released < config.total_lemmings && config.entrance_count > 0 {
                            let order = match config.entrance_count {
                                2 => [0, 1, 1, 0],
                                3 => [0, 1, 2, 1],
                                4 => [0, 1, 2, 3],
                                _ => [0; 4],
                            };
                            let birth = SpawnRequest {
                                actor_id: state.released,
                                entrance_index: order[(state.released % 4) as usize],
                                iteration,
                            };
                            state.released += 1; // bounded by the validated u32 total
                            emit(StartupEvent::Spawn(birth));
                        }
                    }
                }
                emit(StartupEvent::Tick(iteration));
            }
        })
    }
}

#[cfg(test)]
mod tests;
