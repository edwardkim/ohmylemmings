use core::fmt;

/// Pause/Resume affect logical tick notification only. Custom payload semantics
/// and their acceptance/rejection belong to the application, not this schedule.
#[derive(Debug, Eq, PartialEq)]
pub enum InputCommand<C> {
    Pause,
    Resume,
    Custom(C),
}

/// Boundary 0 is before the first update. Sequence numbers may have gaps and
/// restart at any value on another boundary. No rendering timestamps are stored.
#[derive(Debug, Eq, PartialEq)]
pub struct TimedInput<C> {
    pub control_step: u64,
    pub sequence: u32,
    pub command: InputCommand<C>,
}

/// Validated immutable input list, strictly ordered by (control_step, sequence).
/// Validation neither sorts nor copies the caller's data or payloads.
///
/// ```
/// use lemmings_core::{InputCommand, InputSchedule, TimedInput};
/// let inputs = [TimedInput { control_step: 0, sequence: 7,
///                           command: InputCommand::Custom("select") }];
/// let schedule = InputSchedule::new(&inputs)?;
/// assert_eq!(schedule.at(0), &inputs);
/// assert!(schedule.at(1).is_empty());
/// # Ok::<(), lemmings_core::InputOrderError>(())
/// ```
#[derive(Debug)]
pub struct InputSchedule<'a, C> {
    inputs: &'a [TimedInput<C>],
}

impl<'a, C> InputSchedule<'a, C> {
    /// Reports the first index whose key is not greater than the preceding key.
    pub fn new(inputs: &'a [TimedInput<C>]) -> Result<Self, InputOrderError> {
        for (i, pair) in inputs.windows(2).enumerate() {
            let key = |v: &TimedInput<C>| (v.control_step, v.sequence);
            if key(&pair[0]) >= key(&pair[1]) {
                return Err(InputOrderError { index: i + 1 });
            }
        }
        Ok(Self { inputs })
    }

    pub const fn as_inputs(&self) -> &'a [TimedInput<C>] {
        self.inputs
    }

    /// Exact boundary lookup in O(log n), including u64::MAX. Does not consume
    /// events; callers may replay the same schedule from a new stepper.
    pub fn at(&self, control_step: u64) -> &'a [TimedInput<C>] {
        let start = self
            .inputs
            .partition_point(|v| v.control_step < control_step);
        let end = self
            .inputs
            .partition_point(|v| v.control_step <= control_step);
        &self.inputs[start..end]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InputOrderError {
    pub index: usize,
}
impl fmt::Display for InputOrderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "input at index {} is not strictly ordered by (control_step, sequence)",
            self.index
        )
    }
}
impl core::error::Error for InputOrderError {}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use std::string::String;
    fn event(step: u64, sequence: u32) -> TimedInput<()> {
        TimedInput {
            control_step: step,
            sequence,
            command: InputCommand::Custom(()),
        }
    }

    #[test]
    fn empty_schedule_has_no_events_at_any_boundary() {
        let schedule = InputSchedule::<()>::new(&[]).unwrap();
        assert!(schedule.as_inputs().is_empty());
        for step in [0, 1, u64::MAX] {
            assert!(schedule.at(step).is_empty());
        }
    }

    #[test]
    fn boundary_groups_keep_gaps_and_allow_sequence_reset_between_steps() {
        let inputs = [event(0, 7), event(0, 100), event(2, 0), event(2, 1)];
        let schedule = InputSchedule::new(&inputs).unwrap();
        assert_eq!(schedule.at(0), &inputs[..2]);
        assert!(schedule.at(1).is_empty());
        assert_eq!(schedule.at(2), &inputs[2..]);
        assert!(schedule.at(3).is_empty());
        assert_eq!(schedule.at(0).as_ptr(), inputs.as_ptr());
        assert_eq!(schedule.as_inputs().as_ptr(), inputs.as_ptr());
    }

    #[test]
    fn duplicate_reverse_sequence_and_reverse_boundary_are_rejected() {
        for inputs in [
            [event(0, 1), event(0, 1)],
            [event(0, 2), event(0, 1)],
            [event(1, 0), event(0, 100)],
        ] {
            assert_eq!(
                InputSchedule::new(&inputs).unwrap_err(),
                InputOrderError { index: 1 }
            );
        }
    }

    #[test]
    fn validation_reports_first_bad_key_without_changing_input_data() {
        let inputs = [event(0, 1), event(2, 0), event(1, 3), event(0, 0)];
        assert_eq!(
            InputSchedule::new(&inputs).unwrap_err(),
            InputOrderError { index: 2 }
        );
        assert_eq!(inputs, [event(0, 1), event(2, 0), event(1, 3), event(0, 0)]);
    }

    #[test]
    fn maximum_step_and_sequence_are_valid_and_lookup_never_adds_one() {
        let inputs = [
            event(0, u32::MAX),
            event(u64::MAX, 0),
            event(u64::MAX, u32::MAX),
        ];
        let schedule = InputSchedule::new(&inputs).unwrap();
        assert!(schedule.at(u64::MAX - 1).is_empty());
        assert_eq!(schedule.at(u64::MAX), &inputs[1..]);
        assert_eq!(schedule.at(0), &inputs[..1]);
    }

    #[test]
    fn payload_does_not_need_copy_or_clone_and_remains_borrowed() {
        #[derive(Debug, Eq, PartialEq)]
        struct Payload(String);
        let inputs = [TimedInput {
            control_step: 0,
            sequence: 0,
            command: InputCommand::Custom(Payload(String::from("original"))),
        }];
        let schedule = InputSchedule::new(&inputs).unwrap();
        assert_eq!(schedule.at(0), &inputs);
        assert_eq!(schedule.as_inputs().as_ptr(), inputs.as_ptr());
    }
}
