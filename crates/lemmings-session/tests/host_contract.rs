use lemmings_core::*;
use lemmings_session::{Session, pacing::Pacing};
use std::time::Duration;

fn session() -> Session {
    let (width, height) = (160, 160);
    let mut session = Session::new(
        StartupConfig {
            time_limit_minutes: 5,
            total_lemmings: 3,
            initial_release_rate: 99,
            entrance_count: 1,
            pause_glitch: false,
        },
        vec![EntrancePoint { x: 40, y: 40 }],
        width,
        height,
        (0..width * height)
            .map(|i| u8::from(i / width >= 100))
            .collect(),
    )
    .unwrap();
    session.configure_completion(2).unwrap();
    session
        .configure_digger(2, vec![0; width * height])
        .unwrap();
    session
        .configure_basher(
            2,
            [255; 160],
            vec![0; width * height],
            vec![0; width * height],
        )
        .unwrap();
    session.configure_miner(2, [255; 104]);
    session.configure_builder(2);
    let mut cells = vec![OBJECT_NONE; DOS_OBJECT_MAP_WIDTH * DOS_OBJECT_MAP_HEIGHT];
    let mut map = ObjectMapMut::new(
        DOS_OBJECT_MAP_WIDTH,
        DOS_OBJECT_MAP_HEIGHT,
        -16,
        -16,
        &mut cells,
    )
    .unwrap();
    for y in 96..104 {
        for x in 96..104 {
            map.write(x, y, OBJECT_EXIT);
        }
    }
    session.configure_blocker(2, cells).unwrap();
    session.configure_bomber(2, [255; 44]).unwrap();
    session.configure_climber(2);
    session.configure_floater(2);
    session
}

fn state(s: &Session) -> (Vec<String>, Vec<Vec<u8>>) {
    (
        vec![
            s.snapshot(),
            s.completion_snapshot(),
            s.digger_snapshot(),
            s.blocker_snapshot(),
            s.builder_snapshot(),
            s.bomber_snapshot(),
            s.climber_snapshot(),
            s.floater_snapshot(),
            s.hazards_snapshot(),
        ],
        vec![
            s.terrain().to_vec(),
            s.object_map_data().to_vec(),
            s.brick_mask().to_vec(),
        ],
    )
}

#[test]
fn elapsed_delivery_requires_explicit_ticks_and_preserves_frame_debt() {
    let mut s = session();
    s.set_pacing(Pacing::Dos);
    let initial = state(&s);
    let due = s.due(Duration::from_secs(1)).unwrap();
    assert_eq!(due, 14);
    assert_eq!(state(&s), initial);
    for _ in 0..due {
        s.tick().unwrap();
    }
    assert_eq!(s.scheduler.step_state().control_step, 14);
    assert_eq!(s.due(Duration::ZERO).unwrap(), 0);
    assert_eq!(s.due(Duration::from_secs(3599)).unwrap(), 50_448);
}

#[test]
fn pointer_requests_and_stable_id_commands_share_all_eight_skill_paths() {
    for (skill, direct) in [
        (AssignmentSkill::Climber, StartupCommand::AssignClimber(0)),
        (AssignmentSkill::Floater, StartupCommand::AssignFloater(0)),
        (AssignmentSkill::Bomber, StartupCommand::AssignBomber(0)),
        (AssignmentSkill::Blocker, StartupCommand::AssignBlocker(0)),
        (AssignmentSkill::Builder, StartupCommand::AssignBuilder(0)),
        (AssignmentSkill::Basher, StartupCommand::AssignBasher(0)),
        (AssignmentSkill::Miner, StartupCommand::AssignMiner(0)),
        (AssignmentSkill::Digger, StartupCommand::AssignDigger(0)),
    ] {
        let (mut pointer, mut id) = (session(), session());
        for _ in 0..80 {
            pointer.tick().unwrap();
            id.tick().unwrap();
        }
        assert_eq!(pointer.actors[0].state().action, MotionAction::Walking);
        for s in [&mut pointer, &mut id] {
            s.queue(InputCommand::Pause).unwrap();
        }
        pointer
            .queue(InputCommand::Custom(StartupCommand::AssignPointer(
                PointerAssignment {
                    skill,
                    primary: 0,
                    secondary: None,
                    right_click: false,
                },
            )))
            .unwrap();
        id.queue(InputCommand::Custom(direct)).unwrap();
        pointer.tick().unwrap();
        id.tick().unwrap();
        assert_eq!(state(&pointer), state(&id));
        if skill == AssignmentSkill::Climber {
            assert!(pointer.actors[0].state().is_climber);
        }
        for s in [&mut pointer, &mut id] {
            s.queue(InputCommand::Resume).unwrap();
        }
        for _ in 0..180 {
            pointer.tick().unwrap();
            id.tick().unwrap();
            assert_eq!(state(&pointer), state(&id));
        }
    }
}

#[test]
fn frozen_result_rejects_inputs_and_fresh_session_resets_state_and_clock() {
    let mut s = session();
    let initial = state(&s);
    s.queue(InputCommand::Custom(StartupCommand::Nuke)).unwrap();
    s.tick().unwrap();
    assert!(!s.result().unwrap().success);
    let terminal = state(&s);
    assert!(s.queue(InputCommand::Resume).is_err());
    for _ in 0..5 {
        s.tick().unwrap();
    }
    assert_eq!(state(&s), terminal);
    let mut fresh = session();
    assert_eq!(state(&fresh), initial);
    assert!(fresh.inputs.is_empty());
    assert_eq!(fresh.due(Duration::from_nanos(58_823_529)).unwrap(), 0);
    assert_eq!(fresh.due(Duration::from_nanos(1)).unwrap(), 1);
}
