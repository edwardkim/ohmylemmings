//! Synthetic assignment probe for the independently executed Pascal matrix.
use lemmings_core::*;

const ACTIONS: [MotionAction; 15] = [
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
];
const SKILLS: [&str; 8] = [
    "Climber", "Floater", "Bomber", "Blocker", "Builder", "Basher", "Miner", "Digger",
];
const VARIANTS: [&str; 11] = [
    "normal",
    "empty",
    "athlete",
    "armed",
    "steel_below",
    "steel_front",
    "oneway_left",
    "oneway_right",
    "overlap",
    "head_above",
    "head_edge",
];

fn foot_y(action: MotionAction) -> i32 {
    match action {
        MotionAction::Mining | MotionAction::Building => 13,
        MotionAction::Digging | MotionAction::Climbing | MotionAction::Hoisting => 12,
        MotionAction::Exploding => 25,
        MotionAction::Floating => 16,
        _ => 10,
    }
}

fn actor(action: MotionAction, direction: MotionDirection, variant: &str) -> Actor {
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
    s.direction = if action == MotionAction::Splatting {
        MotionDirection::Stopped
    } else {
        direction
    };
    s.frame = if action.frame_count() == 1 { 0 } else { 3 };
    s.fallen = if action == MotionAction::Falling {
        12
    } else {
        0
    };
    s.bricks_left = if action == MotionAction::Building {
        12
    } else {
        0
    };
    s.blocker_saved = (action == MotionAction::Blocking).then_some([128; 9]);
    s.is_climber =
        variant == "athlete" || matches!(action, MotionAction::Climbing | MotionAction::Hoisting);
    s.is_floater = variant == "athlete" || action == MotionAction::Floating;
    s.float_index = if s.is_floater { 7 } else { 0 };
    s.explosion_timer = if variant == "armed" && action != MotionAction::Splatting {
        42
    } else {
        0
    };
    s.object_below = if variant == "steel_below" { 137 } else { 128 };
    s.object_in_front = match variant {
        "steel_front" => 137,
        "oneway_left" => 135,
        "oneway_right" => 136,
        _ => 128,
    };
    s.y = match variant {
        "head_above" => foot_y(action) - 6,
        "head_edge" => foot_y(action) - 5,
        _ => 100,
    };
    Actor::from_state(s, MotionRules::default()).unwrap()
}

fn assign(a: &mut Actor, skill: usize, count: &mut u16, map: &mut ObjectMapMut<'_>) -> bool {
    let blank = [0];
    let mut bash = BashingContext::new(&[0; 160], TerrainMask::new(1, 1, &blank).unwrap());
    bash.object_map = Some(map.as_read_only());
    match skill {
        0 => a.assign_climber(count),
        1 => a.assign_floater(count),
        2 => a.assign_bomber(count),
        3 => a.assign_blocker(count, map),
        4 => a.assign_builder(count),
        5 => a.assign_basher(count, bash),
        6 => a.assign_miner(count, MiningContext::new(&[0; 104], bash)),
        7 => a.assign_digger_with_objects(count, bash),
        _ => unreachable!(),
    }
}

fn main() {
    for action in ACTIONS {
        for (skill, name) in SKILLS.iter().enumerate() {
            for direction in [MotionDirection::Left, MotionDirection::Right] {
                for variant in VARIANTS {
                    let mut a = actor(action, direction, variant);
                    let mut cells = vec![128; 415 * 47];
                    let mut map = ObjectMapMut::new(415, 47, -16, -16, &mut cells).unwrap();
                    if action == MotionAction::Blocking {
                        for dy in [-6, -2, 2] {
                            for (dx, code) in [
                                (-4, OBJECT_FORCE_LEFT),
                                (0, OBJECT_BLOCKER),
                                (4, OBJECT_FORCE_RIGHT),
                            ] {
                                map.write(32 + dx, 100 + dy, code);
                            }
                        }
                    }
                    if variant == "overlap" {
                        map.write(32, i64::from(a.state().y), 131);
                    }
                    let mut count = u16::from(variant != "empty");
                    let accepted = assign(&mut a, skill, &mut count, &mut map);
                    let s = a.state();
                    let hash = map
                        .as_read_only()
                        .as_bytes()
                        .iter()
                        .fold(0xcbf29ce484222325u64, |h, b| {
                            (h ^ u64::from(*b)).wrapping_mul(0x100000001b3)
                        });
                    println!(
                        "{{\"initial\":\"{action:?}\",\"skill\":\"{name}\",\"direction\":{},\"variant\":\"{variant}\",\"accepted\":{accepted},\"remaining\":{count},\"action\":\"{:?}\",\"y\":{},\"frame\":{},\"end\":{},\"fallen\":{},\"new_digger\":{},\"bricks\":{},\"climber\":{},\"floater\":{},\"float_index\":{},\"walking_once\":{},\"timer\":{},\"map_hash\":\"{hash:016x}\"}}",
                        direction.delta(),
                        s.action,
                        s.y,
                        s.frame,
                        s.end_of_animation,
                        s.fallen,
                        s.new_digger,
                        s.bricks_left,
                        s.is_climber,
                        s.is_floater,
                        s.float_index,
                        s.walking_once,
                        s.explosion_timer
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The allow lists are from the eight pinned Assign* bodies, not movement dispatch.
    const ALLOWED: [&[MotionAction]; 8] = [
        &[
            MotionAction::Walking,
            MotionAction::Jumping,
            MotionAction::Falling,
            MotionAction::Digging,
            MotionAction::Bashing,
            MotionAction::Mining,
            MotionAction::Building,
            MotionAction::Shrugging,
            MotionAction::OhNo,
            MotionAction::Floating,
        ],
        &[
            MotionAction::Walking,
            MotionAction::Jumping,
            MotionAction::Falling,
            MotionAction::Digging,
            MotionAction::Bashing,
            MotionAction::Mining,
            MotionAction::Building,
            MotionAction::Shrugging,
            MotionAction::OhNo,
            MotionAction::Climbing,
            MotionAction::Hoisting,
        ],
        &[
            MotionAction::Walking,
            MotionAction::Jumping,
            MotionAction::Falling,
            MotionAction::Digging,
            MotionAction::Bashing,
            MotionAction::Mining,
            MotionAction::Building,
            MotionAction::Shrugging,
            MotionAction::Blocking,
            MotionAction::Climbing,
            MotionAction::Hoisting,
            MotionAction::Floating,
        ],
        &[
            MotionAction::Walking,
            MotionAction::Digging,
            MotionAction::Bashing,
            MotionAction::Mining,
            MotionAction::Building,
            MotionAction::Shrugging,
        ],
        &[
            MotionAction::Walking,
            MotionAction::Digging,
            MotionAction::Bashing,
            MotionAction::Mining,
            MotionAction::Shrugging,
        ],
        &[
            MotionAction::Walking,
            MotionAction::Digging,
            MotionAction::Mining,
            MotionAction::Building,
            MotionAction::Shrugging,
        ],
        &[
            MotionAction::Walking,
            MotionAction::Digging,
            MotionAction::Bashing,
            MotionAction::Building,
            MotionAction::Shrugging,
        ],
        &[
            MotionAction::Walking,
            MotionAction::Bashing,
            MotionAction::Mining,
            MotionAction::Building,
            MotionAction::Shrugging,
        ],
    ];

    #[test]
    fn every_action_skill_pair_matches_the_reference_allow_lists() {
        for action in ACTIONS {
            for (skill, allowed) in ALLOWED.iter().enumerate() {
                let mut a = actor(action, MotionDirection::Right, "normal");
                let before = a;
                let mut cells = vec![128; 415 * 47];
                let mut map = ObjectMapMut::new(415, 47, -16, -16, &mut cells).unwrap();
                let mut count = 1;
                let accepted = assign(&mut a, skill, &mut count, &mut map);
                assert_eq!(
                    accepted,
                    allowed.contains(&action),
                    "{action:?} -> {}",
                    SKILLS[skill]
                );
                assert_eq!(count, u16::from(!accepted));
                Actor::from_state(a.state(), MotionRules::default()).unwrap();
                if !accepted {
                    assert_eq!(a, before);
                    assert!(cells.iter().all(|b| *b == 128));
                }
            }
        }
    }

    #[test]
    fn empty_inventory_preserves_all_actor_and_object_fields() {
        for action in ACTIONS {
            for skill in 0..8 {
                let mut a = actor(action, MotionDirection::Left, "athlete");
                let before = a;
                let mut cells = vec![128; 415 * 47];
                assert!(!assign(
                    &mut a,
                    skill,
                    &mut 0,
                    &mut ObjectMapMut::new(415, 47, -16, -16, &mut cells).unwrap()
                ));
                assert_eq!(a, before);
                assert!(cells.iter().all(|b| *b == 128));
            }
        }
    }

    #[test]
    fn removed_actors_reject_every_skill_without_map_or_inventory_changes() {
        for action in [
            MotionAction::Walking,
            MotionAction::Falling,
            MotionAction::Digging,
            MotionAction::Mining,
            MotionAction::OhNo,
            MotionAction::Floating,
        ] {
            for skill in 0..8 {
                let mut s = actor(action, MotionDirection::Right, "normal").state();
                s.y = 164;
                s.removed = Some(RemovalReason::BottomBoundary);
                let mut a = Actor::from_state(s, MotionRules::default()).unwrap();
                let before = a;
                let mut count = 1;
                let mut cells = vec![128; 415 * 47];
                assert!(!assign(
                    &mut a,
                    skill,
                    &mut count,
                    &mut ObjectMapMut::new(415, 47, -16, -16, &mut cells).unwrap()
                ));
                assert_eq!((a, count), (before, 1));
                assert!(cells.iter().all(|b| *b == 128));
            }
        }
    }
}
