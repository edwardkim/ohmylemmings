use lemmings_core::*;

fn walker(x: i32, y: i32) -> Actor {
    Actor::from_state(
        ActorState {
            id: 0,
            born_iteration: 54,
            x,
            y,
            direction: MotionDirection::Right,
            action: MotionAction::Walking,
            frame: 3,
            end_of_animation: true,
            fallen: 0,
            new_digger: false,
            bricks_left: 0,
            blocker_saved: None,
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
fn steel(cells: &[u8]) -> TerrainMask<'_> {
    TerrainMask::new(32, 170, cells).unwrap()
}

#[test]
fn assignment_consumes_only_on_acceptance_and_requires_walking_off_steel() {
    let empty = [0; 32 * 170];
    let mut a = walker(16, 20);
    let before = a;
    let mut count = 0;
    assert!(!a.assign_digger(&mut count, steel(&empty)));
    assert_eq!(a, before);
    count = 2;
    let mut metal = empty;
    metal[20 * 32 + 16] = 1;
    assert!(!a.assign_digger(&mut count, steel(&metal)));
    assert_eq!((a, count), (before, 2));
    assert!(a.assign_digger(&mut count, steel(&empty)));
    assert_eq!(
        (a.state().action, a.state().frame, count),
        (MotionAction::Digging, 0, 1)
    );
    assert!(!a.assign_digger(&mut count, steel(&empty)));
    assert_eq!(count, 1);
}

#[test]
fn first_update_clears_three_nine_pixel_rows_then_digs_every_eight_ticks() {
    let empty = [0; 32 * 170];
    let mut cells = [1; 32 * 170];
    let mut a = walker(16, 20);
    assert!(a.assign_digger(&mut 1, steel(&empty)));
    let mut t = TerrainMaskMut::new(32, 170, &mut cells).unwrap();
    a.advance_mut(&mut t, steel(&empty)).unwrap();
    assert_eq!((a.state().y, a.state().frame), (21, 0));
    assert_eq!(
        t.as_read_only()
            .as_bytes()
            .iter()
            .filter(|&&v| v == 0)
            .count(),
        27
    );
    for row in 18..=20 {
        assert!((12..=20).all(|x| !t.as_read_only().is_solid(x, row)));
        assert!(t.as_read_only().is_solid(11, row));
        assert!(t.as_read_only().is_solid(21, row));
    }
    for _ in 0..7 {
        a.advance_mut(&mut t, steel(&empty)).unwrap();
    }
    assert_eq!((a.state().y, a.state().frame), (21, 7));
    a.advance_mut(&mut t, steel(&empty)).unwrap();
    assert_eq!((a.state().y, a.state().frame), (22, 8));
    for _ in 0..8 {
        a.advance_mut(&mut t, steel(&empty)).unwrap();
    }
    assert_eq!((a.state().y, a.state().frame), (23, 0));
    assert!(!a.state().end_of_animation);
}

#[test]
fn any_of_the_nine_pixels_supports_digging_even_if_the_center_is_empty() {
    let empty = [0; 32 * 170];
    for x in 12..=20 {
        let mut cells = empty;
        cells[20 * 32 + x] = 1;
        let mut a = walker(16, 20);
        a.assign_digger(&mut 1, steel(&empty));
        a.advance_mut(
            &mut TerrainMaskMut::new(32, 170, &mut cells).unwrap(),
            steel(&empty),
        )
        .unwrap();
        assert_eq!(a.state().action, MotionAction::Digging);
    }
}

#[test]
fn empty_row_falls_and_steel_at_new_foot_walks_after_removing_the_row() {
    let empty = [0; 32 * 170];
    for supported in [false, true] {
        let mut cells = if supported { [1; 32 * 170] } else { empty };
        let mut metal = empty;
        metal[21 * 32 + 16] = 1;
        let mut a = walker(16, 20);
        a.assign_digger(&mut 1, steel(&empty));
        a.advance_mut(
            &mut TerrainMaskMut::new(32, 170, &mut cells).unwrap(),
            steel(&metal),
        )
        .unwrap();
        assert_eq!(
            a.state().action,
            if supported {
                MotionAction::Walking
            } else {
                MotionAction::Falling
            }
        );
        assert_eq!(a.state().fallen, if supported { 0 } else { 3 });
        assert_eq!(a.state().y, 21);
        assert_eq!(cells[20 * 32 + 16], 0);
    }
}

#[test]
fn initial_clear_clamps_negative_rows_and_clips_both_horizontal_edges() {
    let empty = [0; 32 * 170];
    for x in [0, 31] {
        let mut cells = [1; 32 * 170];
        let mut a = walker(x, 0);
        a.assign_digger(&mut 1, steel(&empty));
        a.advance_mut(
            &mut TerrainMaskMut::new(32, 170, &mut cells).unwrap(),
            steel(&empty),
        )
        .unwrap();
        assert_eq!(cells.iter().filter(|&&v| v == 0).count(), 5);
        assert_eq!(a.state().action, MotionAction::Falling);
    }
}

#[test]
fn arithmetic_failure_preserves_actor_and_terrain_and_immutable_api_rejects_digger() {
    let empty = [0; 32 * 170];
    for (x, y) in [
        (i32::MIN, 20),
        (i32::MAX, 20),
        (16, i32::MIN),
        (16, i32::MAX),
    ] {
        let mut cells = [1; 32 * 170];
        let mut a = walker(x, y);
        a.assign_digger(&mut 1, steel(&empty));
        let before = a;
        assert!(a.advance(steel(&cells)).is_err());
        assert!(
            a.advance_mut(
                &mut TerrainMaskMut::new(32, 170, &mut cells).unwrap(),
                steel(&empty)
            )
            .is_err()
        );
        assert_eq!(a, before);
        assert_eq!(cells, [1; 32 * 170]);
    }
}

#[test]
fn bottom_removal_is_delayed_until_dig_frame_and_skips_current_row() {
    let empty = [0; 32 * 170];
    let mut cells = [1; 32 * 170];
    let mut a = walker(16, 163);
    a.assign_digger(&mut 1, steel(&empty));
    let out = a
        .advance_mut(
            &mut TerrainMaskMut::new(32, 170, &mut cells).unwrap(),
            steel(&empty),
        )
        .unwrap();
    assert_eq!(out.removal, Some(RemovalReason::BottomBoundary));
    assert_eq!(a.state().y, 164);
    assert_eq!(cells[163 * 32 + 16], 1);
    let before = cells;
    assert_eq!(
        a.advance_mut(
            &mut TerrainMaskMut::new(32, 170, &mut cells).unwrap(),
            steel(&empty)
        )
        .unwrap(),
        MotionOutcome::default()
    );
    assert_eq!(cells, before);
}

#[test]
fn later_walker_sees_the_hole_made_by_an_earlier_digger() {
    let empty = [0; 32 * 170];
    let mut cells = empty;
    cells[20 * 32..21 * 32].fill(1);
    let mut a = walker(16, 20);
    a.assign_digger(&mut 1, steel(&empty));
    let s = ActorState {
        id: 1,
        x: 15,
        ..walker(16, 20).state()
    };
    let mut actors = [a, Actor::from_state(s, MotionRules::default()).unwrap()];
    step_actors_mut(
        &mut actors,
        &mut TerrainMaskMut::new(32, 170, &mut cells).unwrap(),
        steel(&empty),
        |_, _| {},
    )
    .unwrap();
    assert_eq!(actors[1].state().action, MotionAction::Falling);
    assert_eq!(actors[1].state().y, 24);
}

#[test]
fn falling_jumping_splatting_and_removed_actors_cannot_consume_a_skill() {
    let empty = [0; 32 * 170];
    for action in [
        MotionAction::Falling,
        MotionAction::Jumping,
        MotionAction::Splatting,
    ] {
        let state = ActorState {
            action,
            frame: 0,
            end_of_animation: false,
            direction: if action == MotionAction::Splatting {
                MotionDirection::Stopped
            } else {
                MotionDirection::Left
            },
            ..walker(16, 20).state()
        };
        let mut a = Actor::from_state(state, MotionRules::default()).unwrap();
        let before = a;
        let mut remaining = 1;
        assert!(!a.assign_digger(&mut remaining, steel(&empty)));
        assert_eq!((a, remaining), (before, 1));
    }
    let state = ActorState {
        y: 164,
        removed: Some(RemovalReason::BottomBoundary),
        ..walker(16, 20).state()
    };
    let mut a = Actor::from_state(state, MotionRules::default()).unwrap();
    assert!(!a.assign_digger(&mut 1, steel(&empty)));
}

#[test]
fn restore_validates_new_digger_flag_and_immutable_batch_is_atomic() {
    for state in [
        ActorState {
            new_digger: true,
            bricks_left: 0,
            ..walker(16, 20).state()
        },
        ActorState {
            action: MotionAction::Digging,
            frame: 1,
            new_digger: true,
            bricks_left: 0,
            ..walker(16, 20).state()
        },
    ] {
        assert_eq!(
            Actor::from_state(state, MotionRules::default()),
            Err(MotionError::InvalidNewDigger)
        );
    }
    let empty = [0; 32 * 170];
    let mut a = walker(16, 20);
    a.assign_digger(&mut 1, steel(&empty));
    let mut actors = [a];
    let before = actors;
    assert_eq!(
        step_actors(&mut actors, steel(&empty), |_, _| panic!(
            "unexpected callback"
        )),
        Err(MotionError::MutableTerrainRequired)
    );
    assert_eq!(actors, before);
    assert!(
        Actor::from_state(a.state(), MotionRules::default())
            .unwrap()
            .state()
            .new_digger
    );
}

#[test]
fn mutable_batch_rejects_duplicate_ids_before_any_deletion_or_callback() {
    let empty = [0; 32 * 170];
    let mut cells = [1; 32 * 170];
    let mut a = walker(16, 20);
    a.assign_digger(&mut 1, steel(&empty));
    let mut actors = [a, a];
    let before = actors;
    assert_eq!(
        step_actors_mut(
            &mut actors,
            &mut TerrainMaskMut::new(32, 170, &mut cells).unwrap(),
            steel(&empty),
            |_, _| panic!("unexpected callback")
        ),
        Err(MotionError::ActorOrder { index: 1 })
    );
    assert_eq!(actors, before);
    assert_eq!(cells, [1; 32 * 170]);
}

#[test]
fn every_nine_bit_support_pattern_matches_the_frame_and_steel_truth_table() {
    for pattern in 0u16..512 {
        for frame in 0..16 {
            for metal in [false, true] {
                let mut cells = [0; 32 * 170];
                for bit in 0..9 {
                    cells[20 * 32 + 12 + bit] = ((pattern >> bit) & 1) as u8;
                }
                let before = cells;
                let mut metal_cells = [0; 32 * 170];
                metal_cells[21 * 32 + 16] = u8::from(metal);
                let state = ActorState {
                    action: MotionAction::Digging,
                    frame,
                    end_of_animation: false,
                    ..walker(16, 20).state()
                };
                let mut a = Actor::from_state(state, MotionRules::default()).unwrap();
                a.advance_mut(
                    &mut TerrainMaskMut::new(32, 170, &mut cells).unwrap(),
                    steel(&metal_cells),
                )
                .unwrap();
                let next_frame = (frame + 1) % 16;
                if next_frame == 0 || next_frame == 8 {
                    assert_eq!(cells, [0; 32 * 170]);
                    assert_eq!(a.state().y, 21);
                    assert_eq!(
                        a.state().action,
                        if pattern == 0 {
                            MotionAction::Falling
                        } else if metal {
                            MotionAction::Walking
                        } else {
                            MotionAction::Digging
                        }
                    );
                } else {
                    assert_eq!(cells, before);
                    assert_eq!(
                        (a.state().y, a.state().frame, a.state().action),
                        (20, next_frame, MotionAction::Digging)
                    );
                }
            }
        }
    }
}
