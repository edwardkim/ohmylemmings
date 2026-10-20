use lemmings_core::*;
use lemmings_session::Session;

fn session() -> Session {
    Session::new(
        StartupConfig {
            time_limit_minutes: 5,
            total_lemmings: 1,
            initial_release_rate: 50,
            entrance_count: 1,
            pause_glitch: true,
        },
        vec![EntrancePoint { x: 16, y: 86 }],
        256,
        192,
        vec![0; 256 * 192],
    )
    .unwrap()
}
#[test]
fn original_startup_sound_phase_is_fifteen_thirty_five_fifty_five_and_pause_delays_it() {
    let mut a = session();
    let mut got = Vec::new();
    for boundary in 0..70 {
        if boundary == 10 {
            a.queue(InputCommand::Pause).unwrap();
        }
        if boundary == 20 {
            a.queue(InputCommand::Resume).unwrap();
        }
        a.tick().unwrap();
        got.extend(
            a.sounds()
                .iter()
                .map(|s| (s.control_step, s.iteration, s.cue)),
        );
    }
    assert_eq!(
        got,
        [
            (24, 15, SoundCue::LetsGo),
            (44, 35, SoundCue::EntranceOpening),
            (64, 55, SoundCue::MusicStart)
        ]
    );
}
#[test]
fn accepted_assignments_and_paused_ui_cues_keep_order_without_changing_legacy_snapshots() {
    let mut a = session();
    let mut b = session();
    let mut state = Actor::from_spawn(
        SpawnRequest {
            actor_id: 0,
            entrance_index: 0,
            iteration: 54,
        },
        &[EntrancePoint { x: 16, y: 86 }],
        MotionRules::default(),
    )
    .unwrap()
    .state();
    state.action = MotionAction::Walking;
    state.fallen = 0;
    for session in [&mut a, &mut b] {
        session
            .actors
            .push(Actor::from_state(state, MotionRules::default()).unwrap());
        session.configure_climber(2);
        session.queue(InputCommand::Pause).unwrap();
        session
            .queue(InputCommand::Custom(StartupCommand::AssignClimber(0)))
            .unwrap();
        session
            .queue(InputCommand::Custom(StartupCommand::AssignClimber(0)))
            .unwrap();
    }
    a.notify_skill_selection();
    a.tick().unwrap();
    b.tick().unwrap();
    assert_eq!(a.snapshot(), b.snapshot());
    assert_eq!(
        a.sounds().iter().map(|s| s.cue).collect::<Vec<_>>(),
        [
            SoundCue::SkillButtonSelect,
            SoundCue::AssignSkill(AssignmentSkill::Climber)
        ]
    );
    assert_eq!(a.sounds()[1].actor_id, Some(0));
    assert_eq!(a.sounds()[1].input_sequence, Some(1));
    assert_eq!(a.sounds()[1].iteration, 0);
    a.tick().unwrap();
    assert!(a.sounds().is_empty());
}
#[test]
fn consuming_or_ignoring_sound_frames_has_identical_states_and_terrain() {
    let mut a = session();
    let mut b = session();
    for _ in 0..200 {
        a.tick().unwrap();
        b.tick().unwrap();
        let _ = a.sounds().iter().count();
        assert_eq!(a.snapshot(), b.snapshot());
        assert_eq!(a.terrain(), b.terrain());
    }
}

#[test]
fn ui_burst_is_bounded_and_reports_loss_without_consuming_game_inputs() {
    let mut a = session();
    let mut b = session();
    a.queue(InputCommand::Pause).unwrap();
    b.queue(InputCommand::Pause).unwrap();
    for _ in 0..1100 {
        a.notify_skill_selection();
    }
    a.tick().unwrap();
    b.tick().unwrap();
    assert_eq!(a.sounds().len(), 1024);
    assert_eq!(a.sound_dropped(), 76);
    assert_eq!(a.snapshot(), b.snapshot());
    a.tick().unwrap();
    assert_eq!(a.sound_dropped(), 0);
    assert!(a.sounds().is_empty());
}
