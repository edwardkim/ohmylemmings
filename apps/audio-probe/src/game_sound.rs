//! Owned synthetic scenarios for the optional native/actual-WASM sound verifier.
use lemmings_core::*;
use lemmings_game::{Assets, LevelRun, ResultAction, pacing::Pacing, sound::SoundBatch};
use lemmings_resources::{AdlibCommand, DosMusic};

fn bare() -> LevelRun {
    let mut assets = Assets::synthetic();
    assets.config.total_lemmings = 0;
    LevelRun::new(assets, 1, None, Pacing::Dos).expect("owned fixture")
}
fn state(action: MotionAction) -> ActorState {
    let mut s = Actor::from_spawn(
        SpawnRequest {
            actor_id: 0,
            entrance_index: 0,
            iteration: 54,
        },
        &[EntrancePoint { x: 16, y: 86 }],
        MotionRules::default(),
    )
    .unwrap()
    .state();
    s.x = 40;
    s.y = 100;
    s.action = action;
    s.fallen = if action == MotionAction::Falling {
        61
    } else {
        0
    };
    s.bricks_left = if action == MotionAction::Building {
        3
    } else {
        0
    };
    s.is_floater = action == MotionAction::Floating;
    s
}
fn set_actor(run: &mut LevelRun, s: ActorState) {
    run.session.actors = vec![Actor::from_state(s, MotionRules::default()).expect("owned actor")];
}
fn map(run: &mut LevelRun, x: i64, y: i64, code: u8) {
    let mut cells = run.session.object_map_data().to_vec();
    ObjectMapMut::new(
        DOS_OBJECT_MAP_WIDTH,
        DOS_OBJECT_MAP_HEIGHT,
        DOS_OBJECT_MAP_ORIGIN,
        DOS_OBJECT_MAP_ORIGIN,
        &mut cells,
    )
    .unwrap()
    .write(x, y, code);
    run.session.configure_blocker(10, cells).unwrap();
}
fn fnv(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x100000001b3)
    })
}
fn record(
    out: &mut Vec<String>,
    name: &str,
    mut run: LevelRun,
    steps: u32,
    parts: &[u32],
) -> Result<(), String> {
    let start = run.session.scheduler.step_state().control_step;
    let mut events = Vec::new();
    let mut reset = false;
    let mut dropped = 0;
    let mut remaining = parts[0];
    let mut part = 0;
    for _ in 0..steps {
        run.tick().map_err(|e| e.to_string())?;
        remaining -= 1;
        if remaining == 0 {
            let batch = run.sound.drain();
            reset |= batch.reset;
            dropped += batch.dropped;
            events.extend(batch.events);
            part += 1;
            remaining = parts[part % parts.len()];
        }
    }
    let mut batch = run.sound.drain();
    reset |= batch.reset;
    dropped += batch.dropped;
    events.extend(batch.events);
    batch.events = events;
    batch.reset = reset;
    batch.dropped = dropped;
    let last = run.session.scheduler.step_state().control_step - 1;
    let calls = batch
        .events
        .iter()
        .filter(|e| e.source.control_step == last)
        .filter_map(|e| match e.command {
            Some(AdlibCommand::Music(i)) => Some(format!("[97,{}]", 0x301 + u16::from(i))),
            Some(AdlibCommand::Effect(i)) => Some(format!("[97,{}]", 0x401 + u16::from(i))),
            Some(AdlibCommand::Stop) => Some("[97,512]".into()),
            None => None,
        })
        .collect::<Vec<_>>()
        .join(",");
    let before = run.sound.diagnostic_json();
    let frame = run
        .view
        .frame(&run.assets, &run.session)
        .map_err(|e| e.to_string())?;
    if before != run.sound.diagnostic_json() {
        return Err("render changed sound timeline".into());
    }
    out.push(format!("{{\"name\":{name:?},\"start_control_step\":{start},\"steps\":{steps},\"calls\":[{calls}],\"sounds\":{},\"state\":{},\"terrain_fnv\":\"{:016x}\",\"objects_fnv\":\"{:016x}\",\"rgb_fnv\":\"{:016x}\"}}",
        batch.to_json(),run.session.snapshot(),fnv(run.session.terrain()),fnv(run.session.object_map_data()),fnv(&frame)));
    Ok(())
}
fn assignment(skill: AssignmentSkill, id: u32) -> StartupCommand {
    lemmings_game::pointer::command(skill, id)
}

pub fn trace(partitions: &[u32]) -> Result<String, String> {
    super::parts(partitions)?;
    let mut rows = Vec::new();
    for fallen in [60, 61] {
        let mut run = bare();
        let mut s = state(MotionAction::Falling);
        s.fallen = fallen;
        set_actor(&mut run, s);
        record(&mut rows, &format!("landing-{fallen}"), run, 1, partitions)?;
    }
    for bricks in 1..=12 {
        let mut run = bare();
        let mut s = state(MotionAction::Building);
        s.frame = 9;
        s.bricks_left = bricks;
        set_actor(&mut run, s);
        record(&mut rows, &format!("builder-{bricks}"), run, 1, partitions)?;
    }
    for nuked in [0, 255] {
        for (flags, action) in [
            (0, MotionAction::Walking),
            (4, MotionAction::Falling),
            (0x400, MotionAction::Floating),
            (0x1000, MotionAction::Drowning),
            (0x4000, MotionAction::Vaporizing),
        ] {
            let mut run = if nuked == 0 {
                bare()
            } else {
                let mut assets = Assets::synthetic();
                assets.config.total_lemmings = 1;
                let mut run = LevelRun::new(assets, 1, None, Pacing::Dos).unwrap();
                for _ in 0..55 {
                    run.tick().unwrap();
                }
                run.sound.drain();
                run
            };
            let mut s = state(action);
            s.explosion_timer = 1;
            set_actor(&mut run, s);
            if nuked != 0 {
                run.session
                    .queue(InputCommand::Custom(StartupCommand::Nuke))
                    .unwrap();
            }
            record(
                &mut rows,
                &format!("expiry-{flags}-{nuked}"),
                run,
                1,
                partitions,
            )?;
        }
    }
    let mut run = bare();
    let mut s = state(MotionAction::OhNo);
    s.frame = 15;
    set_actor(&mut run, s);
    record(&mut rows, "ohno-end", run, 1, partitions)?;
    for (name, code) in [
        ("exit", OBJECT_EXIT),
        ("drowning", OBJECT_WATER),
        ("vaporizing", OBJECT_FIRE),
    ] {
        let mut run = bare();
        set_actor(&mut run, state(MotionAction::Walking));
        map(&mut run, 40, 100, code);
        record(&mut rows, name, run, 1, partitions)?;
    }
    for (name, action, frame, x, y) in [
        ("digger-steel", MotionAction::Digging, 7, 40, 101),
        ("basher-steel", MotionAction::Bashing, 10, 49, 92),
        ("miner-steel", MotionAction::Mining, 2, 42, 101),
    ] {
        let mut run = bare();
        let mut s = state(action);
        s.frame = frame;
        set_actor(&mut run, s);
        map(&mut run, x, y, 137);
        record(&mut rows, name, run, 1, partitions)?;
    }
    let mut run = bare();
    run.session.notify_skill_selection();
    record(&mut rows, "skill-select", run, 1, partitions)?;
    for (job, action, frame) in [
        ("basher", MotionAction::Bashing, 10),
        ("miner", MotionAction::Mining, 2),
    ] {
        for direction in [MotionDirection::Left, MotionDirection::Right] {
            for raw_code in [7, 8, 9] {
                let mut run = bare();
                let mut s = state(action);
                s.direction = direction;
                s.frame = frame;
                set_actor(&mut run, s);
                let (x, y) = if action == MotionAction::Bashing {
                    (40 + 9 * direction.delta(), 92)
                } else {
                    (40 + 2 * direction.delta(), 101)
                };
                map(&mut run, i64::from(x), y, 128 + raw_code);
                record(
                    &mut rows,
                    &format!("oneway-{job}-{}-{raw_code}", direction.delta()),
                    run,
                    1,
                    partitions,
                )?;
            }
        }
    }
    let skills = [
        ("climber", AssignmentSkill::Climber),
        ("floater", AssignmentSkill::Floater),
        ("bomber", AssignmentSkill::Bomber),
        ("blocker", AssignmentSkill::Blocker),
        ("builder", AssignmentSkill::Builder),
        ("basher", AssignmentSkill::Basher),
        ("miner", AssignmentSkill::Miner),
        ("digger", AssignmentSkill::Digger),
    ];
    for (name, skill) in skills {
        for empty in [false, true] {
            let mut run = bare();
            set_actor(&mut run, state(MotionAction::Walking));
            if empty {
                run.session.remaining_climbers = 0;
                run.session.remaining_floaters = 0;
                run.session.remaining_bombers = 0;
                run.session.remaining_blockers = 0;
                run.session.remaining_builders = 0;
                run.session.remaining_bashers = 0;
                run.session.remaining_miners = 0;
                run.session.remaining_diggers = 0;
            }
            run.session
                .queue(InputCommand::Custom(assignment(skill, 0)))
                .unwrap();
            record(
                &mut rows,
                &format!("{}-{name}", if empty { "empty" } else { "assign" }),
                run,
                1,
                partitions,
            )?;
        }
    }
    for (name, skill) in skills {
        if !matches!(
            skill,
            AssignmentSkill::Basher | AssignmentSkill::Miner | AssignmentSkill::Digger
        ) {
            continue;
        }
        for (below, front) in [(0, 9), (9, 9), (0, 7)] {
            let mut run = bare();
            let mut s = state(MotionAction::Walking);
            s.object_below = 128 + below;
            s.object_in_front = 128 + front;
            set_actor(&mut run, s);
            run.session
                .queue(InputCommand::Custom(assignment(skill, 0)))
                .unwrap();
            record(
                &mut rows,
                &format!("blocked-{name}-{below}-{front}"),
                run,
                1,
                partitions,
            )?;
        }
    }
    for raw in 0..=18 {
        let mut run = bare();
        run.sound = lemmings_game::sound::SoundTimeline::new(0, {
            let mut raw_ids = [0; 16];
            raw_ids[0] = raw;
            raw_ids
        })
        .unwrap();
        let mut traps = [None; 16];
        traps[0] = Some(TrapState::new(12, 0).unwrap());
        run.session.configure_traps(traps);
        set_actor(&mut run, state(MotionAction::Walking));
        map(&mut run, 40, 100, 0);
        record(&mut rows, &format!("trap-{raw}"), run, 1, partitions)?;
    }
    for counter in [14, 15, 16, 34, 35, 36, 54, 55, 56] {
        record(
            &mut rows,
            &format!("startup-{counter}"),
            bare(),
            counter,
            partitions,
        )?;
    }
    let mut run = bare();
    set_actor(&mut run, state(MotionAction::Falling));
    map(&mut run, 40, 100, OBJECT_EXIT);
    record(&mut rows, "splat-exit", run, 1, partitions)?;
    let mut music_rows = Vec::new();
    for special in 0..=4 {
        for advance in [false, true] {
            for cursor in 0..17 {
                let mut music = DosMusic::default();
                for _ in 0..cursor {
                    music.select(0, true).unwrap();
                }
                let chosen = music.select(special, advance).map_err(|e| e.to_string())? + 1;
                music_rows.push(format!("{{\"special\":{special},\"advance\":{advance},\"cursor\":{cursor},\"next_cursor\":{},\"raw_music\":{chosen}}}",music.cursor()));
            }
        }
    }
    let mut old =
        LevelRun::new(Assets::synthetic_completion("win"), 1, Some(2), Pacing::Dos).unwrap();
    for _ in 0..1200 {
        old.tick().unwrap();
        if old.session.result().is_some() {
            break;
        }
    }
    let old_batch = old.sound.drain();
    let mut next = old
        .prepare(ResultAction::Continue, Pacing::Dos, |_| {
            Ok((Assets::synthetic(), None))
        })
        .unwrap()
        .unwrap();
    for _ in 0..55 {
        next.tick().unwrap();
    }
    let next_batch = next.sound.drain();
    let mut retry = old
        .prepare(ResultAction::Restart, Pacing::Dos, |_| {
            Ok((Assets::synthetic(), None))
        })
        .unwrap()
        .unwrap();
    let retry_batch = retry.sound.drain();
    let before = old.sound.diagnostic_json();
    let failed = old
        .prepare(ResultAction::Continue, Pacing::Dos, |_| {
            Err("missing level".into())
        })
        .is_err();
    let failed_preserved = failed && before == old.sound.diagnostic_json();
    Ok(format!(
        "{{\"cases\":[{}],\"music_cases\":[{}],\"lifecycle\":{{\"old\":{},\"next\":{},\"retry\":{},\"failed_preserved\":{failed_preserved}}}}}",
        rows.join(","),
        music_rows.join(","),
        SoundBatch::to_json(&old_batch),
        next_batch.to_json(),
        retry_batch.to_json()
    ))
}
