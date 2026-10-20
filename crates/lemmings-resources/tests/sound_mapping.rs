use lemmings_core::{AssignmentSkill, SoundCue};
use lemmings_resources::{AdlibCommand, DosMusic, adlib_effect_for_cue};
#[test]
fn dos_call_sites_use_one_based_ids_and_reuse_effects_across_logical_events() {
    for (cue, raw) in [
        (SoundCue::SkillButtonSelect, 1),
        (SoundCue::EntranceOpening, 2),
        (SoundCue::LetsGo, 3),
        (SoundCue::OhNo, 5),
        (SoundCue::BuilderWarning, 18),
        (SoundCue::HitsSteel, 10),
        (SoundCue::Explosion, 12),
        (SoundCue::Vaporizing, 13),
        (SoundCue::Yippee, 16),
        (SoundCue::Drowning, 17),
        (SoundCue::Splat, 8),
    ] {
        assert_eq!(
            adlib_effect_for_cue(cue, 0).unwrap(),
            Some(AdlibCommand::Effect(raw - 1))
        );
    }
    for skill in [
        AssignmentSkill::Climber,
        AssignmentSkill::Floater,
        AssignmentSkill::Bomber,
        AssignmentSkill::Blocker,
        AssignmentSkill::Builder,
        AssignmentSkill::Digger,
        AssignmentSkill::Basher,
        AssignmentSkill::Miner,
    ] {
        let raw = 4;
        assert_eq!(
            adlib_effect_for_cue(SoundCue::AssignSkill(skill), 0).unwrap(),
            Some(AdlibCommand::Effect(raw - 1))
        );
    }
}
#[test]
fn raw_trap_ids_are_checked_and_zero_is_explicitly_silent() {
    for raw in 1..=18 {
        assert_eq!(
            adlib_effect_for_cue(SoundCue::Trap { slot: 15 }, raw).unwrap(),
            Some(AdlibCommand::Effect(raw - 1))
        );
    }
    assert_eq!(
        adlib_effect_for_cue(SoundCue::Trap { slot: 0 }, 0).unwrap(),
        None
    );
    assert!(adlib_effect_for_cue(SoundCue::Trap { slot: 0 }, 19).is_err());
    assert!(adlib_effect_for_cue(SoundCue::Trap { slot: 0 }, 255).is_err());
    assert_eq!(
        adlib_effect_for_cue(SoundCue::MusicStart, 255).unwrap(),
        None
    );
}
#[test]
fn original_normal_seventeen_track_cycle_and_four_special_tracks_preserve_retry_position() {
    let expected = [
        6, 7, 8, 10, 11, 4, 12, 13, 14, 15, 5, 16, 17, 18, 19, 20, 21,
    ];
    let mut music = DosMusic::default();
    for (i, raw) in expected.into_iter().enumerate() {
        assert_eq!(music.select(0, i != 0).unwrap(), raw - 1);
        assert_eq!(music.select(0, false).unwrap(), 5);
        assert_eq!(usize::from(music.cursor()), i);
    }
    assert_eq!(music.select(0, true).unwrap(), 5);
    for (special, raw) in [(1, 1), (2, 9), (3, 3), (4, 2)] {
        let before = music;
        assert_eq!(music.select(special, true).unwrap(), raw - 1);
        assert_eq!(music, before);
    }
    let before = music;
    assert!(music.select(5, true).is_err());
    assert_eq!(music, before);
}
