use lemmings_core::*;

const N: usize = 64 * 170;
fn actor(action: MotionAction, frame: u8, direction: MotionDirection) -> Actor {
    Actor::from_state(
        ActorState {
            id: 0,
            born_iteration: 54,
            x: 32,
            y: 100,
            direction,
            action,
            frame,
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
fn restore(a: Actor, x: i32, y: i32, bricks: u8) -> Actor {
    Actor::from_state(
        ActorState {
            x,
            y,
            bricks_left: bricks,
            ..a.state()
        },
        a.rules(),
    )
    .unwrap()
}
fn context(empty: &[u8]) -> MiningContext<'_> {
    MiningContext::new(
        &[0; 104],
        BashingContext::new(&[0; 160], TerrainMask::new(64, 170, empty).unwrap()),
    )
}
fn tick(a: &mut Actor, cells: &mut [u8]) -> MotionOutcome {
    a.advance_with_jobs(
        &mut TerrainMaskMut::new(64, 170, cells).unwrap(),
        context(&[0; N]),
    )
    .unwrap()
}

#[test]
fn assignment_consumes_only_for_supported_live_actions_and_resets_animation() {
    for action in [
        MotionAction::Walking,
        MotionAction::Digging,
        MotionAction::Bashing,
        MotionAction::Mining,
        MotionAction::Shrugging,
        MotionAction::Building,
        MotionAction::Falling,
        MotionAction::Jumping,
    ] {
        let mut a = actor(action, 0, MotionDirection::Right);
        let before = a;
        let mut remaining = 2;
        let allowed = !matches!(
            action,
            MotionAction::Building | MotionAction::Falling | MotionAction::Jumping
        );
        assert_eq!(a.assign_builder(&mut remaining), allowed);
        if allowed {
            assert_eq!(
                (
                    a.state().action,
                    a.state().frame,
                    a.state().bricks_left,
                    a.state().new_digger
                ),
                (MotionAction::Building, 0, 12, false)
            );
            assert_eq!((a.state().x, a.state().y, remaining), (32, 100, 1));
        } else {
            assert_eq!((a, remaining), (before, 2));
        }
    }
    let mut a = actor(MotionAction::Walking, 0, MotionDirection::Right);
    let before = a;
    assert!(!a.assign_builder(&mut 0));
    assert_eq!(a, before);
    let mut removed = Actor::from_state(
        ActorState {
            y: 164,
            removed: Some(RemovalReason::BottomBoundary),
            ..before.state()
        },
        before.rules(),
    )
    .unwrap();
    assert!(!removed.assign_builder(&mut 1));
}

#[test]
fn assignment_checks_the_current_animations_head_including_digger_and_miner() {
    for (action, foot) in [
        (MotionAction::Walking, 10),
        (MotionAction::Shrugging, 10),
        (MotionAction::Bashing, 10),
        (MotionAction::Digging, 12),
        (MotionAction::Mining, 13),
    ] {
        for allowed in [false, true] {
            let mut a = restore(
                actor(action, 0, MotionDirection::Right),
                32,
                foot - 5 - i32::from(!allowed),
                0,
            );
            let saved = a;
            let mut remaining = 1;
            assert_eq!(
                a.assign_builder(&mut remaining),
                allowed,
                "{action:?} {allowed}"
            );
            if !allowed {
                assert_eq!((a, remaining), (saved, 1));
            }
        }
    }
}

#[test]
fn every_frame_and_brick_count_has_the_reference_six_pixel_addition_schedule() {
    for direction in [MotionDirection::Left, MotionDirection::Right] {
        for frame in 0..15 {
            for bricks in 1..=12 {
                let mut a = restore(
                    actor(MotionAction::Building, frame, direction),
                    32,
                    100,
                    bricks,
                );
                let mut cells = [0; N];
                let left = if direction == MotionDirection::Right {
                    32
                } else {
                    28
                };
                cells[99 * 64 + left + 2] = 1;
                let mut expected = cells;
                let lays = frame == 8 || (frame == 9 && bricks == 9);
                if lays {
                    expected[99 * 64 + left..99 * 64 + left + 6].fill(1);
                }
                let out = tick(&mut a, &mut cells);
                assert_eq!(cells, expected, "{direction:?} {frame} {bricks}");
                assert_eq!(out.check_objects, !lays);
                assert_eq!(
                    out.added_pixels.iter().flatten().count(),
                    if lays { 5 } else { 0 }
                );
                assert_eq!(
                    (a.state().frame, a.state().bricks_left),
                    (frame + 1, bricks)
                );
            }
        }
    }
}

#[test]
fn twelve_bricks_make_a_staircase_then_eight_shrug_ticks_finish_without_moving() {
    for direction in [MotionDirection::Left, MotionDirection::Right] {
        let mut a = actor(MotionAction::Building, 0, direction);
        let mut cells = [0; N];
        for step in 1..=192 {
            tick(&mut a, &mut cells);
            assert_eq!(a.state().x, 32 + (step / 16) * 2 * direction.delta());
            assert_eq!(a.state().y, 100 - step / 16);
        }
        assert_eq!(cells.iter().filter(|&&v| v == 1).count(), 72);
        assert_eq!(
            (a.state().action, a.state().frame, a.state().bricks_left),
            (MotionAction::Shrugging, 0, 0)
        );
        let position = (a.state().x, a.state().y);
        for step in 1..=8 {
            let out = tick(&mut a, &mut cells);
            assert_eq!(out.check_objects, step == 8);
            assert_eq!((a.state().x, a.state().y), position);
            assert_eq!(
                a.state().action,
                if step == 8 {
                    MotionAction::Walking
                } else {
                    MotionAction::Shrugging
                }
            );
        }
    }
}

#[test]
fn each_foot_substep_and_the_forward_head_pixel_can_stop_and_turn_the_builder() {
    for direction in [MotionDirection::Left, MotionDirection::Right] {
        for (offset, height) in [(1, 98), (2, 98), (4, 90)] {
            let mut a = actor(MotionAction::Building, 15, direction);
            let mut cells = [0; N];
            cells[height * 64 + (32 + offset * direction.delta()) as usize] = 1;
            let before = cells;
            assert!(tick(&mut a, &mut cells).check_objects);
            assert_eq!(
                (a.state().action, a.state().frame, a.state().bricks_left),
                (MotionAction::Walking, 0, 0)
            );
            assert_ne!(a.state().direction, direction);
            assert_eq!(
                (a.state().x, a.state().y),
                (32 + offset.min(2) * direction.delta(), 99)
            );
            assert_eq!(cells, before);
        }
    }
}

#[test]
fn final_brick_shrugging_precedes_forward_head_and_second_boundary_checks() {
    let mut a = restore(
        actor(MotionAction::Building, 15, MotionDirection::Right),
        32,
        100,
        1,
    );
    let mut cells = [0; N];
    cells[90 * 64 + 36] = 1;
    tick(&mut a, &mut cells);
    assert_eq!(
        (a.state().action, a.state().direction),
        (MotionAction::Shrugging, MotionDirection::Right)
    );
}

#[test]
fn boundary_checks_include_zero_and_keep_the_reference_substep_order() {
    for (x, direction, expected) in [
        (1, MotionDirection::Left, 0),
        (0, MotionDirection::Left, -1),
        (1647, MotionDirection::Right, 1648),
        (1646, MotionDirection::Right, 1648),
    ] {
        let mut a = restore(actor(MotionAction::Building, 15, direction), x, 100, 12);
        tick(&mut a, &mut [0; N]);
        assert_eq!(
            (a.state().x, a.state().y, a.state().action),
            (expected, 99, MotionAction::Walking)
        );
        assert_ne!(a.state().direction, direction);
    }
}

#[test]
fn high_builder_changes_to_walking_before_top_correction_uses_the_new_foot() {
    for (y, expected_y, action, direction) in [
        (9, 8, MotionAction::Building, MotionDirection::Right),
        (8, 7, MotionAction::Walking, MotionDirection::Right),
        (5, 3, MotionAction::Walking, MotionDirection::Left),
    ] {
        let mut a = restore(
            actor(MotionAction::Building, 15, MotionDirection::Right),
            32,
            y,
            12,
        );
        tick(&mut a, &mut [0; N]);
        assert_eq!(
            (a.state().y, a.state().action, a.state().direction),
            (expected_y, action, direction)
        );
    }
}

#[test]
fn arithmetic_failure_preserves_state_and_pixels_and_immutable_motion_rejects_building() {
    for (x, y, frame, direction) in [
        (i32::MAX, 100, 15, MotionDirection::Right),
        (32, i32::MIN, 15, MotionDirection::Right),
        (i32::MAX, 100, 8, MotionDirection::Right),
        (i32::MIN, 100, 8, MotionDirection::Left),
        (32, i32::MIN, 8, MotionDirection::Right),
    ] {
        let mut a = restore(actor(MotionAction::Building, frame, direction), x, y, 12);
        let saved = a;
        let mut cells = [0; N];
        assert_eq!(
            a.advance_with_jobs(
                &mut TerrainMaskMut::new(64, 170, &mut cells).unwrap(),
                context(&[0; N])
            ),
            Err(MotionError::CoordinateOverflow)
        );
        assert_eq!((a, cells), (saved, [0; N]));
        assert_eq!(
            a.advance(TerrainMask::new(64, 170, &[0; N]).unwrap()),
            Err(MotionError::MutableTerrainRequired)
        );
    }
}

#[test]
fn restored_brick_count_is_valid_only_in_building_and_within_one_to_twelve() {
    for (action, count, valid) in [
        (MotionAction::Building, 0, false),
        (MotionAction::Building, 13, false),
        (MotionAction::Building, 1, true),
        (MotionAction::Building, 12, true),
        (MotionAction::Walking, 1, false),
        (MotionAction::Shrugging, 1, false),
        (MotionAction::Shrugging, 0, true),
    ] {
        let state = ActorState {
            bricks_left: count,
            ..actor(action, 0, MotionDirection::Right).state()
        };
        assert_eq!(
            Actor::from_state(state, MotionRules::default()).is_ok(),
            valid
        );
    }
}

#[test]
fn building_and_shrugging_can_switch_to_every_other_implemented_job() {
    let empty = [0; N];
    for action in [MotionAction::Building, MotionAction::Shrugging] {
        for job in 0..3 {
            let mut a = actor(action, 4, MotionDirection::Right);
            let mut remaining = 1;
            assert!(match job {
                0 => a.assign_digger(&mut remaining, TerrainMask::new(64, 170, &empty).unwrap()),
                1 => a.assign_basher(&mut remaining, context(&empty).objects),
                _ => a.assign_miner(&mut remaining, context(&empty)),
            });
            assert_eq!(
                (remaining, a.state().bricks_left, a.state().frame),
                (0, 0, 0)
            );
        }
    }
}

#[test]
fn later_walker_observes_added_brick_and_duplicate_ids_reject_before_any_edit() {
    let a = actor(MotionAction::Building, 8, MotionDirection::Right);
    let b = Actor::from_state(
        ActorState {
            id: 1,
            y: 99,
            ..actor(MotionAction::Walking, 0, MotionDirection::Right).state()
        },
        MotionRules::default(),
    )
    .unwrap();
    let mut actors = [a, b];
    let mut cells = [0; N];
    step_actors_with_jobs(
        &mut actors,
        &mut TerrainMaskMut::new(64, 170, &mut cells).unwrap(),
        context(&[0; N]),
        |_, _| {},
    )
    .unwrap();
    assert_eq!(
        (actors[1].state().action, actors[1].state().y),
        (MotionAction::Walking, 99)
    );
    let mut duplicate = [a, a];
    let before = duplicate;
    cells.fill(0);
    assert_eq!(
        step_actors_with_jobs(
            &mut duplicate,
            &mut TerrainMaskMut::new(64, 170, &mut cells).unwrap(),
            context(&[0; N]),
            |_, _| panic!("callback")
        ),
        Err(MotionError::ActorOrder { index: 1 })
    );
    assert_eq!((duplicate, cells), (before, [0; N]));
}
