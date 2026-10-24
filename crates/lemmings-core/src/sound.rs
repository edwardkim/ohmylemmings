//! Logical edges only. Resource IDs, synthesis and device policies live outside core.
use crate::AssignmentSkill;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SoundCue {
    /// Enhanced presentation cue; no original AdLib resource ID.
    DigStroke,
    BuilderWarning,
    AssignSkill(AssignmentSkill),
    AssignmentBlocked(AssignmentSkill),
    Yippee,
    Splat,
    LetsGo,
    EntranceOpening,
    Vaporizing,
    Drowning,
    Explosion,
    HitsSteel,
    OhNo,
    SkillButtonSelect,
    Trap {
        slot: u8,
    },
    MusicStart,
    LevelFinished,
}
