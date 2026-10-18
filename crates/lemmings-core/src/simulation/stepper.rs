use super::{InputCommand, InputSchedule, TimedInput};
use core::fmt;

/// Snapshot before the next boundary input. Initial counters are both zero.
/// The logical iteration never exceeds the number of processed control calls.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct StepState {
    pub control_step: u64,
    pub iteration: u64,
    pub paused: bool,
}

/// Every input is emitted in sequence order, followed by one Tick if unpaused.
/// Tick carries the newly incremented logical iteration, starting at 1.
#[derive(Debug, Eq, PartialEq)]
pub enum StepEvent<'a, C> {
    Input(&'a TimedInput<C>),
    Tick(u64),
}

/// The boundary just processed, not the next snapshot's control_step.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StepOutcome {
    pub control_step: u64,
    pub iteration: Option<u64>,
    pub input_count: usize,
}

/// Logical boundary driver, with no wall time, rendering, actors, or I/O.
/// Pause keeps control calls/inputs flowing while suppressing Tick events.
/// This does not implement game clocks, PauseGlitch, or other gameplay phases.
///
/// ```
/// use lemmings_core::{InputCommand, InputSchedule, TickStepper, TimedInput};
/// let inputs: [TimedInput<()>; 2] = [
///     TimedInput { control_step: 0, sequence: 0, command: InputCommand::Pause },
///     TimedInput { control_step: 2, sequence: 0, command: InputCommand::Resume },
/// ];
/// let schedule = InputSchedule::new(&inputs).unwrap();
/// let mut stepper = TickStepper::new();
/// assert_eq!(stepper.step(&schedule, |_| {}).unwrap().iteration, None);
/// assert_eq!(stepper.step(&schedule, |_| {}).unwrap().iteration, None);
/// assert_eq!(stepper.step(&schedule, |_| {}).unwrap().iteration, Some(1));
/// assert_eq!(stepper.state().control_step, 3);
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TickStepper {
    state: StepState,
}

impl TickStepper {
    pub const fn new() -> Self {
        Self {
            state: StepState {
                control_step: 0,
                iteration: 0,
                paused: false,
            },
        }
    }

    /// Restore a boundary snapshot. Application state and the matching immutable
    /// input schedule must be restored separately by the caller.
    pub const fn from_state(state: StepState) -> Result<Self, InvalidStepState> {
        if state.iteration > state.control_step {
            Err(InvalidStepState)
        } else {
            Ok(Self { state })
        }
    }

    pub const fn state(&self) -> StepState {
        self.state
    }

    /// Process this boundary's inputs, then emit the optional logical update.
    /// On counter overflow, emits nothing and leaves state unchanged. Callbacks
    /// must complete normally; application effects are not rolled back on panic.
    /// Custom payload success/rejection and game phase order are caller duties.
    pub fn step<'a, C>(
        &mut self,
        schedule: &InputSchedule<'a, C>,
        mut emit: impl FnMut(StepEvent<'a, C>),
    ) -> Result<StepOutcome, StepOverflow> {
        let inputs = schedule.at(self.state.control_step);
        let next_control = self.state.control_step.checked_add(1).ok_or(StepOverflow)?;
        let mut paused = self.state.paused;
        for input in inputs {
            match &input.command {
                InputCommand::Pause => paused = true,
                InputCommand::Resume => paused = false,
                InputCommand::Custom(_) => {}
            }
        }
        // iteration <= control_step means this cannot overflow before control.
        let iteration = self.state.iteration + u64::from(!paused);
        for input in inputs {
            emit(StepEvent::Input(input));
        }
        let advanced = (!paused).then_some(iteration);
        if let Some(iteration) = advanced {
            emit(StepEvent::Tick(iteration));
        }
        let result = StepOutcome {
            control_step: self.state.control_step,
            iteration: advanced,
            input_count: inputs.len(),
        };
        self.state = StepState {
            control_step: next_control,
            iteration,
            paused,
        };
        Ok(result)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidStepState;
impl fmt::Display for InvalidStepState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("logical iteration exceeds processed control calls")
    }
}
impl core::error::Error for InvalidStepState {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StepOverflow;
impl fmt::Display for StepOverflow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("control step counter is exhausted")
    }
}
impl core::error::Error for StepOverflow {}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use crate::FixedTickClock;
    use core::time::Duration;
    use std::{vec, vec::Vec};

    fn event(step: u64, sequence: u32, command: InputCommand<u8>) -> TimedInput<u8> {
        TimedInput {
            control_step: step,
            sequence,
            command,
        }
    }

    #[test]
    fn empty_initial_boundary_emits_iteration_one_and_stores_next_snapshot() {
        let schedule = InputSchedule::<()>::new(&[]).unwrap();
        let mut stepper = TickStepper::new();
        assert_eq!(stepper, TickStepper::default());
        assert_eq!(stepper.state(), StepState::default());
        let mut events = Vec::new();
        assert_eq!(
            stepper.step(&schedule, |e| events.push(e)),
            Ok(StepOutcome {
                control_step: 0,
                iteration: Some(1),
                input_count: 0,
            })
        );
        assert_eq!(events, [StepEvent::Tick(1)]);
        assert_eq!(
            stepper.state(),
            StepState {
                control_step: 1,
                iteration: 1,
                paused: false
            }
        );
    }

    #[test]
    fn zero_boundary_emits_every_input_before_the_first_tick() {
        let inputs = [
            event(0, 3, InputCommand::Custom(8)),
            event(0, 9, InputCommand::Custom(2)),
        ];
        let schedule = InputSchedule::new(&inputs).unwrap();
        let mut stepper = TickStepper::new();
        let mut events = Vec::new();
        let result = stepper.step(&schedule, |e| events.push(e)).unwrap();
        assert_eq!(
            events,
            [
                StepEvent::Input(&inputs[0]),
                StepEvent::Input(&inputs[1]),
                StepEvent::Tick(1)
            ]
        );
        assert_eq!(result.input_count, 2);
    }

    #[test]
    fn pause_keeps_control_calls_and_resume_input_flowing() {
        let inputs = [
            event(0, 0, InputCommand::Pause),
            event(2, 0, InputCommand::Resume),
        ];
        let schedule = InputSchedule::new(&inputs).unwrap();
        let mut stepper = TickStepper::new();
        let mut events = Vec::new();
        for _ in 0..3 {
            stepper.step(&schedule, |e| events.push(e)).unwrap();
        }
        assert_eq!(
            events,
            [
                StepEvent::Input(&inputs[0]),
                StepEvent::Input(&inputs[1]),
                StepEvent::Tick(1)
            ]
        );
        assert_eq!(
            stepper.state(),
            StepState {
                control_step: 3,
                iteration: 1,
                paused: false
            }
        );
    }

    #[test]
    fn final_pause_command_controls_tick_but_all_boundary_inputs_are_emitted() {
        let inputs = [
            event(0, 0, InputCommand::Pause),
            event(0, 1, InputCommand::Resume),
            event(0, 8, InputCommand::Custom(42)),
            event(0, 9, InputCommand::Pause),
            event(1, 0, InputCommand::Resume),
            event(1, 1, InputCommand::Resume),
        ];
        let schedule = InputSchedule::new(&inputs).unwrap();
        let mut stepper = TickStepper::new();
        let mut events = Vec::new();
        assert_eq!(
            stepper
                .step(&schedule, |e| events.push(e))
                .unwrap()
                .iteration,
            None
        );
        assert_eq!(
            events,
            inputs[..4].iter().map(StepEvent::Input).collect::<Vec<_>>()
        );
        assert_eq!(stepper.step(&schedule, |_| {}).unwrap().iteration, Some(1));
        assert!(!stepper.state().paused);
    }

    #[test]
    fn future_events_are_not_early_or_repeated_after_their_boundary() {
        let inputs = [event(1, 0, InputCommand::Custom(7))];
        let schedule = InputSchedule::new(&inputs).unwrap();
        let mut stepper = TickStepper::new();
        let mut events = Vec::new();
        for _ in 0..3 {
            stepper.step(&schedule, |e| events.push(e)).unwrap();
        }
        assert_eq!(
            events,
            [
                StepEvent::Tick(1),
                StepEvent::Input(&inputs[0]),
                StepEvent::Tick(2),
                StepEvent::Tick(3)
            ]
        );
        assert_eq!(schedule.as_inputs(), &inputs);
    }

    #[test]
    fn restored_snapshots_enforce_counter_invariant_and_preserve_pause() {
        for state in [
            StepState {
                control_step: 0,
                iteration: 1,
                paused: false,
            },
            StepState {
                control_step: u64::MAX - 1,
                iteration: u64::MAX,
                paused: true,
            },
        ] {
            assert_eq!(TickStepper::from_state(state), Err(InvalidStepState));
        }
        let state = StepState {
            control_step: 9,
            iteration: 2,
            paused: true,
        };
        let mut stepper = TickStepper::from_state(state).unwrap();
        assert_eq!(stepper.state(), state);
        let empty = InputSchedule::<()>::new(&[]).unwrap();
        assert_eq!(stepper.step(&empty, |_| {}).unwrap().iteration, None);
        assert_eq!(
            stepper.state(),
            StepState {
                control_step: 10,
                ..state
            }
        );
    }

    #[test]
    fn last_representable_boundary_succeeds_then_overflow_is_atomic() {
        let empty = InputSchedule::<u8>::new(&[]).unwrap();
        let mut stepper = TickStepper::from_state(StepState {
            control_step: u64::MAX - 1,
            iteration: u64::MAX - 1,
            paused: false,
        })
        .unwrap();
        assert_eq!(
            stepper.step(&empty, |_| {}).unwrap().iteration,
            Some(u64::MAX)
        );
        let before = stepper;
        let inputs = [event(u64::MAX, 0, InputCommand::Pause)];
        let schedule = InputSchedule::new(&inputs).unwrap();
        let mut calls = 0;
        assert_eq!(stepper.step(&schedule, |_| calls += 1), Err(StepOverflow));
        assert_eq!(calls, 0);
        assert_eq!(stepper, before);
        let mut paused = TickStepper::from_state(StepState {
            control_step: u64::MAX,
            iteration: 0,
            paused: true,
        })
        .unwrap();
        assert_eq!(paused.step(&empty, |_| calls += 1), Err(StepOverflow));
        assert_eq!(calls, 0);
        assert!(paused.state().paused);
    }

    #[test]
    fn changing_same_boundary_sequence_changes_noncommuting_application_result() {
        fn run(a: u8, b: u8) -> u64 {
            let inputs = [
                event(0, 0, InputCommand::Custom(a)),
                event(0, 1, InputCommand::Custom(b)),
            ];
            let schedule = InputSchedule::new(&inputs).unwrap();
            let mut value = 1;
            TickStepper::new()
                .step(&schedule, |e| match e {
                    StepEvent::Input(TimedInput {
                        command: InputCommand::Custom(2),
                        ..
                    }) => value += 1,
                    StepEvent::Input(TimedInput {
                        command: InputCommand::Custom(3),
                        ..
                    }) => value *= 2,
                    StepEvent::Tick(iteration) => value += iteration,
                    _ => {}
                })
                .unwrap();
            value
        }
        assert_eq!(run(2, 3), 5);
        assert_eq!(run(3, 2), 4);
        assert_eq!(run(2, 3), run(2, 3));
    }

    fn command(code: u8) -> InputCommand<u8> {
        match code {
            0 => InputCommand::Pause,
            1 => InputCommand::Resume,
            c => InputCommand::Custom(c),
        }
    }

    // Independent test application: add/double on input, add iteration on Tick.
    // The reference model below works directly on codes, without schedule/clock.
    fn replay(inputs: &[TimedInput<u8>], frames: &[u64]) -> (StepState, u64, Vec<u8>) {
        let schedule = InputSchedule::new(inputs).unwrap();
        let mut clock = FixedTickClock::new();
        let mut stepper = TickStepper::new();
        let mut value = 1u64;
        let mut stream = Vec::new();
        for &ns in frames {
            for _ in 0..clock.advance(Duration::from_nanos(ns)).unwrap() {
                stepper
                    .step(&schedule, |e| match e {
                        StepEvent::Input(input) => {
                            let code = match input.command {
                                InputCommand::Pause => 0,
                                InputCommand::Resume => 1,
                                InputCommand::Custom(c) => c,
                            };
                            stream.extend_from_slice(&[
                                0,
                                input.control_step as u8,
                                input.sequence as u8,
                                code,
                            ]);
                            match code {
                                2 => value += 1,
                                3 => value *= 2,
                                _ => {}
                            }
                        }
                        StepEvent::Tick(iteration) => {
                            value += iteration;
                            stream.extend_from_slice(&[1, iteration as u8]);
                        }
                    })
                    .unwrap();
            }
        }
        assert_eq!(clock.phase_numerator(), 0);
        let s = stepper.state();
        stream.extend_from_slice(&[
            2,
            s.control_step as u8,
            s.iteration as u8,
            u8::from(s.paused),
        ]);
        stream.extend_from_slice(&value.to_le_bytes());
        (s, value, stream)
    }

    #[test]
    fn exhaustive_replays_match_integer_model_across_three_render_patterns() {
        let sixty_hz: Vec<_> = (0..60u64)
            .map(|i| (i + 1) * 1_000_000_000 / 60 - i * 1_000_000_000 / 60)
            .collect();
        let patterns = [
            vec![1_000_000_000],
            sixty_hz,
            vec![
                0,
                1,
                58_823_528,
                1,
                240_000_001,
                0,
                123_456_789,
                577_719_680,
            ],
        ];
        for p in &patterns {
            assert_eq!(p.iter().sum::<u64>(), 1_000_000_000);
        }
        let mut scenarios = 0;
        let mut hash = 0xcbf29ce484222325u64;
        for length in 0..=6usize {
            for encoded in 0..4usize.pow(length as u32) {
                let codes: Vec<_> = (0..length)
                    .map(|i| ((encoded >> (2 * (length - 1 - i))) & 3) as u8)
                    .collect();
                let inputs: Vec<_> = codes
                    .iter()
                    .enumerate()
                    .map(|(i, &c)| event((i / 2) as u64, (i % 2) as u32, command(c)))
                    .collect();
                let (mut paused, mut iteration, mut value, mut wanted) =
                    (false, 0u64, 1u64, Vec::new());
                for step in 0..17u64 {
                    for (i, &code) in codes
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| i / 2 == step as usize)
                    {
                        wanted.extend_from_slice(&[0, step as u8, (i % 2) as u8, code]);
                        match code {
                            0 => paused = true,
                            1 => paused = false,
                            2 => value += 1,
                            3 => value *= 2,
                            _ => unreachable!(),
                        }
                    }
                    if !paused {
                        iteration += 1;
                        value += iteration;
                        wanted.extend_from_slice(&[1, iteration as u8]);
                    }
                }
                wanted.extend_from_slice(&[2, 17, iteration as u8, u8::from(paused)]);
                wanted.extend_from_slice(&value.to_le_bytes());
                let expected = (
                    StepState {
                        control_step: 17,
                        iteration,
                        paused,
                    },
                    value,
                    wanted,
                );
                for p in &patterns {
                    assert_eq!(replay(&inputs, p), expected);
                }
                for byte in expected.2 {
                    hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
                }
                scenarios += 1;
            }
        }
        assert_eq!(scenarios, 5461);
        assert_eq!(hash, 0x5338d6ae252490ba);
        std::println!(
            "5461 scenarios / 16383 replays / 278511 control calls / fnv1a64={hash:016x}"
        );
    }
}
