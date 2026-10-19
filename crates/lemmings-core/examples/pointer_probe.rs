//! Own synthetic inputs for independent execution of pinned Pascal selection.
use lemmings_core::*;
const ACTIONS: [MotionAction; 18] = [
    MotionAction::Walking,
    MotionAction::Jumping,
    MotionAction::Falling,
    MotionAction::Splatting,
    MotionAction::Digging,
    MotionAction::Bashing,
    MotionAction::Mining,
    MotionAction::Building,
    MotionAction::Shrugging,
    MotionAction::Blocking,
    MotionAction::OhNo,
    MotionAction::Exploding,
    MotionAction::Climbing,
    MotionAction::Hoisting,
    MotionAction::Floating,
    MotionAction::Exiting,
    MotionAction::Drowning,
    MotionAction::Vaporizing,
];
fn actor(action: MotionAction, id: u32, x: i32) -> Actor {
    let mut s = Actor::from_spawn(
        SpawnRequest {
            actor_id: id,
            entrance_index: 0,
            iteration: 54,
        },
        &[EntrancePoint { x: 76, y: 86 }],
        MotionRules::default(),
    )
    .unwrap()
    .state();
    s.action = action;
    s.x = x;
    s.fallen = 0;
    s.direction = if action == MotionAction::Splatting {
        MotionDirection::Stopped
    } else {
        MotionDirection::Right
    };
    s.is_climber = matches!(action, MotionAction::Climbing | MotionAction::Hoisting);
    s.is_floater = action == MotionAction::Floating;
    s.bricks_left = if action == MotionAction::Building {
        12
    } else {
        0
    };
    s.blocker_saved = if action == MotionAction::Blocking {
        Some([128; 9])
    } else {
        None
    };
    Actor::from_state(s, MotionRules::default()).unwrap()
}
fn id(id: Option<u32>) -> i64 {
    id.map_or(-1, i64::from)
}
fn hit(p: &mut PointerSelection, actors: &[Actor], x: i64, y: i64, right: bool, k: &mut u32) {
    p.update(actors.iter().map(|a| a.state()), Some((x, y)), right);
    println!(
        "H {} {} {} {}",
        k,
        p.hits.count,
        id(p.hits.primary),
        id(p.hits.secondary)
    );
    *k += 1;
}
fn main() {
    let mut p = PointerSelection::default();
    let mut k = 0;
    for action in ACTIONS {
        let a = actor(action, 0, 100);
        for x in 80..=112 {
            for y in 70..=106 {
                for right in [false, true] {
                    hit(&mut p, &[a], x, y, right, &mut k);
                }
            }
        }
    }
    for a in ACTIONS {
        for b in ACTIONS {
            for right in [false, true] {
                hit(
                    &mut p,
                    &[actor(a, 0, 100), actor(b, 1, 105)],
                    100,
                    90,
                    right,
                    &mut k,
                );
            }
        }
    }
    for v in 0..2 {
        let mut l = actor(MotionAction::Walking, 0, 32);
        let w = actor(MotionAction::Building, 1, 200);
        p = PointerSelection::default();
        p.update([l.state(), w.state()], Some((32, 95)), false);
        if v == 1 {
            let mut s = l.state();
            s.removed = Some(RemovalReason::Trapped);
            s.object_below = 0;
            l = Actor::from_state(s, MotionRules::default()).unwrap();
        }
        if v == 2 {
            p = PointerSelection::default();
        }
        p.update([l.state(), w.state()], Some((200, 95)), true);
        let mut cells = vec![128; 415 * 47];
        let mut map = ObjectMapMut::new(
            DOS_OBJECT_MAP_WIDTH,
            DOS_OBJECT_MAP_HEIGHT,
            DOS_OBJECT_MAP_ORIGIN,
            DOS_OBJECT_MAP_ORIGIN,
            &mut cells,
        )
        .unwrap();
        let mut n = 2;
        let accepted = p
            .assignment(AssignmentSkill::Blocker)
            .is_some_and(|request| {
                assert_eq!(request.primary, 0);
                if l.state().removed.is_some() {
                    l.assign_removed(|a| a.assign_blocker(&mut n, &mut map))
                } else {
                    l.assign_blocker(&mut n, &mut map)
                }
            });
        let h = cells.iter().fold(0xcbf29ce484222325u64, |h, &v| {
            (h ^ u64::from(v)).wrapping_mul(0x100000001b3)
        });
        println!(
            "R {v} {} {n} {} {h:016x}",
            if accepted { 0 } else { -1 },
            l.state().removed.is_some()
        );
    }
    let mut k = 0;
    let empty = [0; 1];
    let masks = [0; 160];
    let cells = vec![128; 415 * 47];
    let context = BashingContext {
        masks: &masks,
        object_map: Some(ObjectMap::new(415, 47, -16, -16, &cells).unwrap()),
        steel: TerrainMask::new(1, 1, &empty).unwrap(),
        one_way_left: None,
        one_way_right: None,
    };
    let foot = |a: MotionAction| match a {
        MotionAction::Building | MotionAction::Mining | MotionAction::Exiting => 13,
        MotionAction::Digging | MotionAction::Climbing | MotionAction::Hoisting => 12,
        MotionAction::Floating => 16,
        MotionAction::Vaporizing => 14,
        MotionAction::Exploding => 25,
        _ => 10,
    };
    for a in ACTIONS {
        for b in ACTIONS {
            for sk in [AssignmentSkill::Builder, AssignmentSkill::Digger] {
                for v in 0..6 {
                    let mut first = actor(a, 0, 32).state();
                    let mut second = actor(b, 1, 60).state();
                    first.object_below = 128;
                    second.object_below = 128;
                    match v {
                        1 => first.object_below = 137,
                        2 => second.object_below = 137,
                        3 => first.y = foot(a) - 6,
                        4 => second.y = foot(b) - 6,
                        _ => {}
                    }
                    let mut actors = [
                        Actor::from_state(first, MotionRules::default()).unwrap(),
                        Actor::from_state(second, MotionRules::default()).unwrap(),
                    ];
                    let request = PointerAssignment {
                        skill: sk,
                        primary: 0,
                        secondary: Some(1),
                        right_click: false,
                    };
                    let mut n = if v == 5 { 0 } else { 2 };
                    let target = request.target(first).filter(|&id| match (sk, id) {
                        (AssignmentSkill::Builder, 0) => actors[0].assign_builder(&mut n),
                        (AssignmentSkill::Builder, 1) => {
                            actors[1].assign_builder_secondary(&mut n, first)
                        }
                        (AssignmentSkill::Digger, 0) => {
                            actors[0].assign_digger_with_objects(&mut n, context)
                        }
                        (AssignmentSkill::Digger, 1) => {
                            actors[1].assign_digger_secondary(&mut n, first)
                        }
                        _ => false,
                    });
                    println!("F {k} {} {n}", id(target));
                    k += 1;
                }
            }
        }
    }
}
