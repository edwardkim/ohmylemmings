use lemmings_core::*;
const W: usize = 64;
const H: usize = 170;
const N: usize = W * H;
const ON: usize = 24 * 48;
fn actor(action: MotionAction, x: i32, y: i32) -> Actor {
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
    s.action = action;
    s.x = x;
    s.y = y;
    s.fallen = 0;
    s.frame = 0;
    s.bricks_left = if action == MotionAction::Building {
        12
    } else {
        0
    };
    if action == MotionAction::Splatting {
        s.direction = MotionDirection::Stopped;
    }
    Actor::from_state(s, MotionRules::default()).unwrap()
}
fn advance(
    a: &mut Actor,
    t: &mut [u8],
    m: &mut [u8],
    mask: &[u8; 44],
) -> Result<MotionOutcome, MotionError> {
    let empty = [0; N];
    a.advance_with_bomber(
        &mut TerrainMaskMut::new(W, H, t).unwrap(),
        BomberContext::new(
            MiningContext::new(
                &[255; 104],
                BashingContext::new(&[255; 160], TerrainMask::new(W, H, &empty).unwrap()),
            ),
            mask,
        ),
        &mut ObjectMapMut::new(24, 48, -16, -16, m).unwrap(),
    )
}
#[test]
fn bomber_assignment_runs_alongside_every_existing_job_and_rejects_duplicates() {
    for action in [
        MotionAction::Walking,
        MotionAction::Jumping,
        MotionAction::Falling,
        MotionAction::Digging,
        MotionAction::Bashing,
        MotionAction::Mining,
        MotionAction::Building,
        MotionAction::Shrugging,
    ] {
        let mut a = actor(action, 32, 100);
        let before = a.state();
        let mut count = 2;
        assert!(a.assign_bomber(&mut count));
        assert_eq!(count, 1);
        assert_eq!(a.state().explosion_timer, 79);
        let mut wanted = before;
        wanted.explosion_timer = 79;
        assert_eq!(a.state(), wanted);
        let saved = a;
        assert!(!a.assign_bomber(&mut count));
        assert_eq!((a, count), (saved, 1));
    }
    let mut a = actor(MotionAction::Walking, 32, 100);
    assert!(!a.assign_bomber(&mut 0));
    assert!(!actor(MotionAction::Splatting, 32, 100).assign_bomber(&mut 1));
}
#[test]
fn expiry_skips_motion_then_ohno_finishes_before_single_frame_explosion() {
    let mut a = actor(MotionAction::Walking, 0, 100);
    let mut t = [0; N];
    t[100 * W..].fill(1);
    let mut m = [128; ON];
    assert!(a.assign_bomber(&mut 1));
    for remaining in (1..=78).rev() {
        advance(&mut a, &mut t, &mut m, &[255; 44]).unwrap();
        assert_eq!(a.state().explosion_timer, remaining);
    }
    // Expire at a known supported foot, without advancing its Walking handler.
    let mut s = a.state();
    s.x = 32;
    s.y = 100;
    s.action = MotionAction::Walking;
    s.fallen = 0;
    a = Actor::from_state(s, MotionRules::default()).unwrap();
    let out = advance(&mut a, &mut t, &mut m, &[255; 44]).unwrap();
    assert!(!out.check_objects);
    assert_eq!(
        (a.state().action, a.state().frame, a.state().x),
        (MotionAction::OhNo, 0, 32)
    );
    for frame in 1..=15 {
        advance(&mut a, &mut t, &mut m, &[255; 44]).unwrap();
        assert_eq!(
            (a.state().action, a.state().frame),
            (MotionAction::OhNo, frame)
        );
    }
    advance(&mut a, &mut t, &mut m, &[255; 44]).unwrap();
    assert_eq!(
        (a.state().action, a.state().frame),
        (MotionAction::Exploding, 0)
    );
    let out = advance(&mut a, &mut t, &mut m, &[255; 44]).unwrap();
    assert_eq!(out.removal, Some(RemovalReason::Exploded));
    assert_eq!(
        (a.state().particle_timer, a.state().particle_frame),
        (52, 0)
    );
    for tick in 1..=52 {
        advance(&mut a, &mut t, &mut m, &[255; 44]).unwrap();
        assert_eq!(
            (a.state().particle_timer, a.state().particle_frame),
            (52 - tick, tick.min(50))
        );
    }
    let before = a;
    advance(&mut a, &mut t, &mut m, &[255; 44]).unwrap();
    assert_eq!(a, before);
}
#[test]
fn falling_expiry_enters_explosion_without_movement_or_ohno() {
    let mut s = actor(MotionAction::Falling, 32, 50).state();
    s.explosion_timer = 1;
    let mut a = Actor::from_state(s, MotionRules::default()).unwrap();
    let mut t = [0; N];
    let mut m = [128; ON];
    advance(&mut a, &mut t, &mut m, &[255; 44]).unwrap();
    assert_eq!(
        (a.state().action, a.state().y, a.state().frame),
        (MotionAction::Exploding, 50, 0)
    );
    advance(&mut a, &mut t, &mut m, &[255; 44]).unwrap();
    assert_eq!(a.state().removed, Some(RemovalReason::Exploded));
}
#[test]
fn explosion_restores_blocker_before_reading_steel_or_water_and_uses_foot_only() {
    for code in [128, 133, 137] {
        let mut a = actor(MotionAction::Walking, 32, 100);
        let mut t = [1; N];
        let mut m = [code; ON];
        assert!(a.assign_blocker(
            &mut 1,
            &mut ObjectMapMut::new(24, 48, -16, -16, &mut m).unwrap()
        ));
        let mut s = a.state();
        s.action = MotionAction::Exploding;
        a = Actor::from_state(s, MotionRules::default()).unwrap();
        advance(&mut a, &mut t, &mut m, &[255; 44]).unwrap();
        assert!(m.iter().all(|&v| v == code));
        assert_eq!(a.state().blocker_saved, None);
        assert_eq!(
            t.iter().filter(|&&v| v == 0).count(),
            if code == 128 { 16 * 22 } else { 0 }
        );
    }
    let mut a = actor(MotionAction::Exploding, 32, 100);
    let mut t = [1; N];
    let mut m = [137; ON];
    ObjectMapMut::new(24, 48, -16, -16, &mut m)
        .unwrap()
        .write(32, 100, 128);
    let mut mask = [0; 44];
    mask[0] = 128;
    mask[43] = 1;
    let out = advance(&mut a, &mut t, &mut m, &mask).unwrap();
    assert_eq!(out.explosion_erased[0], 1);
    assert_eq!(out.explosion_erased[21], 1 << 15);
    assert_eq!(t[86 * W + 24], 0);
    assert_eq!(t[107 * W + 39], 0);
    assert_eq!(t.iter().filter(|&&v| v == 0).count(), 2);
}
#[test]
fn countdown_glyph_boundaries_and_splat_cancel_match_original_order() {
    for timer in 0..=79 {
        let mut s = actor(MotionAction::Walking, 32, 100).state();
        s.explosion_timer = timer;
        assert_eq!(
            s.countdown_digit(),
            match timer {
                0 => None,
                1..=16 => Some(1),
                17..=32 => Some(2),
                33..=48 => Some(3),
                49..=64 => Some(4),
                _ => Some(5),
            }
        );
    }
    let mut s = actor(MotionAction::Falling, 32, 100).state();
    s.fallen = 63;
    s.explosion_timer = 2;
    let mut a = Actor::from_state(s, MotionRules::default()).unwrap();
    let mut t = [0; N];
    t[100 * W + 32] = 1;
    let mut m = [128; ON];
    advance(&mut a, &mut t, &mut m, &[255; 44]).unwrap();
    assert_eq!(
        (a.state().action, a.state().explosion_timer),
        (MotionAction::Splatting, 0)
    );
}
#[test]
fn gravity_errors_preserve_countdown_actor_terrain_and_object_map() {
    let mut s = actor(MotionAction::Walking, i32::MAX, 100).state();
    s.explosion_timer = 2;
    let mut a = Actor::from_state(s, MotionRules::default()).unwrap();
    let before = a;
    let mut t = [0; N];
    let mut m = [137; ON];
    assert_eq!(
        advance(&mut a, &mut t, &mut m, &[255; 44]),
        Err(MotionError::CoordinateOverflow)
    );
    assert_eq!(a, before);
    assert_eq!(t, [0; N]);
    assert_eq!(m, [137; ON]);
    s.explosion_timer = 1;
    a = Actor::from_state(s, MotionRules::default()).unwrap();
    advance(&mut a, &mut t, &mut m, &[255; 44]).unwrap();
    assert_eq!(a.state().action, MotionAction::OhNo);
    let mut s = actor(MotionAction::OhNo, 32, i32::MAX).state();
    s.frame = 0;
    a = Actor::from_state(s, MotionRules::default()).unwrap();
    let before = a;
    assert_eq!(
        advance(&mut a, &mut t, &mut m, &[255; 44]),
        Err(MotionError::CoordinateOverflow)
    );
    assert_eq!(a, before);
}

#[test]
fn explosion_and_blocker_support_release_follow_actor_id_order() {
    for explosion_first in [true, false] {
        let mut cells = [128; ON];
        let mut map = ObjectMapMut::new(24, 48, -16, -16, &mut cells).unwrap();
        let mut blocker = actor(MotionAction::Walking, 36, 100);
        assert!(blocker.assign_blocker(&mut 1, &mut map));
        let change_id = |a: Actor, id| {
            Actor::from_state(ActorState { id, ..a.state() }, MotionRules::default()).unwrap()
        };
        let exploding = change_id(
            actor(MotionAction::Exploding, 32, 100),
            if explosion_first { 0 } else { 1 },
        );
        let blocker = change_id(blocker, if explosion_first { 1 } else { 0 });
        let walker = change_id(actor(MotionAction::Walking, 31, 96), 2);
        let mut actors = if explosion_first {
            [exploding, blocker, walker]
        } else {
            [blocker, exploding, walker]
        };
        let mut t = [0; N];
        t[100 * W..].fill(1);
        let empty = [0; N];
        let context = BomberContext::new(
            MiningContext::new(
                &[255; 104],
                BashingContext::new(&[255; 160], TerrainMask::new(W, H, &empty).unwrap()),
            ),
            &[255; 44],
        );
        let tick = step_actors_with_bomber(
            &mut actors,
            &mut TerrainMaskMut::new(W, H, &mut t).unwrap(),
            context,
            &mut map,
            |_, _| {},
        )
        .unwrap();
        assert_eq!(tick.removed, 1);
        assert_eq!(
            actors[if explosion_first { 1 } else { 0 }].state().action,
            if explosion_first {
                MotionAction::Walking
            } else {
                MotionAction::Blocking
            }
        );
        assert_eq!(
            actors[2].state().direction,
            if explosion_first {
                MotionDirection::Right
            } else {
                MotionDirection::Left
            }
        );
    }
}
#[test]
fn invalid_restore_missing_context_and_duplicate_ids_cannot_mutate_records() {
    let base = actor(MotionAction::Walking, 32, 100).state();
    for state in [
        ActorState {
            explosion_timer: 80,
            ..base
        },
        ActorState {
            particle_timer: 1,
            ..base
        },
        ActorState {
            particle_frame: 51,
            ..base
        },
        ActorState {
            action: MotionAction::Splatting,
            direction: MotionDirection::Stopped,
            explosion_timer: 1,
            ..base
        },
    ] {
        assert_eq!(
            Actor::from_state(state, MotionRules::default()),
            Err(MotionError::InvalidBomberState)
        );
    }
    let mut a = actor(MotionAction::Walking, 32, 100);
    a.assign_bomber(&mut 1);
    let before = a;
    assert_eq!(
        a.advance(TerrainMask::new(W, H, &[0; N]).unwrap()),
        Err(MotionError::BomberContextRequired)
    );
    assert_eq!(a, before);
    let mut actors = [a, a];
    let mut t = [1; N];
    let mut cells = [137; ON];
    let empty = [0; N];
    let context = BomberContext::new(
        MiningContext::new(
            &[255; 104],
            BashingContext::new(&[255; 160], TerrainMask::new(W, H, &empty).unwrap()),
        ),
        &[255; 44],
    );
    assert_eq!(
        step_actors_with_bomber(
            &mut actors,
            &mut TerrainMaskMut::new(W, H, &mut t).unwrap(),
            context,
            &mut ObjectMapMut::new(24, 48, -16, -16, &mut cells).unwrap(),
            |_, _| panic!("emit")
        ),
        Err(MotionError::ActorOrder { index: 1 })
    );
    assert_eq!(actors, [a, a]);
    assert_eq!(t, [1; N]);
    assert_eq!(cells, [137; ON]);
}
#[test]
fn ohno_falls_at_most_three_pixels_checks_objects_and_removes_below_world() {
    let mut a = actor(MotionAction::OhNo, 32, 100);
    let mut t = [0; N];
    t[102 * W + 32] = 1;
    let mut m = [128; ON];
    ObjectMapMut::new(24, 48, -16, -16, &mut m)
        .unwrap()
        .write(32, 102, 130);
    let out = advance(&mut a, &mut t, &mut m, &[255; 44]).unwrap();
    assert!(out.check_objects);
    assert_eq!(
        (a.state().y, a.state().direction),
        (102, MotionDirection::Left)
    );
    let mut a = actor(MotionAction::OhNo, 32, 162);
    advance(&mut a, &mut t, &mut m, &[255; 44]).unwrap();
    assert_eq!(
        (a.state().y, a.state().removed),
        (165, Some(RemovalReason::BottomBoundary))
    );
    for (x, y) in [
        (0, 0),
        (63, 169),
        (i32::MIN, i32::MIN),
        (i32::MAX, i32::MAX),
    ] {
        let mut a = actor(MotionAction::Exploding, x, y);
        let mut t = [1; N];
        let mut m = [128; ON];
        advance(&mut a, &mut t, &mut m, &[255; 44]).unwrap();
        assert_eq!(a.state().removed, Some(RemovalReason::Exploded));
        let expected = (0..H)
            .flat_map(|yy| (0..W).map(move |xx| (xx as i64, yy as i64)))
            .filter(|&(xx, yy)| {
                xx >= i64::from(x) - 8
                    && xx < i64::from(x) + 8
                    && yy >= i64::from(y) - 14
                    && yy < i64::from(y) + 8
            })
            .count();
        assert_eq!(t.iter().filter(|&&v| v == 0).count(), expected);
    }
}
