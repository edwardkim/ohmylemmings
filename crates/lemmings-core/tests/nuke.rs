use lemmings_core::*;

fn config(total: u32) -> StartupConfig {
    StartupConfig {
        time_limit_minutes: 5,
        total_lemmings: total,
        initial_release_rate: 99,
        entrance_count: 1,
        pause_glitch: true,
    }
}
fn setup(count: u32) -> (StartupScheduler, Vec<Actor>) {
    let mut s = StartupScheduler::new(config(20)).unwrap();
    let mut actors = Vec::new();
    while s.state().released < count {
        s.step(&InputSchedule::new(&[]).unwrap(), |e| {
            if let StartupEvent::Spawn(r) = e {
                actors.push(
                    Actor::from_spawn(r, &[EntrancePoint { x: 0, y: 0 }], MotionRules::default())
                        .unwrap(),
                );
            }
        })
        .unwrap();
    }
    (s, actors)
}
fn commands(s: &mut StartupScheduler, commands: &[InputCommand<StartupCommand>]) -> Vec<bool> {
    let inputs: Vec<_> = commands
        .iter()
        .enumerate()
        .map(|(i, command)| TimedInput {
            control_step: s.step_state().control_step,
            sequence: i as u32,
            command: match command {
                InputCommand::Pause => InputCommand::Pause,
                InputCommand::Resume => InputCommand::Resume,
                InputCommand::Custom(c) => InputCommand::Custom(*c),
            },
        })
        .collect();
    let mut accepted = Vec::new();
    s.step(&InputSchedule::new(&inputs).unwrap(), |e| {
        if let StartupEvent::Input { accepted: a, .. } = e {
            accepted.push(a)
        }
    })
    .unwrap();
    accepted
}
const NUKE: InputCommand<StartupCommand> = InputCommand::Custom(StartupCommand::Nuke);

#[test]
fn nuke_is_one_shot_stops_birth_countdown_and_snapshots_released_maximum() {
    let (mut s, mut actors) = setup(3);
    let before = s.state();
    assert_eq!(commands(&mut s, &[NUKE, NUKE]), [true, false]);
    assert!(s.nuke_state().is_nuked && s.nuke_state().assigning);
    assert_eq!(s.nuke_state().max_lemmings, 3);
    assert_eq!(s.state().spawn_countdown, before.spawn_countdown);
    for _ in 0..100 {
        s.step(&InputSchedule::new(&[]).unwrap(), |_| {}).unwrap();
    }
    assert_eq!(s.state().released, 3);
    assert_eq!(s.update_nuking(&mut actors).unwrap(), Some(0));
    assert_eq!(actors[0].state().explosion_timer, 79);
    assert_eq!(s.nuke_state().next_index, 1);
    assert_eq!(s.update_nuking(&mut actors).unwrap(), None);
    assert_eq!(s.nuke_state().next_index, 1);
}
#[test]
fn paused_nuke_is_rejected_and_same_boundary_resume_then_nuke_is_accepted() {
    let (mut s, mut actors) = setup(1);
    assert_eq!(
        commands(&mut s, &[InputCommand::Pause, NUKE]),
        [true, false]
    );
    assert!(!s.nuke_state().is_nuked);
    assert_eq!(
        commands(&mut s, &[InputCommand::Resume, NUKE]),
        [true, true]
    );
    assert_eq!(s.update_nuking(&mut actors).unwrap(), Some(0));
    let before = s.nuke_state();
    commands(&mut s, &[InputCommand::Pause]);
    assert_eq!(s.update_nuking(&mut actors).unwrap(), None);
    assert_eq!(s.nuke_state(), before);
    commands(&mut s, &[InputCommand::Resume]);
    assert_eq!(s.update_nuking(&mut actors).unwrap(), None);
    assert!(!s.nuke_state().assigning);
}
#[test]
fn nuke_before_first_birth_preserves_startup_markers_and_finishes_empty_scan() {
    let mut s = StartupScheduler::new(config(20)).unwrap();
    commands(&mut s, &[NUKE]);
    assert_eq!(s.nuke_state().max_lemmings, 0);
    s.update_nuking(&mut []).unwrap();
    assert!(!s.nuke_state().assigning);
    for _ in 0..100 {
        s.step(&InputSchedule::new(&[]).unwrap(), |_| {}).unwrap();
    }
    assert!(s.state().entrances_open);
    assert_eq!((s.state().released, s.state().spawn_countdown), (0, 20));
}
#[test]
fn disabling_nuke_glitch_retains_original_maximum_and_still_stops_births() {
    let mut s = StartupScheduler::new_with_nuke_glitch(config(20), false).unwrap();
    commands(&mut s, &[NUKE]);
    assert_eq!(s.nuke_state().max_lemmings, 20);
    for _ in 0..100 {
        s.step(&InputSchedule::new(&[]).unwrap(), |_| {}).unwrap();
    }
    assert_eq!(s.state().released, 0);
}
#[test]
fn removed_records_skip_without_spending_a_live_slot_and_last_slot_finishes_next_tick() {
    let (mut s, mut actors) = setup(3);
    let mut a = actors[0].state();
    a.y = 164;
    a.removed = Some(RemovalReason::BottomBoundary);
    actors[0] = Actor::from_state(a, MotionRules::default()).unwrap();
    commands(&mut s, &[NUKE]);
    assert_eq!(s.update_nuking(&mut actors).unwrap(), Some(1));
    assert_eq!(actors[0].state().explosion_timer, 0);
    assert_eq!(s.nuke_state().next_index, 2);
    commands(&mut s, &[]);
    assert_eq!(s.update_nuking(&mut actors).unwrap(), Some(2));
    assert!(s.nuke_state().assigning);
    commands(&mut s, &[]);
    assert_eq!(s.update_nuking(&mut actors).unwrap(), None);
    assert!(!s.nuke_state().assigning);
}
#[test]
fn running_timer_and_splatting_exploding_consume_their_order_without_rearming() {
    let (mut s, mut actors) = setup(3);
    actors[0].assign_bomber(&mut 1);
    let mut a = actors[1].state();
    a.action = MotionAction::Splatting;
    a.direction = MotionDirection::Stopped;
    a.fallen = 0;
    actors[1] = Actor::from_state(a, MotionRules::default()).unwrap();
    let mut a = actors[2].state();
    a.action = MotionAction::Exploding;
    a.fallen = 0;
    actors[2] = Actor::from_state(a, MotionRules::default()).unwrap();
    let before = actors.clone();
    commands(&mut s, &[NUKE]);
    for _ in 0..3 {
        assert_eq!(s.update_nuking(&mut actors).unwrap(), None);
        commands(&mut s, &[]);
    }
    assert_eq!(actors, before);
    assert_eq!(s.nuke_state().next_index, 3);
}
#[test]
fn missing_or_reordered_actor_list_preserves_timer_and_scan_cursor() {
    let (mut s, mut actors) = setup(3);
    commands(&mut s, &[NUKE]);
    let before = s.nuke_state();
    let states = actors.clone();
    assert!(s.update_nuking(&mut actors[..2]).is_err());
    assert_eq!(s.nuke_state(), before);
    assert_eq!(actors, states);
    actors.swap(1, 2);
    let bad = actors.clone();
    assert!(s.update_nuking(&mut actors).is_err());
    assert_eq!(s.nuke_state(), before);
    assert_eq!(actors, bad);
    actors.swap(1, 2);
    assert_eq!(s.update_nuking(&mut actors).unwrap(), Some(0));
}

#[test]
fn dos_removed_actor_with_timer_consumes_a_scan_slot_before_the_removed_skip() {
    let (mut s, mut actors) = setup(3);
    let mut a = actors[0].state();
    a.y = 164;
    a.removed = Some(RemovalReason::BottomBoundary);
    a.explosion_timer = 20;
    actors[0] = Actor::from_state(a, MotionRules::default()).unwrap();
    commands(&mut s, &[NUKE]);
    assert_eq!(s.update_nuking(&mut actors).unwrap(), None);
    assert_eq!(s.nuke_state().next_index, 1);
    assert_eq!(actors[1].state().explosion_timer, 0);
    commands(&mut s, &[]);
    assert_eq!(s.update_nuking(&mut actors).unwrap(), Some(1));
    assert_eq!(actors[0].state().explosion_timer, 20);
}
#[test]
fn nuke_arms_blocker_and_permanent_abilities_without_consuming_skill_inventory() {
    let (mut s, mut actors) = setup(1);
    let mut a = actors[0].state();
    a.action = MotionAction::Blocking;
    a.fallen = 0;
    a.blocker_saved = Some([128; 9]);
    a.is_climber = true;
    a.is_floater = true;
    actors[0] = Actor::from_state(a, MotionRules::default()).unwrap();
    let before = actors[0].state();
    commands(&mut s, &[NUKE]);
    s.update_nuking(&mut actors).unwrap();
    assert_eq!(
        actors[0].state(),
        ActorState {
            explosion_timer: 79,
            ..before
        }
    );
    assert!(Actor::from_state(actors[0].state(), MotionRules::default()).is_ok());
}
#[test]
fn nuke_rearms_ohno_and_its_timer_survives_exploding_and_removed_restore() {
    let (mut s, mut actors) = setup(1);
    let mut a = actors[0].state();
    a.action = MotionAction::OhNo;
    a.frame = 15;
    a.fallen = 0;
    a.y = 50;
    actors[0] = Actor::from_state(a, MotionRules::default()).unwrap();
    commands(&mut s, &[NUKE]);
    s.update_nuking(&mut actors).unwrap();
    assert_eq!(actors[0].state().explosion_timer, 79);
    let mut t = vec![1; 64 * 170];
    let empty = vec![0; 64 * 170];
    let mut map = vec![128; 24 * 48];
    for expected in [MotionAction::Exploding, MotionAction::Exploding] {
        assert!(Actor::from_state(actors[0].state(), MotionRules::default()).is_ok());
        actors[0]
            .advance_with_bomber(
                &mut TerrainMaskMut::new(64, 170, &mut t).unwrap(),
                BomberContext::new(
                    MiningContext::new(
                        &[255; 104],
                        BashingContext::new(
                            &[255; 160],
                            TerrainMask::new(64, 170, &empty).unwrap(),
                        ),
                    ),
                    &[255; 44],
                ),
                &mut ObjectMapMut::new(24, 48, -16, -16, &mut map).unwrap(),
            )
            .unwrap();
        assert_eq!(actors[0].state().action, expected);
        assert!(Actor::from_state(actors[0].state(), MotionRules::default()).is_ok());
    }
    assert_eq!(actors[0].state().removed, Some(RemovalReason::Exploded));
    assert_eq!(actors[0].state().explosion_timer, 77);
}
