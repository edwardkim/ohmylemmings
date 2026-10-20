use lemmings_core::SoundCue;

/// A logical edge at one control boundary. Sequence preserves emission order;
/// assignment events also carry the accepted input's sequence and resolved actor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SessionSound {
    pub control_step: u64,
    pub iteration: u64,
    pub sequence: u64,
    pub input_sequence: Option<u32>,
    pub actor_id: Option<u32>,
    pub cue: SoundCue,
}

pub(crate) fn push(
    output: &mut Vec<SessionSound>,
    control_step: u64,
    iteration: u64,
    input_sequence: Option<u32>,
    actor_id: Option<u32>,
    cue: SoundCue,
) {
    output.push(SessionSound {
        control_step,
        iteration,
        sequence: output.len() as u64,
        input_sequence,
        actor_id,
        cue,
    });
}
