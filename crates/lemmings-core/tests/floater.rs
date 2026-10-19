use lemmings_core::*;
const W: usize = 64;
const H: usize = 170;
fn actor(action: MotionAction, direction: MotionDirection, x: i32, y: i32) -> Actor {
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
    s.frame = 0;
    s.fallen = 0;
    s.is_climber = matches!(action, MotionAction::Climbing | MotionAction::Hoisting);
    s.is_floater = action == MotionAction::Floating;
    s.bricks_left = if action == MotionAction::Building {
        12
    } else {
        0
    };
    if action == MotionAction::Blocking {
        s.blocker_saved = Some([128; 9]);
    }
    if action == MotionAction::Splatting {
        s.direction = MotionDirection::Stopped;
    }
    Actor::from_state(s, MotionRules::default()).unwrap()
}
fn tick(a: &mut Actor, t: &[u8]) -> MotionOutcome {
    a.advance(TerrainMask::new(W, H, t).unwrap()).unwrap()
}
#[test]
fn permanent_assignment_preserves_action_animation_climber_and_bomber_timer() {
    for action in [
        MotionAction::Walking,
        MotionAction::Jumping,
        MotionAction::Falling,
        MotionAction::Digging,
        MotionAction::Bashing,
        MotionAction::Mining,
        MotionAction::Building,
        MotionAction::Shrugging,
        MotionAction::Climbing,
        MotionAction::Hoisting,
        MotionAction::OhNo,
    ] {
        let mut a = actor(action, MotionDirection::Right, 30, 50);
        let before = a.state();
        let mut count = 2;
        assert!(a.assign_floater(&mut count));
        assert_eq!(count, 1);
        assert_eq!(
            a.state(),
            ActorState {
                is_floater: true,
                ..before
            }
        );
        assert!(!a.assign_floater(&mut count));
        assert_eq!(count, 1);
    }
    let mut a = actor(MotionAction::Shrugging, MotionDirection::Right, 30, 50);
    assert!(a.assign_climber(&mut 1));
    assert!(a.assign_bomber(&mut 1));
    let before = a.state();
    assert!(a.assign_floater(&mut 1));
    assert_eq!(
        a.state(),
        ActorState {
            is_floater: true,
            ..before
        }
    );
}
#[test]
fn blocked_splatting_exploding_removed_and_empty_inventory_refuse_assignment() {
    for action in [
        MotionAction::Blocking,
        MotionAction::Splatting,
        MotionAction::Exploding,
    ] {
        let mut a = actor(action, MotionDirection::Right, 30, 50);
        let before = a.state();
        let mut n = 2;
        assert!(!a.assign_floater(&mut n));
        assert_eq!(n, 2);
        assert_eq!(a.state(), before);
    }
    let mut a = actor(MotionAction::Walking, MotionDirection::Right, 30, 50);
    assert!(!a.assign_floater(&mut 0));
    let mut s = a.state();
    s.y = 164;
    s.removed = Some(RemovalReason::BottomBoundary);
    a = Actor::from_state(s, MotionRules::default()).unwrap();
    assert!(!a.assign_floater(&mut 2));
}
#[test]
fn opening_threshold_checks_previous_fall_counter_before_moving_or_landing() {
    let empty = vec![0; W * H];
    let mut a = actor(MotionAction::Falling, MotionDirection::Right, 30, 40);
    a.assign_floater(&mut 1);
    let mut s = a.state();
    s.fallen = 15;
    a = Actor::from_state(s, MotionRules::default()).unwrap();
    tick(&mut a, &empty);
    assert_eq!(
        (a.state().action, a.state().fallen, a.state().y),
        (MotionAction::Falling, 18, 43)
    );
    tick(&mut a, &empty);
    assert_eq!(
        (
            a.state().action,
            a.state().frame,
            a.state().float_index,
            a.state().fallen,
            a.state().y
        ),
        (MotionAction::Floating, 0, 0, 0, 43)
    );
    tick(&mut a, &empty);
    assert_eq!(
        (a.state().frame, a.state().float_index, a.state().y),
        (1, 1, 46)
    );
    let mut ground = empty;
    ground[50 * W + 30] = 1;
    for fallen in [16, 17, 61, u32::MAX] {
        let mut s = actor(MotionAction::Falling, MotionDirection::Left, 30, 50).state();
        s.is_floater = true;
        s.fallen = fallen;
        let mut a = Actor::from_state(s, MotionRules::default()).unwrap();
        tick(&mut a, &ground);
        assert_eq!(
            a.state().action,
            if fallen > 16 {
                MotionAction::Floating
            } else {
                MotionAction::Walking
            }
        );
        assert_eq!(a.state().y, 50);
    }
}
#[test]
fn full_opening_table_then_eight_stage_loop_preserves_both_directions() {
    let table = [
        (3, 1),
        (3, 2),
        (3, 3),
        (3, 5),
        (-1, 5),
        (0, 5),
        (1, 5),
        (1, 5),
        (2, 5),
        (2, 6),
        (2, 7),
        (2, 7),
        (2, 6),
        (2, 5),
        (2, 4),
        (2, 4),
    ];
    for direction in [MotionDirection::Left, MotionDirection::Right] {
        let mut a = actor(MotionAction::Floating, direction, 30, 20);
        let mut index = 0;
        let mut y = 20;
        for _ in 0..40 {
            let (dy, frame) = table[index];
            y += dy;
            index += 1;
            if index == 16 {
                index = 8;
            }
            let out = tick(&mut a, &vec![0; W * H]);
            let s = a.state();
            assert!(out.check_objects);
            assert_eq!(
                (
                    s.x,
                    s.y,
                    s.frame,
                    s.float_index,
                    s.direction,
                    s.end_of_animation
                ),
                (30, y, frame, index as u8, direction, false)
            );
        }
    }
}
#[test]
fn positive_stages_land_at_each_checked_pixel_before_remaining_movement() {
    for index in [0, 1, 2, 3, 6, 7, 8, 15] {
        let dy = if index <= 3 {
            3
        } else if index <= 7 {
            1
        } else {
            2
        };
        for offset in 0..dy {
            let mut t = vec![0; W * H];
            t[(40 + offset) * W + 30] = 1;
            let mut s = actor(MotionAction::Floating, MotionDirection::Right, 30, 40).state();
            s.float_index = index;
            let mut a = Actor::from_state(s, MotionRules::default()).unwrap();
            let out = tick(&mut a, &t);
            let s = a.state();
            assert_eq!(
                (s.action, s.y, s.frame, s.fallen),
                (MotionAction::Walking, 40 + offset as i32, 0, 0)
            );
            assert!(s.is_floater);
            assert!(out.check_objects);
            assert_eq!(s.float_index, if index == 15 { 8 } else { index + 1 });
        }
    }
}
#[test]
fn upward_and_stationary_stages_ignore_terrain_and_negative_y_is_clamped_per_probe() {
    for (index, dy) in [(4, -1), (5, 0)] {
        let mut s = actor(MotionAction::Floating, MotionDirection::Right, 30, 40).state();
        s.float_index = index;
        let mut a = Actor::from_state(s, MotionRules::default()).unwrap();
        tick(&mut a, &vec![1; W * H]);
        assert_eq!(
            (a.state().action, a.state().y, a.state().frame),
            (MotionAction::Floating, 40 + dy, 5)
        );
    }
    let mut t = vec![0; W * H];
    t[30] = 1;
    let mut a = actor(MotionAction::Floating, MotionDirection::Left, 30, -1);
    tick(&mut a, &t);
    assert_eq!((a.state().action, a.state().y), (MotionAction::Walking, -1));
}
#[test]
fn bottom_is_checked_after_whole_float_stage_but_landing_returns_first() {
    let mut a = actor(MotionAction::Floating, MotionDirection::Right, 30, 162);
    let out = tick(&mut a, &vec![0; W * H]);
    assert_eq!(a.state().y, 165);
    assert_eq!(out.removal, Some(RemovalReason::BottomBoundary));
    assert!(!out.check_objects);
    assert!(Actor::from_state(a.state(), a.rules()).is_ok());
    let mut t = vec![0; W * H];
    t[164 * W + 30] = 1;
    let mut a = actor(MotionAction::Floating, MotionDirection::Right, 30, 163);
    let out = tick(&mut a, &t);
    assert_eq!(
        (a.state().action, a.state().y, a.state().removed),
        (MotionAction::Walking, 164, None)
    );
    assert!(out.check_objects);
}
#[test]
fn invalid_ability_table_index_and_overflow_preserve_valid_records() {
    let s = actor(MotionAction::Floating, MotionDirection::Right, 30, 40).state();
    for bad in [
        ActorState {
            is_floater: false,
            ..s
        },
        ActorState {
            float_index: 16,
            ..s
        },
        ActorState { frame: 8, ..s },
    ] {
        assert!(Actor::from_state(bad, MotionRules::default()).is_err());
    }
    for (index, y) in [(0, i32::MAX), (4, i32::MIN)] {
        let mut s = s;
        s.float_index = index;
        s.y = y;
        let mut a = Actor::from_state(s, MotionRules::default()).unwrap();
        assert_eq!(
            a.advance(TerrainMask::new(W, H, &vec![0; W * H]).unwrap()),
            Err(MotionError::CoordinateOverflow)
        );
        assert_eq!(a.state(), s);
    }
}
#[test]
fn bomber_expiry_while_floating_skips_ohno_and_preserves_permanent_abilities() {
    let mut s = actor(MotionAction::Floating, MotionDirection::Left, 30, 50).state();
    s.is_climber = true;
    s.explosion_timer = 1;
    s.float_index = 9;
    let mut a = Actor::from_state(s, MotionRules::default()).unwrap();
    let mut t = vec![0; W * H];
    let empty = t.clone();
    let mut m = vec![128; 24 * 48];
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
    let s = a.state();
    assert_eq!(
        (s.action, s.frame, s.float_index, s.explosion_timer, s.y),
        (MotionAction::Exploding, 0, 9, 0, 50)
    );
    assert!(s.is_floater && s.is_climber);
    assert!(!out.check_objects);
}

#[test]
fn batch_preflight_keeps_all_floater_records_atomic_on_order_and_overflow_errors() {
    let a = actor(MotionAction::Floating, MotionDirection::Right, 30, 40);
    let mut s = a.state();
    s.id = 1;
    s.y = i32::MAX;
    let b = Actor::from_state(s, MotionRules::default()).unwrap();
    let mut actors = [a, b];
    let before = actors.map(|a| a.state());
    let mut emitted = 0;
    let t = vec![0; W * H];
    assert_eq!(
        step_actors(&mut actors, TerrainMask::new(W, H, &t).unwrap(), |_, _| {
            emitted += 1
        }),
        Err(MotionError::CoordinateOverflow)
    );
    assert_eq!(actors.map(|a| a.state()), before);
    assert_eq!(emitted, 0);
    s.id = 0;
    s.y = 40;
    actors[1] = Actor::from_state(s, MotionRules::default()).unwrap();
    assert_eq!(
        step_actors(&mut actors, TerrainMask::new(W, H, &t).unwrap(), |_, _| {
            emitted += 1
        }),
        Err(MotionError::ActorOrder { index: 1 })
    );
    assert_eq!(emitted, 0);
}
