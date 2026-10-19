use lemmings_core::*;
const W: usize = 64;
const H: usize = 170;
const N: usize = W * H;
const OW: usize = 24;
const OH: usize = 48;
const ON: usize = OW * OH;
fn actor(action: MotionAction, id: u32, x: i32, y: i32) -> Actor {
    Actor::from_state(
        ActorState {
            id,
            born_iteration: 0,
            x,
            y,
            direction: if action == MotionAction::Splatting {
                MotionDirection::Stopped
            } else {
                MotionDirection::Right
            },
            action,
            frame: 0,
            end_of_animation: false,
            fallen: 0,
            new_digger: false,
            bricks_left: if action == MotionAction::Building {
                12
            } else {
                0
            },
            blocker_saved: None,
            is_climber: false,
            is_floater: false,
            float_index: 0,
            walking_once: false,
            explosion_timer: 0,
            particle_timer: 0,
            particle_frame: 0,
            object_below: 0,
            object_in_front: 0,
            removed: None,
        },
        MotionRules::default(),
    )
    .unwrap()
}
fn map(cells: &mut [u8]) -> ObjectMapMut<'_> {
    ObjectMapMut::new(OW, OH, -16, -16, cells).unwrap()
}
fn context(empty: &[u8]) -> MiningContext<'_> {
    MiningContext::new(
        &[255; 104],
        BashingContext::new(&[255; 160], TerrainMask::new(W, H, empty).unwrap()),
    )
}
#[test]
fn object_cells_round_negative_coordinates_down_and_preserve_raw_codes() {
    let mut bytes = [128; 9];
    let mut m = ObjectMapMut::new(3, 3, -4, -4, &mut bytes).unwrap();
    assert!(m.write(-1, -1, 17));
    for y in -4..0 {
        for x in -4..0 {
            assert_eq!(m.as_read_only().read(x, y), 17);
        }
    }
    assert_eq!(m.as_read_only().read(0, -1), 128);
    assert_eq!(m.as_read_only().read(-5, -1), 128);
    assert!(!m.write(8, 0, 137));
    assert!(ObjectMap::new(3, 3, -3, -4, &bytes).is_err());
    assert!(ObjectMap::new(0, 3, 0, 0, &[]).is_err());
    assert!(ObjectMap::new(3, 3, 0, 0, &bytes[..8]).is_err());
    assert!(ObjectMap::new(usize::MAX, 2, 0, 0, &[]).is_err());
}
#[test]
fn assignment_saves_nine_cells_overwrites_columns_and_rejects_duplicates() {
    for action in [
        MotionAction::Walking,
        MotionAction::Digging,
        MotionAction::Bashing,
        MotionAction::Mining,
        MotionAction::Building,
        MotionAction::Shrugging,
    ] {
        for dx in 0..4 {
            for dy in 0..4 {
                let mut cells = [137; ON];
                let mut m = map(&mut cells);
                let mut a = actor(action, 0, 32 + dx, 100 + dy);
                let before = a;
                let mut count = 2;
                assert!(a.assign_blocker(&mut count, &mut m));
                assert_eq!(
                    (count, a.state().action, a.state().bricks_left),
                    (1, MotionAction::Blocking, 0)
                );
                assert_eq!(a.state().blocker_saved, Some([137; 9]));
                for row in [-6, -2, 2] {
                    for (x, v) in [(-4, 130), (0, 138), (4, 131)] {
                        assert_eq!(
                            m.as_read_only().read(
                                i64::from(before.state().x + x),
                                i64::from(before.state().y + row)
                            ),
                            v
                        );
                    }
                }
                let saved = a;
                let bytes = m.as_read_only().as_bytes().to_vec();
                assert!(!a.assign_blocker(&mut count, &mut m));
                assert_eq!((a, count), (saved, 1));
                assert_eq!(m.as_read_only().as_bytes(), bytes);
            }
        }
    }
}
#[test]
fn every_field_cell_blocks_overlap_but_air_and_non_field_codes_allow_assignment() {
    for row in [-6, -2, 2] {
        for x in [-4, 0, 4] {
            for value in [130, 131, 138] {
                let mut cells = [128; ON];
                let mut m = map(&mut cells);
                m.write(32 + x, 100 + row, value);
                let mut a = actor(MotionAction::Walking, 0, 32, 100);
                let before = a;
                let mut count = 1;
                assert!(!a.assign_blocker(&mut count, &mut m));
                assert_eq!((a, count), (before, 1));
            }
        }
    }
    for action in [
        MotionAction::Falling,
        MotionAction::Jumping,
        MotionAction::Splatting,
    ] {
        let mut cells = [128; ON];
        let mut m = map(&mut cells);
        let mut a = actor(action, 0, 32, 100);
        let before = a;
        let mut count = 1;
        assert!(!a.assign_blocker(&mut count, &mut m));
        assert_eq!((a, count), (before, 1));
    }
    let mut cells = [128; ON];
    let mut m = map(&mut cells);
    let mut a = actor(MotionAction::Walking, 0, 32, 100);
    assert!(!a.assign_blocker(&mut 0, &mut m));
    assert!(a.assign_blocker(&mut 1, &mut m)); // no support precheck
}
#[test]
fn blocking_loops_without_moving_then_restores_map_before_next_tick_fall() {
    let mut cells = [128; ON];
    let mut m = map(&mut cells);
    for (i, (x, y)) in [-6, -2, 2]
        .into_iter()
        .flat_map(|y| [-4, 0, 4].map(move |x| (x, y)))
        .enumerate()
    {
        m.write(32 + x, 100 + y, 135 + (i % 3) as u8);
    }
    let original = m.as_read_only().as_bytes().to_vec();
    let mut a = actor(MotionAction::Walking, 0, 32, 100);
    assert!(a.assign_blocker(&mut 1, &mut m));
    let empty = [0; N];
    let mut terrain = [0; N];
    terrain[100 * W + 32] = 1;
    for tick in 1..=32 {
        let out = a
            .advance_with_objects(
                &mut TerrainMaskMut::new(W, H, &mut terrain).unwrap(),
                context(&empty),
                &mut m,
            )
            .unwrap();
        assert_eq!(
            (a.state().x, a.state().y, a.state().frame),
            (32, 100, tick % 16)
        );
        assert!(!out.check_objects);
        assert_eq!(a.state().action, MotionAction::Blocking);
    }
    terrain[100 * W + 32] = 0;
    let out = a
        .advance_with_objects(
            &mut TerrainMaskMut::new(W, H, &mut terrain).unwrap(),
            context(&empty),
            &mut m,
        )
        .unwrap();
    assert!(!out.check_objects);
    assert_eq!(
        (a.state().action, a.state().x, a.state().y),
        (MotionAction::Walking, 32, 100)
    );
    assert_eq!(a.state().blocker_saved, None);
    assert_eq!(m.as_read_only().as_bytes(), original);
    a.advance_with_objects(
        &mut TerrainMaskMut::new(W, H, &mut terrain).unwrap(),
        context(&empty),
        &mut m,
    )
    .unwrap();
    assert_eq!(
        (a.state().action, a.state().y),
        (MotionAction::Falling, 104)
    );
}
#[test]
fn forces_apply_only_after_eligible_motion_frames_and_preserve_animation() {
    let empty = [0; N];
    for (action, frame, bricks) in [
        (MotionAction::Walking, 0, 0),
        (MotionAction::Falling, 1, 0),
        (MotionAction::Building, 8, 12),
        (MotionAction::Building, 9, 9),
        (MotionAction::Building, 10, 8),
        (MotionAction::Bashing, 2, 0),
        (MotionAction::Bashing, 10, 0),
        (MotionAction::Mining, 0, 0),
        (MotionAction::Mining, 2, 0),
        (MotionAction::Digging, 7, 0),
        (MotionAction::Shrugging, 6, 0),
        (MotionAction::Shrugging, 7, 0),
    ] {
        for direction in [MotionDirection::Left, MotionDirection::Right] {
            let mut initial = actor(action, 0, 32, 100).state();
            initial.frame = frame;
            initial.bricks_left = bricks;
            initial.direction = direction;
            let a = Actor::from_state(initial, MotionRules::default()).unwrap();
            let mut expected = a;
            let mut base = [0; N];
            base[100 * W..].fill(1);
            let mut actual_terrain = base;
            let out = expected
                .advance_with_jobs(
                    &mut TerrainMaskMut::new(W, H, &mut base).unwrap(),
                    context(&empty),
                )
                .unwrap();
            let mut actual = a;
            let mut cells = [128; ON];
            let mut m = map(&mut cells);
            m.write(
                i64::from(expected.state().x),
                i64::from(expected.state().y),
                if direction == MotionDirection::Right {
                    130
                } else {
                    131
                },
            );
            actual
                .advance_with_objects(
                    &mut TerrainMaskMut::new(W, H, &mut actual_terrain).unwrap(),
                    context(&empty),
                    &mut m,
                )
                .unwrap();
            let mut wanted = expected.state();
            if out.check_objects && wanted.removed.is_none() {
                wanted.object_below = m
                    .as_read_only()
                    .read(i64::from(wanted.x), i64::from(wanted.y));
                wanted.object_in_front = m.as_read_only().read(
                    i64::from(wanted.x) + i64::from(wanted.direction.delta()) * 8,
                    i64::from(wanted.y) - 8,
                );
            }
            if out.check_objects && wanted.removed.is_none() {
                wanted.direction = if direction == MotionDirection::Left {
                    MotionDirection::Right
                } else {
                    MotionDirection::Left
                };
            }
            assert_eq!(actual.state(), wanted, "{action:?} {frame} {direction:?}");
            assert_eq!(actual_terrain, base);
        }
    }
}
#[test]
fn blocker_overrides_steel_and_restores_it_when_support_disappears() {
    let mut cells = [137; ON];
    let mut m = map(&mut cells);
    let mut b = actor(MotionAction::Walking, 0, 32, 100);
    assert!(b.assign_blocker(&mut 1, &mut m));
    let empty = [0; N];
    let objects = BashingContext {
        object_map: Some(m.as_read_only()),
        ..context(&empty).objects
    };
    let mut digger = actor(MotionAction::Walking, 1, 28, 100);
    assert!(digger.assign_digger_with_objects(&mut 1, objects));
    let mut basher = actor(MotionAction::Walking, 2, 20, 104);
    assert!(basher.assign_basher(&mut 1, objects));
    b.advance_with_objects(
        &mut TerrainMaskMut::new(W, H, &mut [0; N]).unwrap(),
        context(&empty),
        &mut m,
    )
    .unwrap();
    let objects = BashingContext {
        object_map: Some(m.as_read_only()),
        ..context(&empty).objects
    };
    let mut digger = Actor::from_state(
        ActorState {
            object_below: 137,
            ..actor(MotionAction::Walking, 3, 28, 100).state()
        },
        MotionRules::default(),
    )
    .unwrap();
    let mut basher = Actor::from_state(
        ActorState {
            object_in_front: 137,
            ..actor(MotionAction::Walking, 4, 20, 104).state()
        },
        MotionRules::default(),
    )
    .unwrap();
    assert!(!digger.assign_digger_with_objects(&mut 1, objects));
    assert!(!basher.assign_basher(&mut 1, objects));
    assert!(m.as_read_only().as_bytes().iter().all(|&v| v == 137));
}
#[test]
fn same_tick_digging_and_blocker_release_follow_id_order() {
    let empty = [0; N];
    for digger_first in [true, false] {
        let mut cells = [128; ON];
        let mut m = map(&mut cells);
        let mut b = actor(MotionAction::Walking, 1, 32, 100);
        assert!(b.assign_blocker(&mut 1, &mut m));
        let mut d = actor(
            MotionAction::Digging,
            if digger_first { 0 } else { 2 },
            32,
            100,
        )
        .state();
        d.new_digger = true;
        let d = Actor::from_state(d, MotionRules::default()).unwrap();
        let w = actor(
            MotionAction::Walking,
            if digger_first { 2 } else { 0 },
            27,
            96,
        );
        let mut actors = if digger_first { [d, b, w] } else { [w, b, d] };
        let mut terrain = [0; N];
        terrain[100 * W..].fill(1);
        step_actors_with_objects(
            &mut actors,
            &mut TerrainMaskMut::new(W, H, &mut terrain).unwrap(),
            context(&empty),
            &mut m,
            |_, _| {},
        )
        .unwrap();
        if digger_first {
            assert_eq!(actors[1].state().action, MotionAction::Walking);
            assert_eq!(actors[2].state().direction, MotionDirection::Right);
        } else {
            assert_eq!(actors[1].state().action, MotionAction::Blocking);
            assert_eq!(actors[0].state().direction, MotionDirection::Left);
        }
    }
}
#[test]
fn missing_map_invalid_restoration_bad_ids_and_motion_errors_preserve_all_records() {
    let mut cells = [128; ON];
    let mut m = map(&mut cells);
    let mut a = actor(MotionAction::Walking, 0, 32, 100);
    assert!(a.assign_blocker(&mut 1, &mut m));
    let before = a;
    assert_eq!(
        a.advance(TerrainMask::new(W, H, &[0; N]).unwrap()),
        Err(MotionError::ObjectMapRequired)
    );
    assert_eq!(a, before);
    for state in [
        ActorState {
            blocker_saved: None,
            is_climber: false,
            is_floater: false,
            float_index: 0,
            walking_once: false,
            explosion_timer: 0,
            particle_timer: 0,
            particle_frame: 0,
            object_below: 0,
            object_in_front: 0,
            ..before.state()
        },
        ActorState {
            action: MotionAction::Walking,
            ..before.state()
        },
    ] {
        assert_eq!(
            Actor::from_state(state, MotionRules::default()),
            Err(MotionError::InvalidBlockerState)
        );
    }
    let saved = m.as_read_only().as_bytes().to_vec();
    let mut terrain = [0; N];
    let mut actors = [before, before];
    assert_eq!(
        step_actors_with_objects(
            &mut actors,
            &mut TerrainMaskMut::new(W, H, &mut terrain).unwrap(),
            context(&[0; N]),
            &mut m,
            |_, _| panic!("emit")
        ),
        Err(MotionError::ActorOrder { index: 1 })
    );
    assert_eq!(actors, [before, before]);
    assert_eq!(m.as_read_only().as_bytes(), saved);
    assert_eq!(terrain, [0; N]);
    let mut overflow = Actor::from_state(
        ActorState {
            x: i32::MAX,
            action: MotionAction::Walking,
            blocker_saved: None,
            is_climber: false,
            is_floater: false,
            float_index: 0,
            walking_once: false,
            explosion_timer: 0,
            particle_timer: 0,
            particle_frame: 0,
            ..before.state()
        },
        MotionRules::default(),
    )
    .unwrap();
    let initial = overflow;
    assert_eq!(
        overflow.advance_with_objects(
            &mut TerrainMaskMut::new(W, H, &mut terrain).unwrap(),
            context(&[0; N]),
            &mut m
        ),
        Err(MotionError::CoordinateOverflow)
    );
    assert_eq!(overflow, initial);
    assert_eq!(terrain, [0; N]);
    assert_eq!(m.as_read_only().as_bytes(), saved);
    for (x, y) in [(i64::MIN, i64::MAX), (i64::MAX, i64::MIN)] {
        assert_eq!(m.as_read_only().read(x, y), 128);
        assert!(!m.write(x, y, 137));
    }
}

#[test]
fn force_reads_front_before_turn_and_assignments_reuse_the_cached_object_codes() {
    let empty = [0; N];
    let mut terrain = [0; N];
    terrain[100 * W..].fill(1);
    let mut cells = [128; ON];
    let mut m = map(&mut cells);
    m.write(21, 100, 130);
    m.write(29, 92, 137);
    let mut a = actor(MotionAction::Walking, 0, 20, 100);
    a.advance_with_objects(
        &mut TerrainMaskMut::new(W, H, &mut terrain).unwrap(),
        context(&empty),
        &mut m,
    )
    .unwrap();
    assert_eq!(
        (
            a.state().direction,
            a.state().object_below,
            a.state().object_in_front
        ),
        (MotionDirection::Left, 130, 137)
    );
    let objects = BashingContext {
        object_map: Some(m.as_read_only()),
        ..context(&empty).objects
    };
    assert!(!a.assign_basher(&mut 1, objects)); // cached right-side steel despite now facing left
    a.advance_with_objects(
        &mut TerrainMaskMut::new(W, H, &mut terrain).unwrap(),
        context(&empty),
        &mut m,
    )
    .unwrap();
    assert_eq!(a.state().object_in_front, 128);
    m.write(i64::from(a.state().x), i64::from(a.state().y), 137);
    let objects = BashingContext {
        object_map: Some(m.as_read_only()),
        ..context(&empty).objects
    };
    assert!(a.assign_digger_with_objects(&mut 1, objects)); // cached force, not the new steel
}
