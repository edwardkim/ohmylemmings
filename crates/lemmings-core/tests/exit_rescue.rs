use lemmings_core::*;

const W: usize = 64;
const H: usize = 170;
fn actor(action: MotionAction, y: i32) -> Actor {
    let mut s = Actor::from_spawn(
        SpawnRequest {
            actor_id: 0,
            entrance_index: 0,
            iteration: 54,
        },
        &[EntrancePoint { x: 8, y: 86 }],
        MotionRules::default(),
    )
    .unwrap()
    .state();
    s.action = action;
    s.y = y;
    s.fallen = 0;
    s.frame = 0;
    if action == MotionAction::Splatting {
        s.direction = MotionDirection::Stopped;
    }
    s.is_floater = action == MotionAction::Floating;
    Actor::from_state(s, MotionRules::default()).unwrap()
}
fn tick(a: &mut Actor, cells: &mut [u8], map: &mut [u8]) -> Result<MotionOutcome, MotionError> {
    let blank = vec![0; W * H];
    a.advance_with_bomber(
        &mut TerrainMaskMut::new(W, H, cells).unwrap(),
        BomberContext::new(
            MiningContext::new(
                &[0; 104],
                BashingContext::new(&[0; 160], TerrainMask::new(W, H, &blank).unwrap()),
            ),
            &[0; 44],
        ),
        &mut ObjectMapMut::new(20, 48, -16, -16, map).unwrap(),
    )
}
fn put(map: &mut [u8], x: i64, y: i64, code: u8) {
    ObjectMapMut::new(20, 48, -16, -16, map)
        .unwrap()
        .write(x, y, code);
}
fn floor() -> Vec<u8> {
    (0..W * H).map(|i| u8::from(i / W >= 100)).collect()
}

#[test]
fn walking_enters_exit_after_motion_and_rescues_once_after_eight_calls() {
    let mut a = actor(MotionAction::Walking, 100);
    let mut t = floor();
    let before = t.clone();
    let mut map = vec![128; 20 * 48];
    put(&mut map, 33, 100, OBJECT_EXIT);
    assert!(tick(&mut a, &mut t, &mut map).unwrap().check_objects);
    assert_eq!(
        (a.state().action, a.state().frame, a.state().x),
        (MotionAction::Exiting, 0, 33)
    );
    for f in 1..=7 {
        assert_eq!(tick(&mut a, &mut t, &mut map).unwrap().removal, None);
        assert_eq!(a.state().frame, f);
    }
    assert_eq!(
        tick(&mut a, &mut t, &mut map).unwrap().removal,
        Some(RemovalReason::Rescued)
    );
    assert_eq!(a.state().frame, 7);
    assert!(a.state().end_of_animation);
    Actor::from_state(a.state(), MotionRules::default()).unwrap();
    assert_eq!(tick(&mut a, &mut t, &mut map).unwrap().removal, None);
    assert_eq!(t, before);
}

#[test]
fn falling_ignores_exit_until_landing_and_splat_exit_keeps_stopped_direction() {
    for fatal in [false, true] {
        let mut a = actor(MotionAction::Falling, 97);
        let mut s = a.state();
        s.fallen = if fatal { 61 } else { 60 };
        a = Actor::from_state(s, MotionRules::default()).unwrap();
        let mut t = floor();
        let mut map = vec![128; 20 * 48];
        put(&mut map, 32, 96, OBJECT_EXIT);
        tick(&mut a, &mut t, &mut map).unwrap();
        assert_eq!(a.state().action, MotionAction::Falling);
        put(&mut map, 32, 100, OBJECT_EXIT);
        let out = tick(&mut a, &mut t, &mut map).unwrap();
        assert!(out.splat_started); // Previous airborne call adds three, even from60.
        assert_eq!(
            (a.state().action, a.state().direction),
            (MotionAction::Exiting, MotionDirection::Stopped)
        );
        Actor::from_state(a.state(), MotionRules::default()).unwrap();
        for _ in 0..8 {
            tick(&mut a, &mut t, &mut map).unwrap();
        }
        assert_eq!(a.state().removed, Some(RemovalReason::Rescued));
    }
}

#[test]
fn nonfatal_landing_and_floating_can_enter_exit_but_splat_animation_does_not_recheck() {
    for action in [MotionAction::Falling, MotionAction::Floating] {
        let mut a = actor(action, 100);
        let mut t = floor();
        let mut map = vec![128; 20 * 48];
        put(&mut map, 32, 100, OBJECT_EXIT);
        tick(&mut a, &mut t, &mut map).unwrap();
        assert_eq!(a.state().action, MotionAction::Exiting);
    }
    let mut a = actor(MotionAction::Splatting, 100);
    let mut t = floor();
    let mut map = vec![128; 20 * 48];
    put(&mut map, 32, 100, OBJECT_EXIT);
    for _ in 0..16 {
        tick(&mut a, &mut t, &mut map).unwrap();
    }
    assert_eq!(a.state().removed, Some(RemovalReason::SplattingFinished));
}

#[test]
fn fatal_landing_cancels_bomber_even_when_exit_replaces_the_new_splat() {
    let mut a = actor(MotionAction::Falling, 100);
    let mut s = a.state();
    s.fallen = 61;
    s.explosion_timer = 42;
    a = Actor::from_state(s, MotionRules::default()).unwrap();
    let mut t = floor();
    let mut map = vec![128; 20 * 48];
    put(&mut map, 32, 100, OBJECT_EXIT);
    assert!(tick(&mut a, &mut t, &mut map).unwrap().splat_started);
    assert_eq!(a.state().action, MotionAction::Exiting);
    assert_eq!(a.state().explosion_timer, 0);
}

#[test]
fn exiting_ignores_later_objects_and_never_moves_or_erases_terrain() {
    let mut a = actor(MotionAction::Exiting, 100);
    let mut t = floor();
    let before = t.clone();
    let mut map = vec![128; 20 * 48];
    put(&mut map, 32, 100, OBJECT_FORCE_LEFT);
    for _ in 0..8 {
        assert!(!tick(&mut a, &mut t, &mut map).unwrap().check_objects);
    }
    assert_eq!(
        (a.state().x, a.state().y, a.state().direction),
        (32, 100, MotionDirection::Right)
    );
    assert_eq!(t, before);
}

#[test]
fn countdown_expiry_precedes_exit_completion_and_long_timer_survives_rescue() {
    for timer in [1, 8, 9, 79] {
        let mut a = actor(MotionAction::Exiting, 100);
        let mut s = a.state();
        s.explosion_timer = timer;
        a = Actor::from_state(s, MotionRules::default()).unwrap();
        let mut t = floor();
        let mut map = vec![128; 20 * 48];
        for _ in 0..8 {
            tick(&mut a, &mut t, &mut map).unwrap();
        }
        if timer <= 8 {
            assert_eq!(a.state().action, MotionAction::OhNo);
            assert_eq!(a.state().removed, None);
        } else {
            assert_eq!(a.state().removed, Some(RemovalReason::Rescued));
            assert_eq!(a.state().explosion_timer, timer - 8);
        }
        Actor::from_state(a.state(), MotionRules::default()).unwrap();
    }
}

#[test]
fn stopped_exit_interrupted_by_a_restored_timer_keeps_restorable_terminal_states() {
    let mut s = actor(MotionAction::Exiting, 100).state();
    s.direction = MotionDirection::Stopped;
    s.explosion_timer = 1;
    let mut a = Actor::from_state(s, MotionRules::default()).unwrap();
    let mut t = floor();
    let mut map = vec![128; 20 * 48];
    for _ in 0..18 {
        tick(&mut a, &mut t, &mut map).unwrap();
        Actor::from_state(a.state(), MotionRules::default()).unwrap();
    }
    assert_eq!(a.state().removed, Some(RemovalReason::Exploded));
}

#[test]
fn ohno_exit_preserves_original_blocker_storage_without_restoring_it_on_rescue() {
    let mut s = actor(MotionAction::OhNo, 100).state();
    s.blocker_saved = Some([128; 9]);
    let mut a = Actor::from_state(s, MotionRules::default()).unwrap();
    let mut t = floor();
    let mut map = vec![128; 20 * 48];
    put(&mut map, 32, 100, OBJECT_EXIT);
    let before = map.clone();
    tick(&mut a, &mut t, &mut map).unwrap();
    assert_eq!(a.state().action, MotionAction::Exiting);
    for _ in 0..8 {
        tick(&mut a, &mut t, &mut map).unwrap();
    }
    assert_eq!(a.state().removed, Some(RemovalReason::Rescued));
    assert_eq!(a.state().blocker_saved, Some([128; 9]));
    assert_eq!(map, before);
    Actor::from_state(a.state(), MotionRules::default()).unwrap();
}

#[test]
fn exits_on_same_tick_count_as_both_removed_and_rescued_in_strict_id_order() {
    let mut actors = vec![actor(MotionAction::Exiting, 100); 2];
    for (id, a) in actors.iter_mut().enumerate() {
        let mut s = a.state();
        s.id = id as u32;
        s.frame = 7;
        *a = Actor::from_state(s, MotionRules::default()).unwrap();
    }
    let t = floor();
    let mut ids = Vec::new();
    let out = step_actors(&mut actors, TerrainMask::new(W, H, &t).unwrap(), |a, _| {
        ids.push(a.id)
    })
    .unwrap();
    assert_eq!((out.active_before, out.removed, out.rescued), (2, 2, 2));
    assert_eq!(ids, [0, 1]);
    // The older Digger batch is a separate public coordinator, and must report
    // the same population counters when restoring an actor already in an exit.
    let mut restored = actors.clone();
    for a in &mut restored {
        let mut s = a.state();
        s.removed = None;
        s.end_of_animation = false;
        *a = Actor::from_state(s, MotionRules::default()).unwrap();
    }
    let blank = vec![0; W * H];
    let mut mutable_terrain = t.clone();
    let out = step_actors_mut(
        &mut restored,
        &mut TerrainMaskMut::new(W, H, &mut mutable_terrain).unwrap(),
        TerrainMask::new(W, H, &blank).unwrap(),
        |_, _| {},
    )
    .unwrap();
    assert_eq!((out.active_before, out.removed, out.rescued), (2, 2, 2));
    let out = step_actors(
        &mut actors,
        TerrainMask::new(W, H, &t).unwrap(),
        |_, _| panic!(),
    )
    .unwrap();
    assert_eq!((out.removed, out.rescued), (0, 0));
}

#[test]
fn rescue_restore_requires_exit_final_frame_and_finished_animation() {
    let mut s = actor(MotionAction::Exiting, 100).state();
    s.removed = Some(RemovalReason::Rescued);
    assert_eq!(
        Actor::from_state(s, MotionRules::default()),
        Err(MotionError::InvalidRemoval)
    );
    s.frame = 7;
    assert_eq!(
        Actor::from_state(s, MotionRules::default()),
        Err(MotionError::InvalidRemoval)
    );
    s.end_of_animation = true;
    assert!(Actor::from_state(s, MotionRules::default()).is_ok());
}

#[test]
fn coordinate_error_and_actor_order_error_preserve_exit_candidates() {
    let mut a = actor(MotionAction::Walking, 100);
    let mut s = a.state();
    s.x = i32::MAX;
    a = Actor::from_state(s, MotionRules::default()).unwrap();
    let before = a;
    let mut t = floor();
    let terrain = t.clone();
    let mut map = vec![128; 20 * 48];
    let saved_map = map.clone();
    assert_eq!(
        tick(&mut a, &mut t, &mut map),
        Err(MotionError::CoordinateOverflow)
    );
    assert_eq!(a, before);
    assert_eq!(t, terrain);
    assert_eq!(map, saved_map);
    let mut actors = [actor(MotionAction::Exiting, 100); 2];
    let before = actors;
    assert!(
        step_actors(
            &mut actors,
            TerrainMask::new(W, H, &t).unwrap(),
            |_, _| panic!()
        )
        .is_err()
    );
    assert_eq!(actors, before);
}
