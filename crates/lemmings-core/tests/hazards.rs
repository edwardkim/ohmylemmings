use lemmings_core::*;
const W: usize = 64;
const H: usize = 170;
fn actor(action: MotionAction, id: u32, x: i32, y: i32) -> Actor {
    let mut s = Actor::from_spawn(
        SpawnRequest {
            actor_id: id,
            entrance_index: 0,
            iteration: 54,
        },
        &[EntrancePoint { x: 8, y: 86 }],
        MotionRules::default(),
    )
    .unwrap()
    .state();
    s.action = action;
    s.x = x;
    s.y = y;
    s.fallen = 0;
    s.is_floater = action == MotionAction::Floating;
    Actor::from_state(s, MotionRules::default()).unwrap()
}
fn floor() -> Vec<u8> {
    (0..W * H).map(|i| u8::from(i / W >= 100)).collect()
}
fn put(map: &mut [u8], x: i64, y: i64, code: u8) {
    ObjectMapMut::new(20, 48, -16, -16, map)
        .unwrap()
        .write(x, y, code);
}
fn tick(
    actors: &mut [Actor],
    terrain: &mut [u8],
    map: &mut [u8],
    traps: &mut TrapStates,
) -> Result<ActorTick, MotionError> {
    let blank = vec![0; W * H];
    step_actors_with_hazards(
        actors,
        &mut TerrainMaskMut::new(W, H, terrain).unwrap(),
        BomberContext::new(
            MiningContext::new(
                &[0; 104],
                BashingContext::new(&[0; 160], TerrainMask::new(W, H, &blank).unwrap()),
            ),
            &[0; 44],
        ),
        &mut ObjectMapMut::new(20, 48, -16, -16, map).unwrap(),
        traps,
        |_, _| {},
    )
}
#[test]
fn water_and_fire_enter_once_and_remove_after_their_complete_animation() {
    for (code, action, frames, reason) in [
        (
            OBJECT_WATER,
            MotionAction::Drowning,
            16,
            RemovalReason::Drowned,
        ),
        (
            OBJECT_FIRE,
            MotionAction::Vaporizing,
            14,
            RemovalReason::Vaporized,
        ),
    ] {
        for start in [
            MotionAction::Walking,
            MotionAction::Falling,
            MotionAction::Floating,
        ] {
            let mut actors = [actor(start, 0, 32, 100)];
            let mut t = floor();
            let before = t.clone();
            let mut map = vec![128; 20 * 48];
            let mut traps = [None; 16];
            put(&mut map, 33, 100, code);
            tick(&mut actors, &mut t, &mut map, &mut traps).unwrap();
            assert_eq!(
                (actors[0].state().action, actors[0].state().frame),
                (action, 0)
            );
            for _ in 0..frames - 1 {
                assert_eq!(
                    tick(&mut actors, &mut t, &mut map, &mut traps)
                        .unwrap()
                        .removed,
                    0
                );
            }
            let out = tick(&mut actors, &mut t, &mut map, &mut traps).unwrap();
            assert_eq!((out.removed, out.rescued), (1, 0));
            assert_eq!(actors[0].state().removed, Some(reason));
            Actor::from_state(actors[0].state(), MotionRules::default()).unwrap();
            assert_eq!(
                tick(&mut actors, &mut t, &mut map, &mut traps)
                    .unwrap()
                    .removed,
                0
            );
            assert_eq!(t, before);
        }
    }
}
#[test]
fn drowning_moves_toward_empty_probe_without_clipping_y_or_turning_at_world_edge() {
    let mut t = vec![0; W * H];
    t[0] = 1;
    for (x, y, direction, wanted) in [
        (32, 100, MotionDirection::Right, 33),
        (32, 100, MotionDirection::Left, 31),
        (-8, -1, MotionDirection::Right, -7),
        (1647, 100, MotionDirection::Right, 1648),
    ] {
        let mut s = actor(MotionAction::Drowning, 0, x, y).state();
        s.direction = direction;
        let mut a = Actor::from_state(s, MotionRules::default()).unwrap();
        a.advance(TerrainMask::new(W, H, &t).unwrap()).unwrap();
        assert_eq!(a.state().x, wanted);
        assert_eq!(a.state().direction, direction);
    }
    t[100 * W + 40] = 1;
    let mut a = actor(MotionAction::Drowning, 0, 32, 100);
    a.advance(TerrainMask::new(W, H, &t).unwrap()).unwrap();
    assert_eq!(a.state().x, 32);
}
#[test]
fn terminal_hazards_do_not_recheck_exit_or_traps() {
    for action in [MotionAction::Drowning, MotionAction::Vaporizing] {
        let mut a = [actor(action, 0, 32, 100)];
        let mut t = floor();
        let mut map = vec![128; 20 * 48];
        let mut traps = [None; 16];
        put(&mut map, 32, 100, OBJECT_EXIT);
        put(&mut map, 36, 100, 0);
        traps[0] = Some(TrapState::new(3, 0).unwrap());
        for _ in 0..16 {
            tick(&mut a, &mut t, &mut map, &mut traps).unwrap();
        }
        assert_eq!(a[0].state().action, action);
        assert!(!traps[0].unwrap().triggered());
    }
}
#[test]
fn existing_timer_survives_hazards_but_expiry_precedes_the_hazard_check() {
    for code in [OBJECT_WATER, OBJECT_FIRE] {
        for timer in [1, 79] {
            let mut s = actor(MotionAction::Walking, 0, 32, 100).state();
            s.explosion_timer = timer;
            let mut a = [Actor::from_state(s, MotionRules::default()).unwrap()];
            let mut t = floor();
            let mut map = vec![128; 20 * 48];
            let mut traps = [None; 16];
            put(&mut map, 33, 100, code);
            tick(&mut a, &mut t, &mut map, &mut traps).unwrap();
            assert_eq!(a[0].state().explosion_timer, timer - 1);
            assert_eq!(
                a[0].state().action,
                if timer == 1 {
                    MotionAction::OhNo
                } else if code == OBJECT_WATER {
                    MotionAction::Drowning
                } else {
                    MotionAction::Vaporizing
                }
            );
        }
    }
}
#[test]
fn fatal_landing_hazard_keeps_stopped_direction_and_cancelled_timer() {
    for code in [OBJECT_WATER, OBJECT_FIRE] {
        let mut s = actor(MotionAction::Falling, 0, 32, 100).state();
        s.fallen = 61;
        s.explosion_timer = 42;
        let mut a = [Actor::from_state(s, MotionRules::default()).unwrap()];
        let mut t = floor();
        let mut map = vec![128; 20 * 48];
        let mut traps = [None; 16];
        put(&mut map, 32, 100, code);
        tick(&mut a, &mut t, &mut map, &mut traps).unwrap();
        assert_eq!(
            (a[0].state().direction, a[0].state().explosion_timer),
            (MotionDirection::Stopped, 0)
        );
        Actor::from_state(a[0].state(), MotionRules::default()).unwrap();
    }
}
#[test]
fn same_tick_trap_contention_removes_only_first_actor_then_rearms_after_last_frame() {
    let mut a = [
        actor(MotionAction::Walking, 0, 32, 100),
        actor(MotionAction::Walking, 1, 32, 100),
    ];
    let mut t = floor();
    let mut map = vec![128; 20 * 48];
    let mut traps = [None; 16];
    put(&mut map, 33, 100, 15);
    traps[15] = Some(TrapState::new(3, 2).unwrap());
    let out = tick(&mut a, &mut t, &mut map, &mut traps).unwrap();
    assert_eq!((out.removed, out.rescued), (1, 0));
    assert_eq!(a[0].state().removed, Some(RemovalReason::Trapped));
    assert_eq!(a[1].state().removed, None);
    assert_eq!(
        (traps[15].unwrap().frame(), traps[15].unwrap().triggered()),
        (0, true)
    );
    for frame in [1, 2, 0] {
        advance_traps(&mut traps);
        assert_eq!(traps[15].unwrap().frame(), frame);
    }
    assert!(!traps[15].unwrap().triggered());
    tick(&mut a, &mut t, &mut map, &mut traps).unwrap();
    assert_eq!(a[1].state().removed, Some(RemovalReason::Trapped));
    Actor::from_state(a[0].state(), MotionRules::default()).unwrap();
}
#[test]
fn one_frame_trap_is_busy_for_whole_actor_batch() {
    let mut a = [
        actor(MotionAction::Falling, 0, 32, 50),
        actor(MotionAction::Falling, 1, 32, 50),
    ];
    let mut t = floor();
    let mut map = vec![128; 20 * 48];
    let mut traps = [None; 16];
    put(&mut map, 32, 52, 0);
    traps[0] = Some(TrapState::new(1, 0).unwrap());
    assert_eq!(
        tick(&mut a, &mut t, &mut map, &mut traps).unwrap().removed,
        1
    );
    assert!(traps[0].unwrap().triggered());
    advance_traps(&mut traps);
    assert!(!traps[0].unwrap().triggered());
    assert_eq!(a[1].state().removed, None);
}
#[test]
fn invalid_trap_metadata_reference_or_actor_order_fails_before_changes() {
    assert!(TrapState::new(0, 0).is_err());
    assert!(TrapState::new(3, 3).is_err());
    for code in [0, 16, 127] {
        let mut a = [actor(MotionAction::Digging, 0, 32, 100)];
        let before = a;
        let mut t = floor();
        let original = t.clone();
        let mut map = vec![128; 20 * 48];
        let mut traps = [None; 16];
        put(&mut map, 33, 100, code);
        let old = map.clone();
        assert!(tick(&mut a, &mut t, &mut map, &mut traps).is_err());
        assert_eq!(a, before);
        assert_eq!(t, original);
        assert_eq!(map, old);
    }
    let mut a = [actor(MotionAction::Walking, 0, 32, 100); 2];
    let before = a;
    let mut t = floor();
    let mut map = vec![128; 20 * 48];
    let mut traps = [None; 16];
    put(&mut map, 33, 100, 0);
    traps[0] = Some(TrapState::new(3, 0).unwrap());
    assert!(tick(&mut a, &mut t, &mut map, &mut traps).is_err());
    assert_eq!(a, before);
    assert!(!traps[0].unwrap().triggered());
}
#[test]
fn drowning_coordinate_overflow_preserves_actor_and_trap_state() {
    let mut a = [actor(MotionAction::Drowning, 0, i32::MAX, 100)];
    let before = a;
    let mut t = floor();
    let original = t.clone();
    let mut map = vec![128; 20 * 48];
    let mut traps = [None; 16];
    assert_eq!(
        tick(&mut a, &mut t, &mut map, &mut traps),
        Err(MotionError::CoordinateOverflow)
    );
    assert_eq!(a, before);
    assert_eq!(t, original);
    assert_eq!(traps, [None; 16]);
}
#[test]
fn hazard_skill_rules_and_final_state_validation_match_reference() {
    for (action, reason, last) in [
        (MotionAction::Drowning, RemovalReason::Drowned, 15),
        (MotionAction::Vaporizing, RemovalReason::Vaporized, 13),
    ] {
        let mut a = actor(action, 0, 32, 100);
        let mut count = 10;
        assert!(a.assign_climber(&mut count));
        assert!(a.assign_floater(&mut count));
        assert_eq!(
            a.assign_bomber(&mut count),
            action == MotionAction::Drowning
        );
        let mut s = actor(action, 0, 32, 100).state();
        s.removed = Some(reason);
        assert_eq!(
            Actor::from_state(s, MotionRules::default()),
            Err(MotionError::InvalidRemoval)
        );
        s.frame = last;
        s.end_of_animation = true;
        let mut done = Actor::from_state(s, MotionRules::default()).unwrap();
        assert!(!done.assign_climber(&mut count));
        assert!(!done.assign_bomber(&mut count));
    }
}
#[test]
fn drowning_or_fire_from_ohno_retains_blocker_storage_without_restoring_map() {
    for code in [OBJECT_WATER, OBJECT_FIRE] {
        let mut s = actor(MotionAction::OhNo, 0, 32, 100).state();
        s.blocker_saved = Some([128; 9]);
        let mut a = [Actor::from_state(s, MotionRules::default()).unwrap()];
        let mut t = floor();
        let mut map = vec![128; 20 * 48];
        let mut traps = [None; 16];
        put(&mut map, 32, 100, code);
        let original = map.clone();
        for _ in 0..17 {
            tick(&mut a, &mut t, &mut map, &mut traps).unwrap();
            Actor::from_state(a[0].state(), MotionRules::default()).unwrap();
        }
        assert!(a[0].state().removed.is_some());
        assert_eq!(a[0].state().blocker_saved, Some([128; 9]));
        assert_eq!(map, original);
    }
}

#[test]
fn continuous_trap_advances_when_idle_and_resets_on_trigger() {
    let mut traps = [None; 16];
    traps[0] = Some(TrapState::new(3, 1).unwrap().with_continuous(true));
    advance_traps(&mut traps);
    assert_eq!(
        (traps[0].unwrap().frame(), traps[0].unwrap().triggered()),
        (2, false)
    );
    let mut a = [actor(MotionAction::Walking, 0, 32, 100)];
    let mut t = floor();
    let mut map = vec![128; 20 * 48];
    put(&mut map, 33, 100, 0);
    tick(&mut a, &mut t, &mut map, &mut traps).unwrap();
    assert_eq!(
        (traps[0].unwrap().frame(), traps[0].unwrap().triggered()),
        (0, true)
    );
    for _ in 0..3 {
        advance_traps(&mut traps);
    }
    assert!(!traps[0].unwrap().triggered());
}
#[test]
fn timer_expiry_interrupts_either_terminal_hazard_before_animation_completion() {
    for action in [MotionAction::Drowning, MotionAction::Vaporizing] {
        let mut s = actor(action, 0, 32, 100).state();
        s.explosion_timer = 1;
        let mut a = [Actor::from_state(s, MotionRules::default()).unwrap()];
        let mut t = floor();
        let mut map = vec![128; 20 * 48];
        let mut traps = [None; 16];
        tick(&mut a, &mut t, &mut map, &mut traps).unwrap();
        assert_eq!(a[0].state().action, MotionAction::Exploding);
        assert_eq!(a[0].state().removed, None);
        Actor::from_state(a[0].state(), MotionRules::default()).unwrap();
    }
}

#[test]
fn dos_hazard_trigger_checks_stop_at_bottom_of_visible_playfield() {
    // Original EXE2fe5 rejects DOS footY>=176 (native footY>=160).
    for y in [159, 160, 163] {
        for code in [OBJECT_WATER, OBJECT_FIRE, 0] {
            let mut a = [actor(MotionAction::Falling, 0, 32, y)];
            let mut t = floor();
            let mut map = vec![128; 20 * 48];
            let mut traps = [None; 16];
            put(&mut map, 32, i64::from(y), code);
            if code == 0 {
                traps[0] = Some(TrapState::new(3, 0).unwrap());
            }
            tick(&mut a, &mut t, &mut map, &mut traps).unwrap();
            let expected = if y >= 160 || code == 0 {
                MotionAction::Walking
            } else if code == OBJECT_WATER {
                MotionAction::Drowning
            } else {
                MotionAction::Vaporizing
            };
            assert_eq!(a[0].state().action, expected);
            assert_eq!(
                a[0].state().removed,
                if y == 159 && code == 0 {
                    Some(RemovalReason::Trapped)
                } else {
                    None
                }
            );
            assert_eq!(
                traps.iter().flatten().any(|t| t.triggered()),
                y == 159 && code == 0
            );
        }
    }
    let mut state = actor(MotionAction::OhNo, 0, 32, 100).state();
    state.blocker_saved = Some([0; 9]);
    let mut a = [Actor::from_state(state, MotionRules::default()).unwrap()];
    let before = a;
    let mut t = floor();
    let old = t.clone();
    let mut map = vec![128; 20 * 48];
    let mut traps = [None; 16];
    assert_eq!(
        tick(&mut a, &mut t, &mut map, &mut traps),
        Err(MotionError::MissingTrap(0))
    );
    assert_eq!(a, before);
    assert_eq!(t, old);
}
