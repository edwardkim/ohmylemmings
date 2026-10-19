use lemmings_core::*;
const W: usize = 64;
const H: usize = 170;
fn actor(action: MotionAction, direction: MotionDirection, x: i32, y: i32, frame: u8) -> Actor {
    let mut s = Actor::from_spawn(
        SpawnRequest {
            actor_id: 0,
            entrance_index: 0,
            iteration: 54,
        },
        &[EntrancePoint { x: 0, y: 0 }],
        MotionRules::default(),
    )
    .unwrap()
    .state();
    s.x = x;
    s.y = y;
    s.action = action;
    s.direction = direction;
    s.fallen = 0;
    s.frame = frame;
    s.bricks_left = if action == MotionAction::Building {
        12
    } else {
        0
    };
    s.is_climber = matches!(action, MotionAction::Climbing | MotionAction::Hoisting);
    if action == MotionAction::Splatting {
        s.direction = MotionDirection::Stopped;
    }
    Actor::from_state(s, MotionRules::default()).unwrap()
}
fn tick(a: &mut Actor, t: &[u8]) -> MotionOutcome {
    a.advance(TerrainMask::new(W, H, t).unwrap()).unwrap()
}
#[test]
fn permanent_assignment_preserves_each_action_and_consumes_only_once() {
    for action in [
        MotionAction::Walking,
        MotionAction::Jumping,
        MotionAction::Falling,
        MotionAction::Digging,
        MotionAction::Bashing,
        MotionAction::Mining,
        MotionAction::Building,
        MotionAction::OhNo,
    ] {
        let mut a = actor(action, MotionDirection::Right, 30, 100, 0);
        let before = a.state();
        let mut count = 2;
        assert!(a.assign_climber(&mut count));
        assert_eq!(count, 1);
        assert_eq!(
            a.state(),
            ActorState {
                is_climber: true,
                ..before
            }
        );
        assert!(!a.assign_climber(&mut count));
        assert_eq!(count, 1);
    }
    let mut a = actor(MotionAction::Walking, MotionDirection::Right, 30, 100, 0);
    assert!(!a.assign_climber(&mut 0));
    for action in [MotionAction::Exploding, MotionAction::Splatting] {
        let mut a = actor(action, MotionDirection::Right, 30, 100, 0);
        assert!(!a.assign_climber(&mut 2));
    }
}
#[test]
fn seven_pixel_wall_starts_climbing_without_turning_or_changing_height() {
    for direction in [MotionDirection::Left, MotionDirection::Right] {
        for height in 1..=8 {
            let mut terrain = vec![0; W * H];
            for y in 100 - height..=100 {
                terrain[y * W + 30] = 1;
            }
            let mut a = actor(
                MotionAction::Walking,
                direction,
                30 - direction.delta(),
                100,
                0,
            );
            a.assign_climber(&mut 1);
            tick(&mut a, &terrain);
            let s = a.state();
            assert!(s.is_climber);
            assert_eq!(s.direction, direction);
            assert_eq!(
                s.action,
                if height > 6 {
                    MotionAction::Climbing
                } else if height >= 3 {
                    MotionAction::Jumping
                } else {
                    MotionAction::Walking
                }
            );
            if height > 6 {
                assert_eq!((s.x, s.y, s.frame), (30, 100, 0));
            }
        }
    }
}
#[test]
fn climbing_cycle_checks_tip_then_moves_up_four_pixels() {
    let mut terrain = vec![0; W * H];
    for y in 0..110 {
        terrain[y * W + 30] = 1;
    }
    for direction in [MotionDirection::Left, MotionDirection::Right] {
        let mut a = actor(MotionAction::Climbing, direction, 30, 100, 0);
        for n in 1usize..=16 {
            let out = tick(&mut a, &terrain);
            assert!(out.check_objects);
            assert_eq!(a.state().action, MotionAction::Climbing);
            assert_eq!(
                a.state().y,
                100 - ((n / 8) * 4 + (n % 8).saturating_sub(3)) as i32
            );
        }
    }
}
#[test]
fn all_tip_frames_hoist_with_original_height_adjustment_and_clamp() {
    for frame in 0..4 {
        let mut a = actor(
            MotionAction::Climbing,
            MotionDirection::Right,
            30,
            100,
            (frame + 7) % 8,
        );
        tick(&mut a, &vec![0; W * H]);
        let s = a.state();
        assert_eq!(
            (s.action, s.y, s.frame),
            (MotionAction::Hoisting, 102 - i32::from(frame), 0)
        );
    }
    let mut a = actor(MotionAction::Climbing, MotionDirection::Right, 30, -10, 0);
    tick(&mut a, &vec![0; W * H]);
    assert_eq!(
        (a.state().action, a.state().y, a.state().direction),
        (MotionAction::Hoisting, 5, MotionDirection::Left)
    );
}
#[test]
fn hoisting_moves_on_first_four_ticks_and_checks_objects_at_finish() {
    let mut a = actor(MotionAction::Hoisting, MotionDirection::Right, 30, 100, 0);
    for n in 1..=8 {
        let out = tick(&mut a, &vec![0; W * H]);
        assert_eq!(a.state().y, 100 - 2 * n.min(4));
        assert_eq!(out.check_objects, n <= 4 || n == 8);
        assert_eq!(
            a.state().action,
            if n == 8 {
                MotionAction::Walking
            } else {
                MotionAction::Hoisting
            }
        );
    }
    assert!(a.state().is_climber);
}
#[test]
fn overhang_and_level_top_turn_then_move_two_pixels_into_falling() {
    for direction in [MotionDirection::Left, MotionDirection::Right] {
        for frame in 3..7 {
            let mut terrain = vec![0; W * H];
            terrain[91 * W + (30 - direction.delta()) as usize] = 1;
            let mut a = actor(MotionAction::Climbing, direction, 30, 100, frame);
            tick(&mut a, &terrain);
            let s = a.state();
            assert_eq!(
                (s.action, s.x, s.y, s.frame, s.fallen),
                (MotionAction::Falling, 30 - direction.delta() * 2, 99, 0, 3)
            );
            assert_eq!(s.direction.delta(), -direction.delta());
            assert!(s.is_climber);
        }
    }
    for y in [7, 8] {
        let mut a = actor(MotionAction::Climbing, MotionDirection::Right, 30, y, 3);
        tick(&mut a, &vec![0; W * H]);
        assert_eq!(
            a.state().action,
            if y == 7 {
                MotionAction::Falling
            } else {
                MotionAction::Climbing
            }
        );
    }
}
#[test]
fn shrugger_bug_retains_once_animation_until_turn_or_transition() {
    let terrain = vec![1; W * H];
    let mut a = actor(MotionAction::Shrugging, MotionDirection::Right, 30, 100, 6);
    assert!(a.assign_climber(&mut 1));
    assert_eq!(
        (a.state().action, a.state().frame, a.state().walking_once),
        (MotionAction::Walking, 6, true)
    );
    let mut flat = vec![0; W * H];
    flat[100 * W..101 * W].fill(1);
    tick(&mut a, &flat);
    assert_eq!((a.state().frame, a.state().end_of_animation), (7, false));
    tick(&mut a, &flat);
    assert_eq!((a.state().frame, a.state().end_of_animation), (7, true));
    tick(&mut a, &terrain);
    assert_eq!(a.state().action, MotionAction::Climbing);
    assert!(!a.state().walking_once);
    let mut s = actor(
        MotionAction::Shrugging,
        MotionDirection::Right,
        1647,
        100,
        6,
    )
    .state();
    let rules = MotionRules {
        assign_climber_shrugger_bug: false,
        ..MotionRules::default()
    };
    let mut a = Actor::from_state(s, rules).unwrap();
    a.assign_climber(&mut 1);
    assert_eq!(a.state().action, MotionAction::Shrugging);
    s.action = MotionAction::Walking;
    s.walking_once = true;
    assert!(Actor::from_state(s, MotionRules::default()).is_err());
}
#[test]
fn coordinate_overflow_preserves_complete_climber_record() {
    for (action, x, y, frame) in [
        (MotionAction::Climbing, 30, i32::MIN, 3),
        (MotionAction::Climbing, i32::MAX, 7, 3),
        (MotionAction::Hoisting, 30, i32::MIN, 0),
    ] {
        let mut a = actor(action, MotionDirection::Left, x, y, frame);
        let before = a.state();
        assert_eq!(
            a.advance(TerrainMask::new(W, H, &vec![0; W * H]).unwrap()),
            Err(MotionError::CoordinateOverflow)
        );
        assert_eq!(a.state(), before);
    }
}

#[test]
fn bomber_expiry_during_climbing_and_hoisting_retains_permanent_ability() {
    for action in [MotionAction::Climbing, MotionAction::Hoisting] {
        let a = actor(action, MotionDirection::Right, 30, 100, 3);
        let mut state = a.state();
        state.explosion_timer = 1;
        let mut a = Actor::from_state(state, MotionRules::default()).unwrap();
        let mut t = vec![1; W * H];
        let mut m = vec![128; 24 * 48];
        let empty = vec![0; W * H];
        let out = a
            .advance_with_bomber(
                &mut TerrainMaskMut::new(W, H, &mut t).unwrap(),
                BomberContext::new(
                    MiningContext::new(
                        &[255; 104],
                        BashingContext::new(&[255; 160], TerrainMask::new(W, H, &empty).unwrap()),
                    ),
                    &[255; 44],
                ),
                &mut ObjectMapMut::new(24, 48, -16, -16, &mut m).unwrap(),
            )
            .unwrap();
        assert!(!out.check_objects);
        let s = a.state();
        assert_eq!(
            (s.action, s.x, s.y, s.frame, s.explosion_timer),
            (MotionAction::OhNo, 30, 100, 0, 0)
        );
        assert!(s.is_climber);
    }
}
#[test]
fn blocker_and_removed_actors_refuse_ability_without_consuming_inventory() {
    let mut a = actor(MotionAction::Walking, MotionDirection::Right, 30, 100, 0);
    let mut m = vec![128; 24 * 48];
    assert!(a.assign_blocker(
        &mut 1,
        &mut ObjectMapMut::new(24, 48, -16, -16, &mut m).unwrap()
    ));
    let before = a.state();
    let mut count = 2;
    assert!(!a.assign_climber(&mut count));
    assert_eq!(count, 2);
    assert_eq!(a.state(), before);
    let mut s = actor(MotionAction::Walking, MotionDirection::Right, 30, 164, 0).state();
    s.removed = Some(RemovalReason::BottomBoundary);
    let mut a = Actor::from_state(s, MotionRules::default()).unwrap();
    assert!(!a.assign_climber(&mut count));
    assert_eq!(count, 2);
}
