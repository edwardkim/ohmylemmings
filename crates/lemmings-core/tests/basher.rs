use lemmings_core::*;

const N: usize = 64 * 170;
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
fn view(cells: &[u8]) -> TerrainMask<'_> {
    TerrainMask::new(64, 170, cells).unwrap()
}
fn context<'a>(masks: &'a [u8; 160], steel: &'a [u8]) -> BashingContext<'a> {
    BashingContext::new(masks, view(steel))
}
fn tick(a: &mut Actor, cells: &mut [u8], c: BashingContext<'_>) -> MotionOutcome {
    a.advance_with_skills(&mut TerrainMaskMut::new(64, 170, cells).unwrap(), c)
        .unwrap()
}

#[test]
fn assignment_checks_front_direction_inventory_and_current_job() {
    let empty = [0; N];
    let masks = [0; 160];
    for direction in [MotionDirection::Left, MotionDirection::Right] {
        for restriction in 0..3 {
            let mut map = empty;
            map[32 * 64 + (32 + 8 * direction.delta()) as usize] = 1;
            let mut c = context(&masks, &empty);
            match restriction {
                0 => c.steel = view(&map),
                1 => c.one_way_left = Some(view(&map)),
                _ => c.one_way_right = Some(view(&map)),
            }
            let mut a = actor(MotionAction::Walking, 3, direction);
            let before = a;
            let mut count = 2;
            let allowed = restriction
                == if direction == MotionDirection::Left {
                    1
                } else {
                    2
                };
            assert_eq!(a.assign_basher(&mut count, c), allowed);
            assert_eq!(count, if allowed { 1 } else { 2 });
            if !allowed {
                assert_eq!(a, before);
            }
        }
    }
    for action in [
        MotionAction::Walking,
        MotionAction::Digging,
        MotionAction::Falling,
        MotionAction::Jumping,
        MotionAction::Bashing,
    ] {
        let mut a = actor(action, 0, MotionDirection::Right);
        let mut count = 2;
        let allowed = matches!(action, MotionAction::Walking | MotionAction::Digging);
        assert_eq!(
            a.assign_basher(&mut count, context(&masks, &empty)),
            allowed
        );
        if allowed {
            assert_eq!((a.state().frame, a.state().new_digger), (0, false));
        }
    }
    let mut a = actor(MotionAction::Walking, 0, MotionDirection::Right);
    let before = a;
    assert!(!a.assign_basher(&mut 0, context(&masks, &empty)));
    assert_eq!(a, before);
}

#[test]
fn directional_masks_select_stored_frames_and_clip_without_mirroring() {
    let empty = [0; N];
    let mut masks = [0; 160];
    for i in 0..8 {
        masks[i * 20] = 128 >> i;
    }
    for direction in [MotionDirection::Right, MotionDirection::Left] {
        for next in [2, 3, 4, 5, 18, 19, 20, 21] {
            let mut a = actor(MotionAction::Bashing, next - 1, direction);
            let mut cells = [1; N];
            tick(&mut a, &mut cells, context(&masks, &empty));
            let bit = usize::from((next % 16) - 2)
                + if direction == MotionDirection::Left {
                    4
                } else {
                    0
                };
            assert_eq!(cells.iter().filter(|&&v| v == 0).count(), 1);
            assert_eq!(cells[30 * 64 + 24 + bit], 0);
        }
    }
    let mut a = Actor::from_state(
        ActorState {
            x: 0,
            y: 0,
            ..actor(MotionAction::Bashing, 1, MotionDirection::Right).state()
        },
        MotionRules::default(),
    )
    .unwrap();
    let mut cells = [1; N];
    tick(&mut a, &mut cells, context(&[255; 160], &empty));
    assert_eq!(cells, [1; N]); // mask ends at y=-1; it does not clamp like dig rows
}

#[test]
fn only_frame_five_checks_four_pixels_after_applying_the_mask() {
    let empty = [0; N];
    let mut masks = [0; 160];
    // Left-facing last mask removes forward support at x=24,y=34.
    masks[80 + 60 + 4 * 2] = 128;
    for previous in [4, 20] {
        let mut cells = empty;
        cells[34 * 64 + 24] = 1;
        let mut a = actor(MotionAction::Bashing, previous, MotionDirection::Left);
        let out = tick(&mut a, &mut cells, context(&masks, &empty));
        assert!(!out.check_objects);
        assert_eq!(cells, empty);
        assert_eq!(
            a.state().action,
            if previous == 4 {
                MotionAction::Walking
            } else {
                MotionAction::Bashing
            }
        );
    }
    for offset in 0..4 {
        let mut cells = empty;
        cells[34 * 64 + 40 + offset] = 1;
        let mut a = actor(MotionAction::Bashing, 4, MotionDirection::Right);
        tick(&mut a, &mut cells, context(&[0; 160], &empty));
        assert_eq!(a.state().action, MotionAction::Bashing);
    }
}

#[test]
fn movement_checks_three_floor_pixels_then_front_restrictions() {
    let empty = [0; N];
    let masks = [0; 160];
    for direction in [MotionDirection::Left, MotionDirection::Right] {
        for drop in 0..4 {
            let x = 32 + direction.delta();
            let mut cells = empty;
            if drop < 3 {
                cells[(40 + drop) * 64 + x as usize] = 1;
            }
            let mut metal = empty;
            metal[(32 + drop.min(2)) * 64 + (x + 8 * direction.delta()) as usize] = 1;
            let mut a = actor(MotionAction::Bashing, 10, direction);
            assert!(tick(&mut a, &mut cells, context(&masks, &metal)).check_objects);
            assert_eq!((a.state().x, a.state().y), (x, 40 + drop as i32));
            assert_eq!(
                a.state().action,
                if drop == 3 {
                    MotionAction::Falling
                } else {
                    MotionAction::Walking
                }
            );
            assert_eq!(
                a.state().direction,
                if drop == 3 {
                    direction
                } else if direction == MotionDirection::Left {
                    MotionDirection::Right
                } else {
                    MotionDirection::Left
                }
            );
        }
    }
}

#[test]
fn full_cycle_has_ten_moves_eight_masks_and_wraps_the_animation() {
    let empty = [0; N];
    let mut cells = [1; N];
    let mut a = actor(MotionAction::Bashing, 0, MotionDirection::Right);
    for next in 1..=32 {
        let out = tick(&mut a, &mut cells, context(&[0; 160], &empty));
        assert_eq!(out.check_objects, matches!(next % 16, 11..=15));
        assert_eq!(a.state().frame, next % 32);
        assert_eq!(a.state().end_of_animation, next == 32);
    }
    assert_eq!((a.state().x, a.state().y), (42, 40));
}

#[test]
fn boundaries_turn_and_arithmetic_failures_preserve_state_and_pixels() {
    let empty = [0; N];
    for (x, direction, expected) in [
        (0, MotionDirection::Left, -1),
        (1647, MotionDirection::Right, 1648),
    ] {
        let mut a = Actor::from_state(
            ActorState {
                x,
                ..actor(MotionAction::Bashing, 10, direction).state()
            },
            MotionRules::default(),
        )
        .unwrap();
        let mut cells = [1; N];
        tick(&mut a, &mut cells, context(&[0; 160], &empty));
        assert_eq!(
            (a.state().x, a.state().action, a.state().frame),
            (expected, MotionAction::Walking, 0)
        );
    }
    for (x, y, frame) in [
        (i32::MAX, 40, 10),
        (i32::MIN, 40, 1),
        (32, i32::MIN, 1),
        (32, i32::MAX, 10),
    ] {
        let mut a = Actor::from_state(
            ActorState {
                x,
                y,
                ..actor(MotionAction::Bashing, frame, MotionDirection::Right).state()
            },
            MotionRules::default(),
        )
        .unwrap();
        let before = a;
        let mut cells = [0; N];
        assert!(
            a.advance_with_skills(
                &mut TerrainMaskMut::new(64, 170, &mut cells).unwrap(),
                context(&[255; 160], &empty)
            )
            .is_err()
        );
        assert_eq!((a, cells), (before, [0; N]));
        assert_eq!(
            a.advance(view(&empty)),
            Err(MotionError::MutableTerrainRequired)
        );
    }
}

#[test]
fn digger_can_replace_basher_and_batch_order_is_validated_before_edits() {
    let empty = [0; N];
    let mut a = actor(MotionAction::Bashing, 1, MotionDirection::Right);
    assert!(a.assign_digger(&mut 1, view(&empty)));
    assert!(a.state().new_digger);
    let b = actor(MotionAction::Bashing, 1, MotionDirection::Right);
    let mut actors = [b, b];
    let before = actors;
    let mut cells = [1; N];
    assert_eq!(
        step_actors_with_skills(
            &mut actors,
            &mut TerrainMaskMut::new(64, 170, &mut cells).unwrap(),
            context(&[255; 160], &empty),
            |_, _| panic!("callback")
        ),
        Err(MotionError::ActorOrder { index: 1 })
    );
    assert_eq!(actors, before);
    assert_eq!(cells, [1; N]);
}

#[test]
fn every_frame_drop_and_front_object_matches_the_directional_truth_table() {
    let empty = [0; N];
    for direction in [MotionDirection::Left, MotionDirection::Right] {
        for frame in 0..32 {
            for drop in 0..4 {
                for object in 0..4 {
                    let next = (frame + 1) % 32;
                    let moving = matches!(next % 16, 11..=15);
                    let x = 32 + direction.delta();
                    let mut cells = empty;
                    cells[34 * 64 + (32 + direction.delta() * 8) as usize] = 1;
                    if drop < 3 {
                        cells[(40 + drop) * 64 + x as usize] = 1;
                    }
                    let mut map = empty;
                    map[(32 + drop) * 64 + (x + 8 * direction.delta()) as usize] = 1;
                    let masks = [0; 160];
                    let mut c = context(&masks, &empty);
                    match object {
                        1 => c.steel = view(&map),
                        2 => c.one_way_left = Some(view(&map)),
                        3 => c.one_way_right = Some(view(&map)),
                        _ => {}
                    }
                    let blocked = object == 1
                        || (object == 2 && direction == MotionDirection::Right)
                        || (object == 3 && direction == MotionDirection::Left);
                    let mut a = actor(MotionAction::Bashing, frame, direction);
                    let out = tick(&mut a, &mut cells, c);
                    let expected = if !moving {
                        MotionAction::Bashing
                    } else if drop == 3 {
                        MotionAction::Falling
                    } else if blocked {
                        MotionAction::Walking
                    } else {
                        MotionAction::Bashing
                    };
                    assert_eq!(
                        a.state().action,
                        expected,
                        "{direction:?} {frame} {drop} {object}"
                    );
                    assert_eq!(out.check_objects, moving);
                    assert_eq!(a.state().x, if moving { x } else { 32 });
                    assert_eq!(a.state().y, 40 + if moving { drop as i32 } else { 0 });
                }
            }
        }
    }
}

#[test]
fn later_walker_observes_basher_deletion_and_later_error_keeps_earlier_commits() {
    let empty = [0; N];
    let mut masks = [0; 160];
    masks[19] = 64;
    let first = actor(MotionAction::Bashing, 1, MotionDirection::Right);
    let mut cells = empty;
    cells[39 * 64 + 33] = 1;
    let second = Actor::from_state(
        ActorState {
            id: 1,
            y: 39,
            ..actor(MotionAction::Walking, 0, MotionDirection::Right).state()
        },
        MotionRules::default(),
    )
    .unwrap();
    let mut actors = [first, second];
    step_actors_with_skills(
        &mut actors,
        &mut TerrainMaskMut::new(64, 170, &mut cells).unwrap(),
        context(&masks, &empty),
        |_, _| {},
    )
    .unwrap();
    assert_eq!(actors[1].state().action, MotionAction::Falling);
    let broken = Actor::from_state(
        ActorState {
            id: 1,
            x: i32::MAX,
            ..second.state()
        },
        MotionRules::default(),
    )
    .unwrap();
    let mut actors = [first, broken];
    cells[39 * 64 + 33] = 1;
    assert_eq!(
        step_actors_with_skills(
            &mut actors,
            &mut TerrainMaskMut::new(64, 170, &mut cells).unwrap(),
            context(&masks, &empty),
            |_, _| {}
        ),
        Err(MotionError::CoordinateOverflow)
    );
    assert_eq!(cells[39 * 64 + 33], 0);
    assert_eq!(actors[1], broken);
    assert_eq!(actors[0].state().frame, 2);
}
