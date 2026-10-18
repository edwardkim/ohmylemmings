//! Development adapter: allocation and JSON stay outside the no_std library.
use lemmings_core::{
    Actor, EntrancePoint, FixedTickClock, InputCommand, InputSchedule, MotionRules, StartupCommand,
    StartupConfig, StartupEvent, StartupScheduler, TerrainMask, TimedInput,
};
use std::{
    error::Error,
    fmt::Write as _,
    io::{self, Read, Write},
    time::Duration,
};

struct Run {
    width: usize,
    height: usize,
    config: StartupConfig,
    rules: MotionRules,
    entrances: Vec<EntrancePoint>,
    inputs: Vec<TimedInput<StartupCommand>>,
    steps: u64,
}
fn parse(text: &str) -> Result<Run, Box<dyn Error>> {
    let mut words = text.split_whitespace();
    let mut next = || -> Result<i64, Box<dyn Error>> {
        Ok(words.next().ok_or("truncated replay script")?.parse()?)
    };
    let width = usize::try_from(next()?)?;
    let height = usize::try_from(next()?)?;
    let minutes = u16::try_from(next()?)?;
    let total = u32::try_from(next()?)?;
    let rate = u8::try_from(next()?)?;
    let count = u8::try_from(next()?)?;
    fn flag(v: i64) -> Result<bool, Box<dyn Error>> {
        match v {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err("boolean must be 0 or 1".into()),
        }
    }
    let pause_glitch = flag(next()?)?;
    let faller_starts_with_three = flag(next()?)?;
    let steps = u64::try_from(next()?)?;
    if width == 0
        || height == 0
        || width.checked_mul(height).is_none_or(|n| n > 10_000_000)
        || count > 4
        || total > 10000
        || steps > 100000
    {
        return Err("replay exceeds development adapter limits".into());
    }
    let mut entrances = Vec::new();
    for _ in 0..count {
        entrances.push(EntrancePoint {
            x: i32::try_from(next()?)?,
            y: i32::try_from(next()?)?,
        });
    }
    let input_count = usize::try_from(next()?)?;
    if input_count > 1000000 {
        return Err("too many inputs".into());
    }
    let mut inputs = Vec::new();
    for _ in 0..input_count {
        let control_step = u64::try_from(next()?)?;
        let sequence = u32::try_from(next()?)?;
        let code = next()?;
        let value = u8::try_from(next()?)?;
        let command = match (code, value) {
            (0, 0) => InputCommand::Pause,
            (1, 0) => InputCommand::Resume,
            (2, rate) => InputCommand::Custom(StartupCommand::SetReleaseRate(rate)),
            _ => return Err("unknown command or argument".into()),
        };
        if control_step >= steps {
            return Err("input outside run".into());
        }
        inputs.push(TimedInput {
            control_step,
            sequence,
            command,
        });
    }
    if words.next().is_some() {
        return Err("trailing replay data".into());
    }
    InputSchedule::new(&inputs)?;
    let config = StartupConfig {
        time_limit_minutes: minutes,
        total_lemmings: total,
        initial_release_rate: rate,
        entrance_count: count,
        pause_glitch,
    };
    StartupScheduler::new(config)?;
    Ok(Run {
        width,
        height,
        config,
        rules: MotionRules {
            faller_starts_with_three,
        },
        entrances,
        inputs,
        steps,
    })
}
fn snapshot(scheduler: &StartupScheduler, actors: &[Actor], events: &[String]) -> String {
    let s = scheduler.state();
    let step = scheduler.step_state();
    let removed = actors
        .iter()
        .filter(|a| a.state().removed.is_some())
        .count();
    let mut out = format!(
        "{{\"control_step\":{},\"iteration\":{},\"paused\":{},\"seconds_remaining\":{},\"clock_frame\":{},\"glitch_pause_iterations\":{},\"release_rate\":{},\"spawn_countdown\":{},\"entrances_open\":{},\"released\":{},\"alive\":{},\"removed\":{},\"actors\":[",
        step.control_step,
        step.iteration,
        step.paused,
        s.seconds_remaining,
        s.clock_frame,
        s.glitch_pause_iterations,
        s.release_rate,
        s.spawn_countdown,
        s.entrances_open,
        s.released,
        actors.len() - removed,
        removed
    );
    for (i, actor) in actors.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        let a = actor.state();
        let removal = a
            .removed
            .map_or_else(|| "null".to_owned(), |r| format!("\"{r:?}\""));
        write!(out, "{{\"id\":{},\"born_iteration\":{},\"x\":{},\"y\":{},\"direction\":{},\"action\":\"{:?}\",\"frame\":{},\"end_of_animation\":{},\"fallen\":{},\"removed\":{removal}}}",
            a.id, a.born_iteration, a.x, a.y, a.direction.delta(), a.action, a.frame, a.end_of_animation, a.fallen).unwrap();
    }
    out.push_str("],\"events\":[");
    for (i, event) in events.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        write!(out, "\"{event}\"").unwrap();
    }
    out.push_str("]}");
    out
}
fn replay(
    run: &Run,
    cells: &[u8],
    pattern: &str,
    mut emit: impl FnMut(String) -> io::Result<()>,
) -> Result<(), Box<dyn Error>> {
    let durations: &[u64] = match pattern {
        "single" => &[58_823_530],
        "burst" => &[1_000_000_000],
        "jitter" => &[1, 31_000_000, 199_000_000, 7, 370_000_000],
        _ => return Err("unknown render pattern".into()),
    };
    let terrain = TerrainMask::new(run.width, run.height, cells)?;
    let mut scheduler = StartupScheduler::new(run.config)?;
    let schedule = InputSchedule::new(&run.inputs)?;
    let mut actors = Vec::new();
    let mut clock = FixedTickClock::new();
    let mut frame = 0;
    emit(snapshot(&scheduler, &actors, &[]))?;
    while scheduler.step_state().control_step < run.steps {
        let due = clock.advance(Duration::from_nanos(durations[frame % durations.len()]))?;
        frame += 1;
        for _ in 0..due.min(run.steps - scheduler.step_state().control_step) {
            let mut events = Vec::new();
            let mut spawn = None;
            let outcome = scheduler.step(&schedule, |event| match event {
                StartupEvent::Input {
                    control_step,
                    sequence,
                    accepted,
                } => events.push(format!("Input:{control_step}:{sequence}:{accepted}")),
                StartupEvent::Spawn(request) => {
                    events.push(format!(
                        "Spawn:{}:{}:{}",
                        request.actor_id, request.entrance_index, request.iteration
                    ));
                    spawn = Some(request);
                }
                StartupEvent::LetsGo => events.push("LetsGo".into()),
                StartupEvent::EntranceSound => events.push("EntranceSound".into()),
                StartupEvent::EntrancesOpened => events.push("EntrancesOpened".into()),
                StartupEvent::Tick(_) => {}
            })?;
            if let Some(request) = spawn {
                actors.push(Actor::from_spawn(request, &run.entrances, run.rules)?);
            }
            if outcome.iteration.is_some() {
                // Preserve ID order and complete removed records. No object updates in G05.
                for actor in &mut actors {
                    actor.advance(terrain)?;
                }
            }
            emit(snapshot(&scheduler, &actors, &events))?;
        }
    }
    Ok(())
}
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: trace_replay MASK_FILE single|burst|jitter; script on stdin".into());
    }
    let mut script = String::new();
    io::stdin().read_to_string(&mut script)?;
    let run = parse(&script)?;
    let cells = std::fs::read(&args[0])?;
    let mut output = io::BufWriter::new(io::stdout().lock());
    replay(&run, &cells, &args[1], |s| writeln!(output, "{s}"))?;
    output.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    const SCRIPT: &str = "96 160 5 10 50 1 1 1 340\n0 0\n3\n16 0 0 0\n34 0 1 0\n54 0 2 99\n";
    fn trace(pattern: &str) -> Vec<String> {
        let mut states = Vec::new();
        replay(&parse(SCRIPT).unwrap(), &vec![0; 96 * 160], pattern, |s| {
            states.push(s);
            Ok(())
        })
        .unwrap();
        states
    }
    #[test]
    fn every_snapshot_is_render_frequency_independent() {
        let single = trace("single");
        assert_eq!(single.len(), 341);
        assert_eq!(single, trace("burst"));
        assert_eq!(single, trace("jitter"));
        assert!(single[17].contains("\"iteration\":16,\"paused\":true"));
        assert!(single[72].contains("\"born_iteration\":54,\"x\":24,\"y\":17"));
        assert!(single[340].contains("BottomBoundary"));
    }
    #[test]
    fn malformed_scripts_fail_before_replay() {
        for invalid in [
            SCRIPT.replace("16 0 0 0", "340 0 0 0"),
            SCRIPT.replace("34 0 1 0", "16 0 1 0"),
            SCRIPT.replace("54 0 2 99", "54 0 3 99"),
            format!("{SCRIPT}trailing"),
            "96 160".into(),
        ] {
            assert!(parse(&invalid).is_err(), "{invalid}");
        }
    }
    #[test]
    fn absent_or_nonbinary_terrain_is_rejected() {
        assert!(replay(&parse(SCRIPT).unwrap(), &[], "single", |_| Ok(())).is_err());
        assert!(
            replay(
                &parse(SCRIPT).unwrap(),
                &vec![2; 96 * 160],
                "single",
                |_| Ok(())
            )
            .is_err()
        );
    }
}
