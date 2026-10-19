use lemmings_core::*;

const N: usize = 64 * 170;
fn view(cells: &[u8]) -> TerrainMask<'_> {
    TerrainMask::new(64, 170, cells).unwrap()
}
fn actor(action: MotionAction, frame: u8, direction: MotionDirection) -> Actor {
    Actor::from_state(
        ActorState {
            id: 0,
            born_iteration: 54,
            x: 32,
            y: 40,
            direction,
            action,
            frame,
            end_of_animation: false,
            fallen: 0,
            new_digger: false,
            bricks_left: 0,
            blocker_saved: None,
            object_below: 0,
            object_in_front: 0,
            removed: None,
        },
        MotionRules::default(),
    )
    .unwrap()
}
fn context<'a>(masks: &'a [u8; 104], steel: &'a [u8]) -> MiningContext<'a> {
    MiningContext::new(masks, BashingContext::new(&[0; 160], view(steel)))
}
fn tick(a: &mut Actor, cells: &mut [u8], c: MiningContext<'_>) -> MotionOutcome {
    a.advance_with_jobs(&mut TerrainMaskMut::new(64, 170, cells).unwrap(), c)
        .unwrap()
}

#[test]
fn assignment_checks_front_and_foot_and_replaces_current_supported_jobs() {
    let empty = [0; N];
    let masks = [0; 104];
    for action in [
        MotionAction::Walking,
        MotionAction::Digging,
        MotionAction::Bashing,
        MotionAction::Mining,
        MotionAction::Falling,
        MotionAction::Jumping,
    ] {
        let mut a = actor(action, 0, MotionDirection::Right);
        let before = a;
        let mut count = 2;
        let allowed = matches!(
            action,
            MotionAction::Walking | MotionAction::Digging | MotionAction::Bashing
        );
        assert_eq!(a.assign_miner(&mut count, context(&masks, &empty)), allowed);
        assert_eq!(count, if allowed { 1 } else { 2 });
        if allowed {
            assert_eq!(
                (a.state().action, a.state().frame, a.state().new_digger),
                (MotionAction::Mining, 0, false)
            );
            assert_eq!(a.state().y, 41); // entering Mining also drops the foot once
        } else {
            assert_eq!(a, before);
        }
    }
    for direction in [MotionDirection::Left, MotionDirection::Right] {
        for restriction in 0..4 {
            let mut map = empty;
            map[32 * 64 + (32 + 8 * direction.delta()) as usize] = 1;
            let mut c = context(&masks, &empty);
            match restriction {
                0 => c.objects.steel = view(&map),
                1 => {
                    map = empty;
                    map[40 * 64 + 32] = 1;
                    c.objects.steel = view(&map);
                }
                2 => c.objects.one_way_left = Some(view(&map)),
                _ => c.objects.one_way_right = Some(view(&map)),
            }
            let allowed = (restriction == 2 && direction == MotionDirection::Left)
                || (restriction == 3 && direction == MotionDirection::Right);
            let mut a = actor(MotionAction::Walking, 0, direction);
            let before = a;
            let mut count = 2;
            assert_eq!(a.assign_miner(&mut count, c), allowed);
            if !allowed {
                assert_eq!((a, count), (before, 2));
            }
        }
    }
    let mut a = actor(MotionAction::Walking, 0, MotionDirection::Right);
    let before = a;
    assert!(!a.assign_miner(&mut 0, context(&masks, &empty)));
    assert_eq!(a, before);
    let mut overflow = Actor::from_state(
        ActorState {
            y: i32::MAX,
            ..before.state()
        },
        MotionRules::default(),
    )
    .unwrap();
    let saved = overflow;
    let mut remaining = 1;
    assert!(!overflow.assign_miner(&mut remaining, context(&masks, &empty)));
    assert_eq!((overflow, remaining), (saved, 1));
}

#[test]
fn two_directional_masks_use_distinct_frames_and_second_mask_moves_origin() {
    let empty = [0; N];
    let mut masks = [0; 104];
    for i in 0..4 {
        masks[i * 26] = 128 >> i;
    }
    for direction in [MotionDirection::Right, MotionDirection::Left] {
        for next in [1, 2] {
            let mut a = actor(MotionAction::Mining, next - 1, direction);
            let mut cells = [1; N];
            assert!(!tick(&mut a, &mut cells, context(&masks, &empty)).check_objects);
            let bit = usize::from(next - 1)
                + if direction == MotionDirection::Left {
                    2
                } else {
                    0
                };
            let left = 24 + if next == 2 { direction.delta() } else { 0 };
            let top = 27 + if next == 2 { 1 } else { 0 };
            assert_eq!(cells.iter().filter(|&&v| v == 0).count(), 1);
            assert_eq!(cells[top as usize * 64 + left as usize + bit], 0);
        }
    }
}

#[test]
fn full_cycle_moves_four_pixels_sideways_and_two_down_with_object_checks_only_at_zero_three_fifteen()
 {
    let empty = [0; N];
    let mut cells = [1; N];
    let mut a = actor(MotionAction::Mining, 0, MotionDirection::Right);
    for i in 1..=24 {
        let out = tick(&mut a, &mut cells, context(&[0; 104], &empty));
        assert_eq!(out.check_objects, matches!(i % 24, 0 | 3 | 15));
        assert_eq!(a.state().frame, i % 24);
        assert_eq!(a.state().end_of_animation, i == 24);
    }
    assert_eq!((a.state().x, a.state().y), (36, 42));
}

#[test]
fn each_of_the_two_horizontal_substeps_checks_boundary_before_vertical_motion() {
    let empty = [0; N];
    for (x, direction, expected) in [
        (0, MotionDirection::Left, -1),
        (1, MotionDirection::Left, -1),
        (1647, MotionDirection::Right, 1648),
        (1646, MotionDirection::Right, 1648),
    ] {
        let mut a = Actor::from_state(
            ActorState {
                x,
                ..actor(MotionAction::Mining, 2, direction).state()
            },
            MotionRules::default(),
        )
        .unwrap();
        let mut cells = [1; N];
        assert!(tick(&mut a, &mut cells, context(&[0; 104], &empty)).check_objects);
        assert_eq!(
            (a.state().x, a.state().y, a.state().action, a.state().frame),
            (expected, 40, MotionAction::Walking, 0)
        );
        assert_ne!(a.state().direction, direction);
    }
}

#[test]
fn bottom_removal_precedes_support_and_zero_frame_does_not_test_support() {
    let empty = [0; N];
    for frame in [2, 23] {
        let mut a = Actor::from_state(
            ActorState {
                y: 163,
                ..actor(MotionAction::Mining, frame, MotionDirection::Right).state()
            },
            MotionRules::default(),
        )
        .unwrap();
        let mut cells = [1; N];
        let out = tick(&mut a, &mut cells, context(&[0; 104], &empty));
        assert_eq!(out.removal, Some(RemovalReason::BottomBoundary));
        assert!(!out.check_objects);
        assert_eq!(a.state().y, 164);
        assert_eq!(cells, [1; N]);
    }
    let mut a = actor(MotionAction::Mining, 23, MotionDirection::Right);
    let mut cells = empty;
    assert!(tick(&mut a, &mut cells, context(&[0; 104], &empty)).check_objects);
    assert_eq!((a.state().action, a.state().y), (MotionAction::Mining, 41));
}

#[test]
fn support_and_steel_one_way_truth_table_includes_original_right_wall_bug() {
    let empty = [0; N];
    let masks = [0; 104];
    for bug in [false, true] {
        for direction in [MotionDirection::Left, MotionDirection::Right] {
            for frame in 0..24 {
                for supported in [false, true] {
                    for object in 0..4 {
                        let next = (frame + 1) % 24;
                        let moving = matches!(next, 3 | 15);
                        let x = 32 + 2 * direction.delta();
                        let y = 40 + i32::from(next == 3);
                        let mut cells = empty;
                        if supported {
                            cells[y as usize * 64 + x as usize] = 1;
                        }
                        let mut map = empty;
                        map[y as usize * 64 + x as usize] = 1;
                        let mut c = context(&masks, &empty);
                        c.one_way_right_bug = bug;
                        match object {
                            1 => c.objects.steel = view(&map),
                            2 => c.objects.one_way_left = Some(view(&map)),
                            3 => c.objects.one_way_right = Some(view(&map)),
                            _ => {}
                        }
                        let mut a = actor(MotionAction::Mining, frame, direction);
                        let out = tick(&mut a, &mut cells, c);
                        let blocked = object == 1
                            || (object == 2 && direction == MotionDirection::Right)
                            || (object == 3 && (bug || direction == MotionDirection::Left));
                        let expected = if moving && !supported {
                            MotionAction::Falling
                        } else if moving && blocked {
                            MotionAction::Walking
                        } else {
                            MotionAction::Mining
                        };
                        assert_eq!(
                            a.state().action,
                            expected,
                            "{bug} {direction:?} {frame} {supported} {object}"
                        );
                        assert_eq!(out.check_objects, matches!(next, 0 | 3 | 15));
                    }
                }
            }
        }
    }
}

#[test]
fn arithmetic_errors_preserve_pixels_and_record_and_legacy_apis_reject_mining() {
    let empty = [0; N];
    for (x, y, frame) in [
        (i32::MIN, 40, 0),
        (32, i32::MIN, 0),
        (i32::MAX, 40, 2),
        (32, i32::MAX, 2),
        (32, i32::MAX, 23),
    ] {
        let mut a = Actor::from_state(
            ActorState {
                x,
                y,
                ..actor(MotionAction::Mining, frame, MotionDirection::Right).state()
            },
            MotionRules::default(),
        )
        .unwrap();
        let before = a;
        let mut cells = [1; N];
        assert_eq!(
            a.advance_with_jobs(
                &mut TerrainMaskMut::new(64, 170, &mut cells).unwrap(),
                context(&[255; 104], &empty)
            ),
            Err(MotionError::CoordinateOverflow)
        );
        assert_eq!((a, cells), (before, [1; N]));
        assert_eq!(
            a.advance(view(&empty)),
            Err(MotionError::MutableTerrainRequired)
        );
    }
}

#[test]
fn all_implemented_job_replacements_reset_the_animation_and_new_digger_flag() {
    let empty = [0; N];
    let mut a = actor(MotionAction::Mining, 9, MotionDirection::Right);
    assert!(a.assign_digger(&mut 1, view(&empty)));
    assert!(a.state().new_digger);
    assert!(a.assign_miner(&mut 1, context(&[0; 104], &empty)));
    assert!(!a.state().new_digger);
    assert!(a.assign_basher(&mut 1, BashingContext::new(&[0; 160], view(&empty))));
    assert_eq!(
        (a.state().action, a.state().frame),
        (MotionAction::Bashing, 0)
    );
}

#[test]
fn batch_validates_ids_and_later_walker_observes_the_miner_mask() {
    let empty = [0; N];
    let mut masks = [0; 104];
    masks[25] = 64;
    let a = actor(MotionAction::Mining, 0, MotionDirection::Right);
    let mut cells = empty;
    cells[39 * 64 + 33] = 1;
    let b = Actor::from_state(
        ActorState {
            id: 1,
            y: 39,
            ..actor(MotionAction::Walking, 0, MotionDirection::Right).state()
        },
        MotionRules::default(),
    )
    .unwrap();
    let mut actors = [a, b];
    step_actors_with_jobs(
        &mut actors,
        &mut TerrainMaskMut::new(64, 170, &mut cells).unwrap(),
        context(&masks, &empty),
        |_, _| {},
    )
    .unwrap();
    assert_eq!(actors[1].state().action, MotionAction::Falling);
    let mut actors = [a, a];
    let before = actors;
    cells[39 * 64 + 33] = 1;
    assert_eq!(
        step_actors_with_jobs(
            &mut actors,
            &mut TerrainMaskMut::new(64, 170, &mut cells).unwrap(),
            context(&masks, &empty),
            |_, _| panic!("callback")
        ),
        Err(MotionError::ActorOrder { index: 1 })
    );
    assert_eq!(actors, before);
    assert_eq!(cells[39 * 64 + 33], 1);
}
