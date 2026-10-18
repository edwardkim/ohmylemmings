extern crate std;
use super::*;
use crate::{
    FixedTickClock, InputCommand, InputSchedule, StartupCommand, StartupConfig, StartupEvent,
    StartupScheduler, TimedInput,
};
use core::time::Duration;
use std::{vec, vec::Vec};
#[path = "golden.rs"]
mod golden;

fn state() -> ActorState {
    ActorState {
        id: 0,
        born_iteration: 54,
        x: 2,
        y: 20,
        direction: MotionDirection::Right,
        action: MotionAction::Walking,
        frame: 0,
        end_of_animation: false,
        fallen: 0,
        removed: None,
    }
}
fn actor(s: ActorState) -> Actor {
    Actor::from_state(s, MotionRules::default()).unwrap()
}
fn board(w: usize, h: usize, solid: impl Fn(usize, usize) -> bool) -> Vec<u8> {
    (0..h)
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .map(|(x, y)| u8::from(solid(x, y)))
        .collect()
}

#[test]
fn birth_uses_selected_original_entrance_and_preserves_stable_identity_and_rules() {
    let entrances = [
        EntrancePoint { x: 0, y: 0 },
        EntrancePoint { x: 40, y: -14 },
    ];
    for enabled in [false, true] {
        let request = SpawnRequest {
            actor_id: u32::MAX,
            entrance_index: 1,
            iteration: u64::MAX,
        };
        let rules = MotionRules {
            faller_starts_with_three: enabled,
        };
        let a = Actor::from_spawn(request, &entrances, rules).unwrap();
        assert_eq!(a.rules(), rules);
        assert_eq!(
            a.state(),
            ActorState {
                id: u32::MAX,
                born_iteration: u64::MAX,
                x: 64,
                y: 0,
                direction: MotionDirection::Right,
                action: MotionAction::Falling,
                frame: 0,
                end_of_animation: false,
                fallen: if enabled { 3 } else { 0 },
                removed: None
            }
        );
    }
    assert!(MotionRules::default().faller_starts_with_three);
}

#[test]
fn birth_rejects_bad_entrance_selection_and_coordinate_overflow_without_touching_input() {
    let r = SpawnRequest {
        actor_id: 0,
        entrance_index: 0,
        iteration: 54,
    };
    let rules = MotionRules::default();
    assert_eq!(
        Actor::from_spawn(r, &[], rules),
        Err(MotionError::EntranceIndex(0))
    );
    assert_eq!(
        Actor::from_spawn(r, &[EntrancePoint { x: 0, y: 0 }; 5], rules),
        Err(MotionError::EntranceCount(5))
    );
    let entrances = [
        EntrancePoint { x: 0, y: 0 },
        EntrancePoint {
            x: i32::MAX,
            y: i32::MAX,
        },
    ];
    assert!(Actor::from_spawn(r, &entrances, rules).is_ok());
    assert_eq!(
        Actor::from_spawn(
            SpawnRequest {
                entrance_index: 2,
                ..r
            },
            &entrances,
            rules
        ),
        Err(MotionError::EntranceIndex(2))
    );
    assert_eq!(
        Actor::from_spawn(
            SpawnRequest {
                entrance_index: 1,
                ..r
            },
            &entrances,
            rules
        ),
        Err(MotionError::CoordinateOverflow)
    );
    assert_eq!(
        Actor::from_spawn(r, &[EntrancePoint { x: 0, y: i32::MAX }], rules),
        Err(MotionError::CoordinateOverflow)
    );
    assert_eq!(
        entrances[1],
        EntrancePoint {
            x: i32::MAX,
            y: i32::MAX
        }
    );
}

#[test]
fn restoration_rejects_all_invalid_frames_and_inconsistent_fields() {
    for action in [
        MotionAction::Walking,
        MotionAction::Jumping,
        MotionAction::Falling,
        MotionAction::Splatting,
    ] {
        for frame in action.frame_count()..=255 {
            let s = ActorState {
                action,
                frame,
                direction: if action == MotionAction::Splatting {
                    MotionDirection::Stopped
                } else {
                    MotionDirection::Right
                },
                ..state()
            };
            assert_eq!(
                Actor::from_state(s, MotionRules::default()),
                Err(MotionError::Frame { action, frame })
            );
        }
    }
    for s in [
        ActorState {
            direction: MotionDirection::Stopped,
            ..state()
        },
        ActorState {
            action: MotionAction::Splatting,
            ..state()
        },
    ] {
        assert_eq!(
            Actor::from_state(s, MotionRules::default()),
            Err(MotionError::Direction)
        );
    }
    assert_eq!(
        Actor::from_state(
            ActorState {
                fallen: 1,
                ..state()
            },
            MotionRules::default()
        ),
        Err(MotionError::InvalidFallen)
    );
    for s in [
        ActorState {
            removed: Some(RemovalReason::BottomBoundary),
            ..state()
        },
        ActorState {
            removed: Some(RemovalReason::SplattingFinished),
            ..state()
        },
        ActorState {
            action: MotionAction::Splatting,
            direction: MotionDirection::Stopped,
            frame: 15,
            end_of_animation: false,
            removed: Some(RemovalReason::SplattingFinished),
            ..state()
        },
    ] {
        assert_eq!(
            Actor::from_state(s, MotionRules::default()),
            Err(MotionError::InvalidRemoval)
        );
    }
}

#[test]
fn flat_walking_moves_one_pixel_and_wraps_animation_without_resetting_on_turn() {
    let pixels = board(8, 32, |_, y| y >= 20);
    let terrain = TerrainMask::new(8, 32, &pixels).unwrap();
    for direction in [MotionDirection::Left, MotionDirection::Right] {
        let mut a = actor(ActorState {
            direction,
            frame: 7,
            ..state()
        });
        assert_eq!(
            a.advance(terrain).unwrap(),
            MotionOutcome {
                check_objects: true,
                ..MotionOutcome::default()
            }
        );
        assert_eq!((a.state().x, a.state().y), (2 + direction.delta(), 20));
        assert_eq!((a.state().frame, a.state().end_of_animation), (0, true));
        a.advance(terrain).unwrap();
        assert_eq!((a.state().frame, a.state().end_of_animation), (1, false));
    }
    assert_eq!(pixels, board(8, 32, |_, y| y >= 20));
}

#[test]
fn rises_zero_through_six_choose_walk_or_jump_and_seven_turns_inside_the_wall() {
    for rise in 0..=7 {
        let pixels = board(8, 32, |x, y| x == 3 && (20 - rise..=20).contains(&y));
        let terrain = TerrainMask::new(8, 32, &pixels).unwrap();
        let mut a = actor(ActorState {
            frame: 4,
            ..state()
        });
        a.advance(terrain).unwrap();
        assert_eq!(a.state().x, 3);
        if rise == 7 {
            assert_eq!(
                (
                    a.state().y,
                    a.state().direction,
                    a.state().action,
                    a.state().frame
                ),
                (20, MotionDirection::Left, MotionAction::Walking, 5)
            );
        } else if rise >= 3 {
            assert_eq!(
                (a.state().y, a.state().action, a.state().frame),
                (18, MotionAction::Jumping, 0)
            );
        } else {
            assert_eq!(
                (a.state().y, a.state().action, a.state().frame),
                (20 - rise as i32, MotionAction::Walking, 5)
            );
        }
    }
}

#[test]
fn jumping_climbs_two_per_tick_then_walks_when_the_column_clears() {
    let pixels = board(8, 32, |x, y| x == 3 && (14..=20).contains(&y));
    let terrain = TerrainMask::new(8, 32, &pixels).unwrap();
    let mut a = actor(state());
    for y in [18, 16, 14] {
        a.advance(terrain).unwrap();
        assert_eq!(
            (a.state().x, a.state().y, a.state().action),
            (3, y, MotionAction::Jumping)
        );
    }
    a.advance(terrain).unwrap();
    assert_eq!(
        (
            a.state().y,
            a.state().action,
            a.state().frame,
            a.state().end_of_animation
        ),
        (14, MotionAction::Walking, 0, false)
    );
}

#[test]
fn gaps_one_to_three_stay_walking_and_four_enters_falling_with_profile_counter() {
    for gap in 1..=4 {
        let pixels = board(8, 32, |x, y| x == 3 && y == 20 + gap);
        let terrain = TerrainMask::new(8, 32, &pixels).unwrap();
        for enabled in [false, true] {
            let mut a = Actor::from_state(
                state(),
                MotionRules {
                    faller_starts_with_three: enabled,
                },
            )
            .unwrap();
            a.advance(terrain).unwrap();
            assert_eq!(a.state().y, 20 + gap as i32);
            assert_eq!(
                a.state().action,
                if gap == 4 {
                    MotionAction::Falling
                } else {
                    MotionAction::Walking
                }
            );
            assert_eq!(a.state().fallen, if gap == 4 && enabled { 3 } else { 0 });
        }
    }
}

#[test]
fn horizontal_borders_reverse_without_clamping_and_falling_does_not_apply_walker_wall_rules() {
    let pixels = board(8, 32, |_, y| y == 20);
    let terrain = TerrainMask::new(8, 32, &pixels).unwrap();
    for (x, direction, wanted, next) in [
        (0, MotionDirection::Left, -1, MotionDirection::Right),
        (1647, MotionDirection::Right, 1648, MotionDirection::Left),
    ] {
        let mut a = actor(ActorState {
            x,
            direction,
            frame: 7,
            ..state()
        });
        a.advance(terrain).unwrap();
        assert_eq!(
            (
                a.state().x,
                a.state().y,
                a.state().direction,
                a.state().frame,
                a.state().end_of_animation
            ),
            (wanted, 20, next, 0, true)
        );
    }
    let mut a = actor(ActorState {
        x: 1648,
        y: 10,
        action: MotionAction::Falling,
        fallen: 3,
        ..state()
    });
    a.advance(terrain).unwrap();
    assert_eq!(
        (a.state().x, a.state().y, a.state().direction),
        (1648, 13, MotionDirection::Right)
    );
}

#[test]
fn head_boundary_and_negative_clip_y_keep_the_reference_query_order() {
    let pixels = board(4, 8, |_, y| y == 0);
    let terrain = TerrainMask::new(4, 8, &pixels).unwrap();
    for y in [i32::MIN, -4, 4, 5] {
        let standing = board(4, 8, |_, row| row == if y < 0 { 0 } else { y as usize });
        let floor = TerrainMask::new(4, 8, &standing).unwrap();
        let mut a = actor(ActorState { x: 0, y, ..state() });
        a.advance(floor).unwrap();
        assert_eq!(
            (a.state().x, a.state().y, a.state().direction),
            (
                1,
                if y < 5 { 3 } else { 5 },
                if y < 5 {
                    MotionDirection::Left
                } else {
                    MotionDirection::Right
                }
            )
        );
    }
    let mut a = actor(ActorState {
        x: 1,
        y: -2,
        action: MotionAction::Falling,
        fallen: 3,
        ..state()
    });
    a.advance(terrain).unwrap();
    assert_eq!((a.state().y, a.state().action), (-2, MotionAction::Walking));
    let pixels = board(4, 8, |_, y| y == 1);
    let terrain = TerrainMask::new(4, 8, &pixels).unwrap();
    let mut a = actor(ActorState {
        x: 1,
        y: -2,
        action: MotionAction::Falling,
        fallen: 3,
        ..state()
    });
    a.advance(terrain).unwrap();
    assert_eq!((a.state().y, a.state().action), (-1, MotionAction::Walking));
}

#[test]
fn falling_advances_three_and_adds_three_only_after_a_full_air_step() {
    let cells = [0; 9];
    let terrain = TerrainMask::new(3, 3, &cells).unwrap();
    let mut a = actor(ActorState {
        action: MotionAction::Falling,
        frame: 3,
        fallen: 3,
        ..state()
    });
    a.advance(terrain).unwrap();
    assert_eq!(
        (
            a.state().x,
            a.state().y,
            a.state().fallen,
            a.state().frame,
            a.state().end_of_animation
        ),
        (2, 23, 6, 0, true)
    );
}

#[test]
fn landing_uses_the_old_fall_counter_and_new_splats_still_request_object_checks() {
    for distance in 0..=2 {
        let pixels = board(8, 32, |x, y| x == 2 && y == 20 + distance);
        let terrain = TerrainMask::new(8, 32, &pixels).unwrap();
        for fallen in [60, 61, u32::MAX] {
            let mut a = actor(ActorState {
                action: MotionAction::Falling,
                fallen,
                frame: 3,
                ..state()
            });
            let result = a.advance(terrain).unwrap();
            assert_eq!(a.state().y, 20 + distance as i32);
            assert!(result.check_objects);
            assert_eq!(result.removal, None);
            assert_eq!(a.state().removed, None);
            assert_eq!(
                (
                    a.state().frame,
                    a.state().end_of_animation,
                    a.state().fallen
                ),
                (0, false, 0)
            );
            assert_eq!(
                a.state().action,
                if fallen > 60 {
                    MotionAction::Splatting
                } else {
                    MotionAction::Walking
                }
            );
            assert_eq!(result.splat_started, fallen > 60);
            assert_eq!(
                a.state().direction,
                if fallen > 60 {
                    MotionDirection::Stopped
                } else {
                    MotionDirection::Right
                }
            );
        }
    }
}

#[test]
fn coordinate_and_fall_counter_errors_preserve_the_complete_actor() {
    let cells = [0; 9];
    let terrain = TerrainMask::new(3, 3, &cells).unwrap();
    for (s, error) in [
        (
            ActorState {
                x: i32::MAX,
                ..state()
            },
            MotionError::CoordinateOverflow,
        ),
        (
            ActorState {
                x: i32::MIN,
                direction: MotionDirection::Left,
                ..state()
            },
            MotionError::CoordinateOverflow,
        ),
        (
            ActorState {
                y: i32::MAX,
                action: MotionAction::Falling,
                fallen: 3,
                ..state()
            },
            MotionError::CoordinateOverflow,
        ),
        (
            ActorState {
                action: MotionAction::Falling,
                fallen: u32::MAX,
                ..state()
            },
            MotionError::FallCounterOverflow,
        ),
    ] {
        let mut a = actor(s);
        let before = a;
        assert_eq!(a.advance(terrain), Err(error));
        assert_eq!(a, before);
    }
}

#[test]
fn bottom_checks_occur_at_action_specific_positions_and_keep_removal_reasons() {
    let cells = [0; 9];
    let terrain = TerrainMask::new(3, 3, &cells).unwrap();
    for (action, y, wanted) in [
        (MotionAction::Falling, 163, 164),
        (MotionAction::Walking, 163, 167),
    ] {
        let mut a = actor(ActorState {
            action,
            y,
            fallen: if action == MotionAction::Falling {
                3
            } else {
                0
            },
            ..state()
        });
        let out = a.advance(terrain).unwrap();
        assert_eq!(a.state().y, wanted);
        assert_eq!(a.state().removed, Some(RemovalReason::BottomBoundary));
        assert_eq!(out.removal, Some(RemovalReason::BottomBoundary));
        assert!(!out.check_objects);
    }
    let pixels = board(8, 180, |_, y| y == 164);
    let terrain = TerrainMask::new(8, 180, &pixels).unwrap();
    let mut a = actor(ActorState { y: 162, ..state() });
    a.advance(terrain).unwrap();
    assert_eq!(
        (a.state().y, a.state().action, a.state().removed),
        (
            164,
            MotionAction::Walking,
            Some(RemovalReason::BottomBoundary)
        )
    );
}

#[test]
fn splat_removal_waits_sixteen_updates_then_removed_records_are_frozen() {
    let cells = [0; 1];
    let terrain = TerrainMask::new(1, 1, &cells).unwrap();
    let mut a = actor(ActorState {
        action: MotionAction::Splatting,
        direction: MotionDirection::Stopped,
        ..state()
    });
    for frame in 1..=15 {
        let out = a.advance(terrain).unwrap();
        assert_eq!(a.state().frame, frame);
        assert_eq!(out, MotionOutcome::default());
        assert_eq!(a.state().removed, None);
    }
    assert_eq!(
        a.advance(terrain).unwrap().removal,
        Some(RemovalReason::SplattingFinished)
    );
    assert!(a.state().end_of_animation);
    let before = a;
    assert_eq!(a.advance(terrain).unwrap(), MotionOutcome::default());
    assert_eq!(a, before);
    assert_eq!(Actor::from_state(a.state(), a.rules()).unwrap(), a);
}

#[test]
fn batch_updates_live_actors_in_strict_id_order_and_counts_each_removal_once() {
    let pixels = board(8, 32, |_, y| y == 20);
    let terrain = TerrainMask::new(8, 32, &pixels).unwrap();
    let mut actors = [
        actor(state()),
        actor(ActorState {
            id: 2,
            action: MotionAction::Splatting,
            direction: MotionDirection::Stopped,
            frame: 15,
            ..state()
        }),
        actor(ActorState {
            id: u32::MAX,
            y: 164,
            action: MotionAction::Falling,
            removed: Some(RemovalReason::BottomBoundary),
            ..state()
        }),
    ];
    let mut ids = Vec::new();
    let tick = step_actors(&mut actors, terrain, |s, _| ids.push(s.id)).unwrap();
    assert_eq!(ids, [0, 2]);
    assert_eq!(
        tick,
        ActorTick {
            active_before: 2,
            removed: 1,
            checking_objects: 1,
            splats_started: 0
        }
    );
    ids.clear();
    assert_eq!(
        step_actors(&mut actors, terrain, |s, _| ids.push(s.id))
            .unwrap()
            .removed,
        0
    );
    assert_eq!(ids, [0]);
    assert_eq!(
        step_actors(&mut [], terrain, |_, _| panic!("empty callback")).unwrap(),
        ActorTick::default()
    );
}

#[test]
fn batch_order_and_later_arithmetic_errors_are_atomic_before_any_callback() {
    let cells = [0; 1];
    let terrain = TerrainMask::new(1, 1, &cells).unwrap();
    for second in [
        actor(ActorState { id: 1, ..state() }),
        actor(ActorState { id: 0, ..state() }),
    ] {
        let mut actors = [actor(ActorState { id: 1, ..state() }), second];
        let before = actors;
        assert_eq!(
            step_actors(&mut actors, terrain, |_, _| panic!(
                "invalid order callback"
            )),
            Err(MotionError::ActorOrder { index: 1 })
        );
        assert_eq!(actors, before);
    }
    let mut actors = [
        actor(state()),
        actor(ActorState {
            id: 1,
            x: i32::MAX,
            ..state()
        }),
    ];
    let before = actors;
    assert_eq!(
        step_actors(&mut actors, terrain, |_, _| panic!("preflight callback")),
        Err(MotionError::CoordinateOverflow)
    );
    assert_eq!(actors, before);
}

fn action_code(action: MotionAction) -> u8 {
    match action {
        MotionAction::Walking => 0,
        MotionAction::Jumping => 1,
        MotionAction::Falling => 2,
        MotionAction::Splatting => 3,
    }
}
fn removal_code(reason: Option<RemovalReason>) -> u8 {
    match reason {
        None => 0,
        Some(RemovalReason::BottomBoundary) => 1,
        Some(RemovalReason::SplattingFinished) => 2,
    }
}
fn put64(out: &mut Vec<u8>, v: u64) {
    out.extend_from_slice(&v.to_le_bytes());
}
fn put32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}
fn motion_bytes(out: &mut Vec<u8>, s: ActorState) {
    out.extend_from_slice(&s.x.to_le_bytes());
    out.extend_from_slice(&s.y.to_le_bytes());
    out.extend_from_slice(&[
        s.direction.delta() as i8 as u8,
        action_code(s.action),
        s.frame,
        u8::from(s.end_of_animation),
    ]);
    put32(out, s.fallen);
    out.push(removal_code(s.removed));
}
fn actor_bytes(out: &mut Vec<u8>, s: ActorState) {
    put32(out, s.id);
    put64(out, s.born_iteration);
    motion_bytes(out, s);
}
fn outcome_bytes(out: &mut Vec<u8>, r: MotionOutcome) {
    out.extend_from_slice(&[
        u8::from(r.check_objects),
        removal_code(r.removal),
        u8::from(r.splat_started),
    ]);
}
fn fnv(bytes: &[u8], mut value: u64) -> u64 {
    for &byte in bytes {
        value = (value ^ u64::from(byte)).wrapping_mul(0x100000001b3);
    }
    value
}
const OFFSET: u64 = 0xcbf29ce484222325;

#[test]
fn exhaustive_motion_matches_independent_per_terrain_and_rule_goldens() {
    let mut total = 0;
    let mut aggregate = OFFSET;
    for pattern in 0..512u16 {
        let cells: Vec<_> = (0..9).map(|i| ((pattern >> i) & 1) as u8).collect();
        let terrain = TerrainMask::new(3, 3, &cells).unwrap();
        for rule in [false, true] {
            let mut stream = Vec::new();
            for x in 0..3i32 {
                for y in -1..=3i32 {
                    for action in [
                        MotionAction::Walking,
                        MotionAction::Jumping,
                        MotionAction::Falling,
                        MotionAction::Splatting,
                    ] {
                        let directions: &[_] = if action == MotionAction::Splatting {
                            &[MotionDirection::Stopped]
                        } else {
                            &[MotionDirection::Left, MotionDirection::Right]
                        };
                        let frames: &[_] = match action {
                            MotionAction::Walking => &[0, 7],
                            MotionAction::Jumping => &[0],
                            MotionAction::Falling => &[0, 3],
                            MotionAction::Splatting => &[0, 14, 15],
                        };
                        let fallen_values: &[_] = if action == MotionAction::Falling {
                            &[0, 60, 61]
                        } else {
                            &[0]
                        };
                        for &direction in directions {
                            for &frame in frames {
                                for &fallen in fallen_values {
                                    let initial = ActorState {
                                        id: 0,
                                        born_iteration: 0,
                                        x,
                                        y,
                                        direction,
                                        action,
                                        frame,
                                        end_of_animation: false,
                                        fallen,
                                        removed: None,
                                    };
                                    let mut a = Actor::from_state(
                                        initial,
                                        MotionRules {
                                            faller_starts_with_three: rule,
                                        },
                                    )
                                    .unwrap();
                                    let result = a.advance(terrain).unwrap();
                                    stream.extend_from_slice(&pattern.to_le_bytes());
                                    stream.push(u8::from(rule));
                                    stream.extend_from_slice(&x.to_le_bytes());
                                    stream.extend_from_slice(&y.to_le_bytes());
                                    stream.extend_from_slice(&[
                                        action_code(action),
                                        direction.delta() as i8 as u8,
                                        frame,
                                    ]);
                                    put32(&mut stream, fallen);
                                    motion_bytes(&mut stream, a.state());
                                    outcome_bytes(&mut stream, result);
                                    assert_eq!((a.state().id, a.state().born_iteration), (0, 0));
                                    total += 1;
                                }
                            }
                        }
                    }
                }
            }
            assert_eq!(
                fnv(&stream, OFFSET),
                golden::GROUP_DIGESTS[usize::from(pattern) * 2 + usize::from(rule)],
                "pattern={pattern}, starts_three={rule}"
            );
            aggregate = fnv(&stream, aggregate);
        }
    }
    assert_eq!(total, 322560);
    assert_eq!(aggregate, golden::CORPUS_DIGEST);
    std::println!("322560 transitions / 1024 groups / fnv1a64={aggregate:016x}");
}

fn scene(kind: u8) -> Vec<u8> {
    let floor = if kind == 1 {
        100
    } else if kind == 5 {
        4
    } else {
        60
    };
    board(96, 160, |x, y| {
        let height = if kind == 3 && (48..64).contains(&x) {
            56
        } else {
            floor
        };
        (kind != 2 && y >= height) || (kind == 4 && (40..48).contains(&x) && (40..60).contains(&y))
    })
}
fn startup_config(glitch: bool) -> StartupConfig {
    StartupConfig {
        time_limit_minutes: 5,
        total_lemmings: 4,
        initial_release_rate: 50,
        entrance_count: 2,
        pause_glitch: glitch,
    }
}
fn entrance_points(kind: u8) -> [EntrancePoint; 2] {
    let y = if kind == 5 { -14 } else { 0 };
    [EntrancePoint { x: 0, y }, EntrancePoint { x: 40, y }]
}
fn inputs(scenario: u8) -> Vec<TimedInput<StartupCommand>> {
    match scenario {
        1 => vec![TimedInput {
            control_step: 54,
            sequence: 0,
            command: InputCommand::Custom(StartupCommand::SetReleaseRate(99)),
        }],
        2 => vec![
            TimedInput {
                control_step: 16,
                sequence: 0,
                command: InputCommand::Pause,
            },
            TimedInput {
                control_step: 34,
                sequence: 0,
                command: InputCommand::Resume,
            },
        ],
        _ => vec![],
    }
}
fn startup_bytes(out: &mut Vec<u8>, event: StartupEvent) {
    match event {
        StartupEvent::Input {
            control_step,
            sequence,
            accepted,
        } => {
            out.push(0);
            put64(out, control_step);
            put32(out, sequence);
            out.push(u8::from(accepted));
        }
        StartupEvent::LetsGo => out.push(1),
        StartupEvent::EntranceSound => out.push(2),
        StartupEvent::EntrancesOpened => out.push(3),
        StartupEvent::Spawn(s) => {
            out.push(4);
            put32(out, s.actor_id);
            out.push(s.entrance_index);
            put64(out, s.iteration);
        }
        StartupEvent::Tick(i) => {
            out.push(5);
            put64(out, i);
        }
    }
}
fn snapshot_bytes(out: &mut Vec<u8>, startup: &StartupScheduler) {
    let step = startup.step_state();
    let s = startup.state();
    out.push(6);
    put64(out, step.control_step);
    put64(out, step.iteration);
    out.push(u8::from(step.paused));
    put32(out, s.seconds_remaining);
    out.extend_from_slice(&[
        s.clock_frame,
        s.glitch_pause_iterations,
        s.release_rate,
        s.spawn_countdown,
        u8::from(s.entrances_open),
    ]);
    put32(out, s.released);
}
fn replay(kind: u8, scenario: u8, glitch: bool, frames: &[u64]) -> (Vec<u8>, usize) {
    let cells = scene(kind);
    let terrain = TerrainMask::new(96, 160, &cells).unwrap();
    let entrances = entrance_points(kind);
    let input = inputs(scenario);
    let schedule = InputSchedule::new(&input).unwrap();
    let mut startup = StartupScheduler::new(startup_config(glitch)).unwrap();
    let mut clock = FixedTickClock::new();
    let mut actors = Vec::new();
    let mut stream = vec![8, kind, scenario, u8::from(glitch)];
    let mut updates = 0;
    for &ns in frames {
        for _ in 0..clock.advance(Duration::from_nanos(ns)).unwrap() {
            startup
                .step(&schedule, |event| {
                    startup_bytes(&mut stream, event);
                    match event {
                        StartupEvent::Spawn(request) => actors.push(
                            Actor::from_spawn(request, &entrances, MotionRules::default()).unwrap(),
                        ),
                        StartupEvent::Tick(_) => {
                            step_actors(&mut actors, terrain, |s, outcome| {
                                stream.push(9);
                                actor_bytes(&mut stream, s);
                                outcome_bytes(&mut stream, outcome);
                                updates += 1;
                            })
                            .unwrap();
                        }
                        _ => {}
                    }
                })
                .unwrap();
            snapshot_bytes(&mut stream, &startup);
            stream.push(10);
            put32(&mut stream, actors.len() as u32);
            for actor in &actors {
                actor_bytes(&mut stream, actor.state());
            }
        }
    }
    assert_eq!(startup.step_state().control_step, 340);
    assert_eq!(clock.phase_numerator(), 0);
    assert_eq!(cells, scene(kind));
    (stream, updates)
}

#[test]
fn spawned_actor_advances_on_its_birth_tick_before_the_next_control_snapshot() {
    let cells = scene(0);
    let terrain = TerrainMask::new(96, 160, &cells).unwrap();
    let entrances = entrance_points(0);
    let schedule = InputSchedule::<StartupCommand>::new(&[]).unwrap();
    let mut startup = StartupScheduler::new(startup_config(true)).unwrap();
    let mut actors = Vec::new();
    for _ in 0..54 {
        startup
            .step(&schedule, |event| match event {
                StartupEvent::Spawn(request) => actors
                    .push(Actor::from_spawn(request, &entrances, MotionRules::default()).unwrap()),
                StartupEvent::Tick(_) => {
                    step_actors(&mut actors, terrain, |_, _| {}).unwrap();
                }
                _ => {}
            })
            .unwrap();
    }
    assert_eq!(actors.len(), 1);
    assert_eq!(
        (
            actors[0].state().id,
            actors[0].state().born_iteration,
            actors[0].state().y,
            actors[0].state().fallen,
            actors[0].state().frame
        ),
        (0, 54, 17, 6, 1)
    );
}

#[test]
fn startup_and_actor_replays_match_python_and_three_render_patterns() {
    let sixty_hz: Vec<_> = (0..1200u64)
        .map(|i| (i + 1) * 1_000_000_000 / 60 - i * 1_000_000_000 / 60)
        .collect();
    let patterns = [
        vec![20_000_000_000],
        sixty_hz,
        [
            0,
            1,
            58_823_528,
            1,
            240_000_001,
            0,
            123_456_789,
            577_719_680,
        ]
        .repeat(20),
    ];
    for pattern in &patterns {
        assert_eq!(pattern.iter().sum::<u64>(), 20_000_000_000);
    }
    let mut cases = 0;
    let mut aggregate = OFFSET;
    let mut updates = 0;
    for kind in 0..6 {
        for scenario in 0..3 {
            for glitch in [false, true] {
                let wanted = replay(kind, scenario, glitch, &patterns[0]);
                for frames in &patterns[1..] {
                    assert_eq!(replay(kind, scenario, glitch, frames), wanted);
                }
                aggregate = fnv(&wanted.0, aggregate);
                updates += wanted.1;
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 36);
    assert_eq!(updates, 17040);
    assert_eq!(aggregate, golden::INTEGRATION_DIGEST);
    std::println!(
        "36 cases / 108 replays / 36720 control calls / 17040 actor updates per pattern / fnv1a64={aggregate:016x}"
    );
}
