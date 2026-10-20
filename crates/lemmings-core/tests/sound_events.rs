use lemmings_core::*;

fn actor(action: MotionAction) -> Actor {
    let mut state = Actor::from_spawn(
        SpawnRequest {
            actor_id: 7,
            entrance_index: 0,
            iteration: 54,
        },
        &[EntrancePoint { x: 16, y: 86 }],
        MotionRules::default(),
    )
    .unwrap()
    .state();
    state.x = 40;
    state.y = 100;
    state.action = action;
    state.fallen = if action == MotionAction::Falling {
        61
    } else {
        0
    };
    state.frame = 0;
    state.bricks_left = if action == MotionAction::Building {
        3
    } else {
        0
    };
    Actor::from_state(state, MotionRules::default()).unwrap()
}
fn cues(out: MotionOutcome) -> Vec<SoundCue> {
    out.sound_cues.into_iter().flatten().collect()
}
fn advance(
    a: &mut Actor,
    terrain: &mut [u8],
    map: &mut [u8],
    traps: &mut TrapStates,
) -> MotionOutcome {
    let steel = vec![0; 256 * 192];
    let masks = [0; 160];
    let mines = [0; 104];
    let blast = [0; 44];
    let context = BomberContext::new(
        MiningContext::new(
            &mines,
            BashingContext::new(&masks, TerrainMask::new(256, 192, &steel).unwrap()),
        ),
        &blast,
    );
    let mut out = None;
    step_actors_with_hazards(
        core::slice::from_mut(a),
        &mut TerrainMaskMut::new(256, 192, terrain).unwrap(),
        context,
        &mut ObjectMapMut::new(64, 48, 0, 0, map).unwrap(),
        traps,
        |_, result| out = Some(result),
    )
    .unwrap();
    out.unwrap_or_default()
}
#[test]
fn fatal_landing_then_exit_preserves_both_transition_cues_in_order() {
    let mut a = actor(MotionAction::Falling);
    let mut terrain = vec![0; 256 * 192];
    terrain[100 * 256 + 40] = 1;
    let mut map = vec![OBJECT_NONE; 64 * 48];
    ObjectMapMut::new(64, 48, 0, 0, &mut map)
        .unwrap()
        .write(40, 100, OBJECT_EXIT);
    let out = advance(&mut a, &mut terrain, &mut map, &mut [None; 16]);
    assert_eq!(cues(out), [SoundCue::Splat, SoundCue::Yippee]);
    assert_eq!(a.state().action, MotionAction::Exiting);
    assert!(cues(advance(&mut a, &mut terrain, &mut map, &mut [None; 16])).is_empty());
}
#[test]
fn builder_warns_at_frame_ten_on_each_of_the_last_three_bricks() {
    let mut terrain = vec![0; 256 * 192];
    for bricks in [1, 2, 3, 4, 9] {
        let mut s = actor(MotionAction::Building).state();
        s.frame = 9;
        s.bricks_left = bricks;
        let mut a = Actor::from_state(s, MotionRules::default()).unwrap();
        let out = a
            .advance_with_jobs(
                &mut TerrainMaskMut::new(256, 192, &mut terrain).unwrap(),
                MiningContext::new(
                    &[0; 104],
                    BashingContext::new(
                        &[0; 160],
                        TerrainMask::new(256, 192, &vec![0; 256 * 192]).unwrap(),
                    ),
                ),
            )
            .unwrap();
        assert_eq!(
            cues(out),
            if bricks <= 3 {
                vec![SoundCue::BuilderWarning]
            } else {
                vec![]
            }
        );
    }
}
#[test]
fn countdown_and_ohno_animation_emit_before_explosion_terrain_work_once() {
    let mut terrain = vec![0; 256 * 192];
    let mut map = vec![OBJECT_NONE; 64 * 48];
    let mut s = actor(MotionAction::Walking).state();
    s.explosion_timer = 1;
    let mut a = Actor::from_state(s, MotionRules::default()).unwrap();
    assert_eq!(
        cues(advance(&mut a, &mut terrain, &mut map, &mut [None; 16])),
        [SoundCue::OhNo]
    );
    s = a.state();
    s.frame = 15;
    a = Actor::from_state(s, MotionRules::default()).unwrap();
    assert_eq!(
        cues(advance(&mut a, &mut terrain, &mut map, &mut [None; 16])),
        [SoundCue::Explosion]
    );
    assert!(cues(advance(&mut a, &mut terrain, &mut map, &mut [None; 16])).is_empty());
    assert_eq!(a.state().removed, Some(RemovalReason::Exploded));
    let mut s = actor(MotionAction::Falling).state();
    s.explosion_timer = 1;
    let mut a = Actor::from_state(s, MotionRules::default()).unwrap();
    assert_eq!(
        cues(advance(&mut a, &mut terrain, &mut map, &mut [None; 16])),
        [SoundCue::Explosion]
    );
}
#[test]
fn water_fire_and_trap_emit_at_entry_and_trap_arbitration_not_final_removal() {
    for code in [OBJECT_WATER, OBJECT_FIRE, 0] {
        let mut terrain = vec![0; 256 * 192];
        terrain[100 * 256..101 * 256].fill(1);
        let mut map = vec![OBJECT_NONE; 64 * 48];
        ObjectMapMut::new(64, 48, 0, 0, &mut map)
            .unwrap()
            .write(40, 100, code);
        let mut traps = [None; 16];
        traps[0] = Some(TrapState::new(5, 0).unwrap());
        let mut a = actor(MotionAction::Walking);
        let want = match code {
            OBJECT_WATER => SoundCue::Drowning,
            OBJECT_FIRE => SoundCue::Vaporizing,
            _ => SoundCue::Trap { slot: 0 },
        };
        assert_eq!(
            cues(advance(&mut a, &mut terrain, &mut map, &mut traps)),
            [want]
        );
        if code == 0 {
            let mut b = actor(MotionAction::Walking);
            assert!(cues(advance(&mut b, &mut terrain, &mut map, &mut traps)).is_empty());
        } else {
            assert!(cues(advance(&mut a, &mut terrain, &mut map, &mut traps)).is_empty());
        }
    }
}

#[test]
fn steel_rejected_assignment_has_its_own_cue_but_empty_inventory_and_one_way_rejection_are_silent()
{
    for miner in [false, true] {
        for (below, front, stock) in [
            (128, 137, 2),
            (137, 137, 2),
            (128, 135, 2),
            (128, 137, 0),
            (128, 128, 2),
        ] {
            let mut s = actor(MotionAction::Walking).state();
            s.object_below = below;
            s.object_in_front = front;
            let mut a = Actor::from_state(s, MotionRules::default()).unwrap();
            let steel = vec![0; 256 * 192];
            let map = vec![OBJECT_NONE; 64 * 48];
            let mut context =
                BashingContext::new(&[0; 160], TerrainMask::new(256, 192, &steel).unwrap());
            context.object_map = Some(ObjectMap::new(64, 48, 0, 0, &map).unwrap());
            let mut remaining = stock;
            let mut got = Vec::new();
            let accepted = if miner {
                a.assign_miner_with_cues(
                    &mut remaining,
                    MiningContext::new(&[0; 104], context),
                    |cue| got.push(cue),
                )
            } else {
                a.assign_basher_with_cues(&mut remaining, context, |cue| got.push(cue))
            };
            let skill = if miner {
                AssignmentSkill::Miner
            } else {
                AssignmentSkill::Basher
            };
            let expected = if stock == 0 || (miner && below == 137) || front == 135 {
                vec![]
            } else if front == 137 {
                vec![SoundCue::AssignmentBlocked(skill)]
            } else {
                vec![SoundCue::AssignSkill(skill)]
            };
            assert_eq!(got, expected);
            assert_eq!(remaining, stock - u16::from(accepted));
            if !accepted {
                assert_eq!(a.state(), s);
            }
        }
    }
}

#[test]
fn one_way_turnarounds_are_silent_while_real_steel_emits_one_collision_cue() {
    for action in [MotionAction::Bashing, MotionAction::Mining] {
        for direction in [MotionDirection::Left, MotionDirection::Right] {
            for code in [135, 136, 137] {
                let mut s = actor(action).state();
                s.direction = direction;
                s.frame = if action == MotionAction::Bashing {
                    10
                } else {
                    2
                };
                let mut a = Actor::from_state(s, MotionRules::default()).unwrap();
                let mut terrain = vec![0; 256 * 192];
                terrain[100 * 256..].fill(1);
                let mut map = vec![OBJECT_NONE; 64 * 48];
                let (x, y) = if action == MotionAction::Bashing {
                    (40 + 9 * direction.delta(), 92)
                } else {
                    (40 + 2 * direction.delta(), 101)
                };
                ObjectMapMut::new(64, 48, 0, 0, &mut map)
                    .unwrap()
                    .write(i64::from(x), y, code);
                let out = advance(&mut a, &mut terrain, &mut map, &mut [None; 16]);
                assert_eq!(
                    cues(out),
                    if code == 137 {
                        vec![SoundCue::HitsSteel]
                    } else {
                        vec![]
                    }
                );
            }
        }
    }
}
