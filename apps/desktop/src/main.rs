mod assets;
mod input;
mod lifecycle;
mod pacing;
mod panel;
mod render;
mod session;
use assets::Assets;
use lemmings_core::{InputCommand, StartupCommand};
use lifecycle::{LevelRun, ResultAction};
use pacing::Pacing;
use render::{HEIGHT, WIDTH};
use sdl2::{
    event::Event,
    keyboard::{Keycode, Mod},
    pixels::PixelFormatEnum,
};
use session::Session;
use std::{
    error::Error,
    fs::{self, File, OpenOptions},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

const USAGE: &str = "Usage: lemmings-desktop [--assets DIR --level FUN_NUMBER | --synthetic | --synthetic-basher | --synthetic-builder | --synthetic-blocker | --synthetic-bomber | --synthetic-climber | --synthetic-floater | --synthetic-nuke | --synthetic-exit | --synthetic-exit-drop | --synthetic-water | --synthetic-fire | --synthetic-trap | --synthetic-trap-slow | --synthetic-trap-continuous | --synthetic-water-drop | --synthetic-fire-drop | --synthetic-water-late | --synthetic-fire-late | --synthetic-outcome win|loss|clock|particles|zero|mixed] [--steps N] [--fast] [--render-every N] [--pacing dos|reference] [--capture NEW_DIR] [--replay FILE | --demo-input] [--result-actions restart,continue]\nF/C/E/K/U/M/B/D: select Floater/Climber/Bomber/Blocker/Builder/Miner/Basher/Digger; left click: assign selected skill; N: nuke; Space: pause/resume; Up/Down: release rate; Left/Right/Home/End: camera; Esc: quit. On results: Enter/left click: retry or next; R: restart; Esc/right click: quit.\nReplay: CONTROL_STEP SEQUENCE Pause|Resume|Nuke|SetReleaseRate RATE|AssignDigger ACTOR_ID|AssignBasher ACTOR_ID|AssignMiner ACTOR_ID|AssignBuilder ACTOR_ID|AssignBlocker ACTOR_ID|AssignBomber ACTOR_ID|AssignClimber ACTOR_ID|AssignFloater ACTOR_ID.";
struct Options {
    assets: PathBuf,
    synthetic: bool,
    basher_scene: bool,
    builder_scene: bool,
    blocker_scene: bool,
    climber_scene: bool,
    floater_scene: bool,
    exit_scene: Option<bool>,
    hazard_scene: Option<(u8, bool)>,
    outcome_scene: Option<String>,
    result_actions: std::collections::VecDeque<ResultAction>,
    level: u8,
    steps: Option<u64>,
    fast: bool,
    render_every: u64,
    capture: Option<PathBuf>,
    replay: Option<PathBuf>,
    demo_input: bool,
    pacing: Option<Pacing>,
}
impl Options {
    fn parse(args: impl Iterator<Item = String>) -> Result<Option<Self>, Box<dyn Error>> {
        let mut args = args.peekable();
        let mut o = Self {
            assets: "assets/local/dos-original".into(),
            synthetic: false,
            basher_scene: false,
            builder_scene: false,
            blocker_scene: false,
            climber_scene: false,
            floater_scene: false,
            exit_scene: None,
            hazard_scene: None,
            outcome_scene: None,
            result_actions: std::collections::VecDeque::new(),
            level: 1,
            steps: None,
            fast: false,
            render_every: 1,
            capture: None,
            replay: None,
            demo_input: false,
            pacing: None,
        };
        let mut specified_assets = false;
        while let Some(flag) = args.next() {
            match flag.as_str() {
                "--help" | "-h" => return Ok(None),
                "--result-actions" => {
                    let value = args
                        .next()
                        .ok_or("--result-actions needs restart,continue actions")?;
                    for word in value.split(',') {
                        o.result_actions.push_back(match word {
                            "restart" => ResultAction::Restart,
                            "continue" => ResultAction::Continue,
                            _ => return Err("result actions: restart or continue".into()),
                        });
                    }
                    if o.result_actions.len() > 32 {
                        return Err("at most 32 result actions".into());
                    }
                }
                "--synthetic" => o.synthetic = true,
                "--synthetic-outcome" => {
                    let kind = args
                        .next()
                        .ok_or("--synthetic-outcome needs win|loss|clock|particles|zero|mixed")?;
                    if !matches!(
                        kind.as_str(),
                        "win" | "loss" | "clock" | "particles" | "zero" | "mixed"
                    ) {
                        return Err("unknown outcome scenario".into());
                    }
                    o.synthetic = true;
                    o.outcome_scene = Some(kind);
                }
                "--synthetic-water-late"
                | "--synthetic-fire-late"
                | "--synthetic-water"
                | "--synthetic-fire"
                | "--synthetic-trap"
                | "--synthetic-trap-slow"
                | "--synthetic-trap-continuous"
                | "--synthetic-water-drop"
                | "--synthetic-fire-drop" => {
                    o.synthetic = true;
                    let kind = if flag.contains("water") {
                        if flag.ends_with("-late") { 143 } else { 133 }
                    } else if flag.contains("fire") {
                        if flag.ends_with("-late") { 144 } else { 134 }
                    } else if flag.ends_with("-slow") {
                        1
                    } else if flag.ends_with("-continuous") {
                        2
                    } else {
                        0
                    };
                    o.hazard_scene = Some((kind, flag.ends_with("-drop")));
                }
                "--synthetic-exit" | "--synthetic-exit-drop" => {
                    o.synthetic = true;
                    o.exit_scene = Some(flag == "--synthetic-exit-drop");
                }
                "--synthetic-basher" => {
                    o.synthetic = true;
                    o.basher_scene = true;
                }
                "--synthetic-builder" => {
                    o.synthetic = true;
                    o.builder_scene = true;
                }
                "--synthetic-floater" => {
                    o.synthetic = true;
                    o.floater_scene = true;
                }
                "--synthetic-climber" => {
                    o.synthetic = true;
                    o.climber_scene = true;
                }
                "--synthetic-nuke" | "--synthetic-bomber" | "--synthetic-blocker" => {
                    o.synthetic = true;
                    o.blocker_scene = true;
                }
                "--level" => {
                    o.level = args
                        .next()
                        .ok_or("--level requires Fun number 1..30")?
                        .parse()?
                }
                "--fast" => o.fast = true,
                "--demo-input" => o.demo_input = true,
                "--pacing" => {
                    o.pacing = Some(match args.next().as_deref() {
                        Some("dos") => Pacing::Dos,
                        Some("reference") => Pacing::Reference,
                        _ => return Err("--pacing requires dos or reference".into()),
                    });
                }
                "--assets" => {
                    o.assets = args.next().ok_or("--assets requires a directory")?.into();
                    specified_assets = true;
                }
                "--steps" => {
                    o.steps = Some(args.next().ok_or("--steps requires an integer")?.parse()?)
                }
                "--render-every" => {
                    o.render_every = args
                        .next()
                        .ok_or("--render-every requires an integer")?
                        .parse()?
                }
                "--capture" => {
                    o.capture = Some(
                        args.next()
                            .ok_or("--capture requires a new directory")?
                            .into(),
                    )
                }
                "--replay" => {
                    o.replay = Some(args.next().ok_or("--replay requires a file")?.into())
                }
                _ => return Err(format!("unknown option {flag}\n{USAGE}").into()),
            }
        }
        if o.synthetic && specified_assets {
            return Err("choose --assets or --synthetic".into());
        }
        if !(1..=30).contains(&o.level) || (o.synthetic && o.level != 1) {
            return Err("--level: Fun 1..30 with original assets".into());
        }
        if !o.result_actions.is_empty() && o.steps.is_none() {
            return Err("--result-actions requires bounded --steps per attempt".into());
        }
        if o.fast && o.steps.is_none() {
            return Err("--fast requires bounded --steps".into());
        }
        if o.steps.is_some_and(|n| n == 0 || n > 100000)
            || o.render_every == 0
            || o.render_every > 10000
        {
            return Err("steps: 1..100000; render-every: 1..10000".into());
        }
        if o.demo_input && !o.fast {
            return Err("--demo-input requires --fast".into());
        }
        if o.demo_input && o.replay.is_some() {
            return Err("choose --replay or --demo-input".into());
        }
        Ok(Some(o))
    }
}
struct Capture {
    root: PathBuf,
    trace: BufWriter<File>,
    digger_trace: BufWriter<File>,
    basher_trace: BufWriter<File>,
    miner_trace: BufWriter<File>,
    builder_trace: BufWriter<File>,
    blocker_trace: BufWriter<File>,
    bomber_trace: BufWriter<File>,
    climber_trace: BufWriter<File>,
    floater_trace: BufWriter<File>,
    nuke_trace: BufWriter<File>,
    exit_trace: BufWriter<File>,
    hazards_trace: BufWriter<File>,
    completion_trace: BufWriter<File>,
    previous_object_map: Vec<u8>,
    previous_terrain: Vec<u8>,
}
fn write_new(path: &Path, data: &[u8]) -> Result<(), Box<dyn Error>> {
    let mut f = OpenOptions::new().write(true).create_new(true).open(path)?;
    f.write_all(data)?;
    Ok(())
}
impl Capture {
    fn new(root: &Path, synthetic: bool) -> Result<Self, Box<dyn Error>> {
        if !synthetic {
            let parent = fs::canonicalize(root.parent().ok_or("capture parent required")?)?;
            let local =
                fs::canonicalize(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/local"))?;
            if !parent.starts_with(local) {
                return Err("original-derived captures must stay under assets/local".into());
            }
        }
        fs::create_dir(root)?;
        let trace = BufWriter::new(
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(root.join("trace.jsonl"))?,
        );
        Ok(Self {
            root: root.to_path_buf(),
            trace,
            previous_terrain: Vec::new(),
            previous_object_map: Vec::new(),
            completion_trace: BufWriter::new(
                OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(root.join("completion.jsonl"))?,
            ),
            hazards_trace: BufWriter::new(
                OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(root.join("hazards.jsonl"))?,
            ),
            exit_trace: BufWriter::new(
                OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(root.join("exit.jsonl"))?,
            ),
            nuke_trace: BufWriter::new(
                OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(root.join("nuke.jsonl"))?,
            ),
            floater_trace: BufWriter::new(
                OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(root.join("floater.jsonl"))?,
            ),
            climber_trace: BufWriter::new(
                OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(root.join("climber.jsonl"))?,
            ),
            bomber_trace: BufWriter::new(
                OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(root.join("bomber.jsonl"))?,
            ),
            blocker_trace: BufWriter::new(
                OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(root.join("blocker.jsonl"))?,
            ),
            builder_trace: BufWriter::new(
                OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(root.join("builder.jsonl"))?,
            ),
            miner_trace: BufWriter::new(
                OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(root.join("miner.jsonl"))?,
            ),
            basher_trace: BufWriter::new(
                OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(root.join("basher.jsonl"))?,
            ),
            digger_trace: BufWriter::new(
                OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(root.join("digger.jsonl"))?,
            ),
        })
    }
    fn snapshot(&mut self, s: &Session) -> Result<(), Box<dyn Error>> {
        writeln!(self.trace, "{}", s.snapshot())?;
        writeln!(self.exit_trace, "{}", s.exit_snapshot())?;
        writeln!(self.hazards_trace, "{}", s.hazards_snapshot())?;
        writeln!(self.completion_trace, "{}", s.completion_snapshot())?;
        self.completion_trace.flush()?;
        writeln!(self.bomber_trace, "{}", s.bomber_snapshot())?;
        writeln!(self.climber_trace, "{}", s.climber_snapshot())?;
        writeln!(self.floater_trace, "{}", s.floater_snapshot())?;
        let n = s.scheduler.nuke_state();
        writeln!(
            self.nuke_trace,
            "{{\"is_nuked\":{},\"assigning\":{},\"next_index\":{},\"max_lemmings\":{}}}",
            n.is_nuked, n.assigning, n.next_index, n.max_lemmings
        )?;
        writeln!(self.miner_trace, "{{\"remaining\":{}}}", s.remaining_miners)?;
        self.trace.flush()?;
        writeln!(
            self.basher_trace,
            "{{\"remaining\":{}}}",
            s.remaining_bashers
        )?;
        let cleared: Vec<_> = self
            .previous_terrain
            .iter()
            .zip(s.terrain())
            .enumerate()
            .filter(|(_, (before, after))| **before == 1 && **after == 0)
            .map(|(i, _)| i.to_string())
            .collect();
        if self.previous_terrain.is_empty() {
            write_new(&self.root.join("terrain-initial.bin"), s.terrain())?;
        }
        let mut details = s.digger_snapshot();
        details.pop();
        writeln!(
            self.digger_trace,
            "{details},\"cleared\":[{}]}}",
            cleared.join(",")
        )?;
        let filled: Vec<_> = self
            .previous_terrain
            .iter()
            .zip(s.terrain())
            .enumerate()
            .filter(|(_, (before, after))| **before == 0 && **after == 1)
            .map(|(i, _)| i.to_string())
            .collect();
        let mut builder = s.builder_snapshot();
        builder.pop();
        writeln!(
            self.builder_trace,
            "{builder},\"filled\":[{}]}}",
            filled.join(",")
        )?;
        let changed = self
            .previous_object_map
            .iter()
            .zip(s.object_map_data())
            .enumerate()
            .filter(|(_, (a, b))| a != b)
            .map(|(i, (_, b))| format!("[{i},{b}]"))
            .collect::<Vec<_>>()
            .join(",");
        let mut blocker = s.blocker_snapshot();
        blocker.pop();
        writeln!(self.blocker_trace, "{blocker},\"changed\":[{changed}]}}")?;
        self.previous_object_map.clear();
        self.previous_object_map
            .extend_from_slice(s.object_map_data());
        self.previous_terrain.clear();
        self.previous_terrain.extend_from_slice(s.terrain());
        Ok(())
    }
    fn frame(
        &mut self,
        step: u64,
        logical: &[u8],
        pixels: &[u8],
        size: (u32, u32),
    ) -> Result<(), Box<dyn Error>> {
        write_new(&self.root.join(format!("frame-{step:06}.rgb")), logical)?;
        let mut ppm = format!("P6\n{} {}\n255\n", size.0, size.1).into_bytes();
        ppm.extend_from_slice(pixels);
        write_new(&self.root.join(format!("frame-{step:06}.ppm")), &ppm)
    }
    fn finish(&mut self, s: &Session, summary: &str) -> Result<(), Box<dyn Error>> {
        self.trace.flush()?;
        self.digger_trace.flush()?;
        self.basher_trace.flush()?;
        self.miner_trace.flush()?;
        self.builder_trace.flush()?;
        self.blocker_trace.flush()?;
        self.bomber_trace.flush()?;
        self.climber_trace.flush()?;
        self.floater_trace.flush()?;
        self.nuke_trace.flush()?;
        self.exit_trace.flush()?;
        self.hazards_trace.flush()?;
        self.completion_trace.flush()?;
        write_new(&self.root.join("object-map-final.bin"), s.object_map_data())?;
        write_new(&self.root.join("terrain-final.bin"), s.terrain())?;
        write_new(&self.root.join("brick-final.bin"), s.brick_mask())?;
        let mut inputs = String::new();
        for i in &s.inputs {
            let command = match i.command {
                InputCommand::Pause => "Pause".to_owned(),
                InputCommand::Resume => "Resume".to_owned(),
                InputCommand::Custom(StartupCommand::Nuke) => "Nuke".to_owned(),
                InputCommand::Custom(StartupCommand::AssignDigger(id)) => {
                    format!("AssignDigger {id}")
                }
                InputCommand::Custom(StartupCommand::AssignBasher(id)) => {
                    format!("AssignBasher {id}")
                }
                InputCommand::Custom(StartupCommand::AssignMiner(id)) => {
                    format!("AssignMiner {id}")
                }
                InputCommand::Custom(StartupCommand::AssignBuilder(id)) => {
                    format!("AssignBuilder {id}")
                }
                InputCommand::Custom(StartupCommand::AssignFloater(id)) => {
                    format!("AssignFloater {id}")
                }
                InputCommand::Custom(StartupCommand::AssignClimber(id)) => {
                    format!("AssignClimber {id}")
                }
                InputCommand::Custom(StartupCommand::AssignBomber(id)) => {
                    format!("AssignBomber {id}")
                }
                InputCommand::Custom(StartupCommand::AssignBlocker(id)) => {
                    format!("AssignBlocker {id}")
                }
                InputCommand::Custom(StartupCommand::SetReleaseRate(rate)) => {
                    format!("SetReleaseRate {rate}")
                }
            };
            inputs.push_str(&format!("{} {} {command}\n", i.control_step, i.sequence));
        }
        write_new(&self.root.join("inputs.txt"), inputs.as_bytes())?;
        write_new(&self.root.join("run.json"), summary.as_bytes())
    }
}
fn key_event(keycode: Keycode) -> Event {
    Event::KeyDown {
        timestamp: 0,
        window_id: 0,
        keycode: Some(keycode),
        scancode: None,
        keymod: Mod::NOMOD,
        repeat: false,
    }
}
fn basher_capture(s: &Session) -> bool {
    matches!(s.scheduler.step_state().control_step, 220 | 260 | 300)
        && s.inputs.iter().any(|i| {
            matches!(
                i.command,
                InputCommand::Custom(
                    StartupCommand::AssignBasher(_)
                        | StartupCommand::AssignMiner(_)
                        | StartupCommand::AssignBuilder(_)
                        | StartupCommand::AssignBlocker(_)
                        | StartupCommand::AssignBomber(_)
                        | StartupCommand::AssignClimber(_)
                        | StartupCommand::AssignFloater(_)
                        | StartupCommand::Nuke
                )
            )
        })
}
fn bomber_capture(s: &Session) -> bool {
    s.inputs.iter().any(|i| {
        matches!(
            i.command,
            InputCommand::Custom(StartupCommand::AssignBomber(_) | StartupCommand::Nuke)
        )
    }) && s.actors.iter().any(|a| {
        let a = a.state();
        matches!(
            a.explosion_timer,
            1 | 16 | 17 | 32 | 33 | 48 | 49 | 64 | 65 | 78
        ) || (a.action == lemmings_core::MotionAction::OhNo && a.frame == 15)
            || (a.action == lemmings_core::MotionAction::Exploding && a.removed.is_none())
            || matches!(a.particle_timer, 1 | 52)
    })
}
fn climber_capture(s: &Session) -> bool {
    s.inputs.iter().any(|i| {
        matches!(
            i.command,
            InputCommand::Custom(StartupCommand::AssignClimber(_))
        )
    }) && s.actors.iter().any(|a| {
        let a = a.state();
        matches!(
            a.action,
            lemmings_core::MotionAction::Climbing | lemmings_core::MotionAction::Hoisting
        ) && matches!(a.frame, 0 | 3 | 4 | 7)
    })
}
fn hazard_capture(s: &Session) -> bool {
    s.hazard_just_happened()
        || s.actors.iter().any(|a| {
            let a = a.state();
            a.removed.is_none()
                && matches!(
                    a.action,
                    lemmings_core::MotionAction::Drowning | lemmings_core::MotionAction::Vaporizing
                )
                && matches!(a.frame, 0 | 7 | 13 | 15)
        })
        || s.traps()
            .iter()
            .flatten()
            .any(|t| t.triggered() && (t.frame() == 1 || t.frame() == t.frame_count() - 1))
}
fn exit_capture(s: &Session) -> bool {
    s.rescue_just_happened()
        || s.actors.iter().any(|a| {
            a.state().action == lemmings_core::MotionAction::Exiting && a.state().removed.is_none()
        })
}
fn floater_capture(s: &Session) -> bool {
    s.inputs.iter().any(|i| {
        matches!(
            i.command,
            InputCommand::Custom(StartupCommand::AssignFloater(_))
        )
    }) && s.actors.iter().any(|a| {
        let a = a.state();
        a.action == lemmings_core::MotionAction::Floating
            && matches!(a.float_index, 0 | 1 | 4 | 5 | 8 | 15)
    })
}
fn apply_replay_boundary(
    session: &mut Session,
    replay: &mut std::iter::Peekable<std::vec::IntoIter<lemmings_core::TimedInput<StartupCommand>>>,
) {
    if session.result().is_some() {
        return;
    }
    let step = session.scheduler.step_state().control_step;
    while replay
        .peek()
        .is_some_and(|input| input.control_step == step)
    {
        session.inputs.push(replay.next().unwrap());
    }
}
fn initialize_capture(
    c: &mut Capture,
    assets: &Assets,
    session: &Session,
    level: u8,
) -> Result<(), Box<dyn Error>> {
    c.snapshot(session)?;
    write_new(&c.root.join("bash-masks.bin"), &assets.bash_masks)?;
    write_new(&c.root.join("mine-masks.bin"), &assets.mine_masks)?;
    let objects: Vec<_> = assets
        .steel
        .iter()
        .zip(&assets.one_way_left)
        .zip(&assets.one_way_right)
        .map(|((&steel, &left), &right)| {
            if steel != 0 {
                137
            } else if left != 0 {
                135
            } else if right != 0 {
                136
            } else {
                0
            }
        })
        .collect();
    write_new(&c.root.join("objects.bin"), &objects)?;
    write_new(
        &c.root.join("object-map-initial.bin"),
        session.object_map_data(),
    )?;
    write_new(&c.root.join("explosion-mask.bin"), &assets.explosion_mask)?;
    let trap_metadata: Vec<_> = assets
        .traps
        .iter()
        .flat_map(|trap| {
            trap.map_or([0, 0, 0], |t| {
                [t.frame_count(), t.frame(), u8::from(t.continuous())]
            })
        })
        .collect();
    write_new(&c.root.join("traps.bin"), &trap_metadata)?;
    let config = &assets.config;
    let entrances = assets
        .entrances
        .iter()
        .map(|p| format!("[{},{}]", p.x, p.y))
        .collect::<Vec<_>>()
        .join(",");
    let description = format!(
        "{{\"width\":{},\"height\":{},\"minutes\":{},\"total\":{},\"rate\":{},\"entrances\":[{entrances}],\"pause_glitch\":true,\"nuke_glitch\":true,\"faller_three\":true,\"diggers\":{},\"bashers\":{},\"miners\":{},\"builders\":{},\"blockers\":{},\"bombers\":{},\"climbers\":{},\"floaters\":{},\"level\":{}}}",
        assets.width,
        assets.height,
        config.time_limit_minutes,
        config.total_lemmings,
        config.initial_release_rate,
        assets.diggers,
        assets.bashers,
        assets.miners,
        assets.builders,
        assets.blockers,
        assets.bombers,
        assets.climbers,
        assets.floaters,
        level
    );
    let description = format!(
        "{},\"required\":{}}}",
        description.trim_end_matches('}'),
        assets
            .required
            .map_or_else(|| "null".into(), |v| v.to_string())
    );
    write_new(&c.root.join("config.json"), description.as_bytes())?;
    Ok(())
}
fn run_summary(
    game: &LevelRun,
    rendered: u64,
    observed_events: u64,
    display: (&str, &str),
    mode: (bool, bool, Pacing),
) -> String {
    let (driver, renderer) = display;
    let (fast, synthetic, pacing) = mode;
    let state = game.session.scheduler.step_state();
    let runtime = sdl2::version::version();
    format!(
        "{{\"control_step\":{},\"iteration\":{},\"frames\":{rendered},\"input_count\":{},\"events_seen\":{observed_events},\"camera\":{},\"video_driver\":{:?},\"renderer\":{:?},\"sdl_version\":\"{}.{}.{}\",\"timing\":\"{}\",\"synthetic\":{},\"pacing\":{:?},\"rate_numerator\":{},\"rate_denominator\":{}}}",
        state.control_step,
        state.iteration,
        game.session.inputs.len(),
        game.view.camera,
        driver,
        renderer,
        runtime.major,
        runtime.minor,
        runtime.patch,
        if fast { "fixed-step" } else { "realtime" },
        synthetic,
        pacing.label(),
        pacing.rate().0,
        pacing.rate().1
    )
}
fn load_assets(options: &Options, level: u8) -> Result<(Assets, Option<u8>), Box<dyn Error>> {
    let assets = if let Some(kind) = &options.outcome_scene {
        Assets::synthetic_completion(if kind == "win" && level == 2 {
            "loss"
        } else {
            kind
        })
    } else if let Some((kind, drop)) = options.hazard_scene {
        Assets::synthetic_hazards(kind, drop)
    } else if let Some(drop) = options.exit_scene {
        Assets::synthetic_exit(drop)
    } else if options.floater_scene {
        Assets::synthetic_floater()
    } else if options.climber_scene {
        Assets::synthetic_climber()
    } else if options.blocker_scene || options.builder_scene {
        Assets::synthetic_builder()
    } else if options.basher_scene {
        Assets::synthetic_basher()
    } else if options.synthetic {
        Assets::synthetic()
    } else if level == 1 {
        Assets::load(&options.assets)?
    } else {
        Assets::load_level(&options.assets, level)?
    };
    let next = if options.synthetic {
        (options.outcome_scene.as_deref() == Some("win") && level == 1).then_some(2)
    } else {
        (level < 30).then_some(level + 1)
    };
    Ok((assets, next))
}
fn run(mut options: Options) -> Result<(), Box<dyn Error>> {
    let (assets, next) = load_assets(&options, options.level)?;
    let replay = if let Some(path) = &options.replay {
        input::parse_replay(&fs::read_to_string(path)?, options.steps)?
    } else {
        Vec::new()
    };
    let mut replay = replay.into_iter().peekable();
    let pacing = options.pacing.unwrap_or(if options.synthetic {
        Pacing::Reference
    } else {
        Pacing::Dos
    });
    let mut game = LevelRun::new(assets, options.level, next, pacing)?;
    game.view.replaying = options.replay.is_some();
    let mut capture = options
        .capture
        .as_ref()
        .map(|p| Capture::new(p, options.synthetic))
        .transpose()?;
    if let Some(c) = &mut capture {
        initialize_capture(c, &game.assets, &game.session, options.level)?;
    }
    let sdl = sdl2::init()?;
    let video = sdl.video()?;
    sdl2::hint::set("SDL_RENDER_SCALE_QUALITY", "0");
    let window = video
        .window(
            &format!("Oh My Lemmings - {}", game.assets.title),
            WIDTH as u32 * 3,
            HEIGHT as u32 * 3,
        )
        .position_centered()
        .resizable()
        .build()?;
    let mut canvas = window.into_canvas().software().build()?;
    canvas.set_logical_size(WIDTH as u32, HEIGHT as u32)?;
    canvas.set_integer_scale(true)?;
    let creator = canvas.texture_creator();
    let mut texture =
        creator.create_texture_streaming(PixelFormatEnum::RGB24, WIDTH as u32, HEIGHT as u32)?;
    let events = sdl.event()?;
    let mut pump = sdl.event_pump()?;
    let mut previous = Instant::now();
    let mut last_render = previous;
    let mut rendered = 0u64;
    let mut observed_events = 0u64;
    let mut initial = true;
    let mut last_captured = None;
    let mut demo_at = None;
    let mut pending_due = 0u64;
    let mut attempt = 0u32;
    let mut result_rendered = false;
    let mut transitions = options
        .capture
        .as_ref()
        .map(|p| {
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(p.join("transitions.jsonl"))
        })
        .transpose()?;
    'running: loop {
        let step = game.session.scheduler.step_state().control_step;
        if options.demo_input && demo_at != Some(step) {
            let key = match step {
                16 | 34 => Some(Keycode::Space),
                54 => Some(Keycode::Up),
                55 => Some(Keycode::Right),
                56 => Some(Keycode::Left),
                100 => Some(Keycode::Escape),
                _ => None,
            };
            if let Some(key) = key {
                events.push_event(key_event(key))?;
            }
            demo_at = Some(step);
        }
        if result_rendered && let Some(action) = options.result_actions.front() {
            events.push_event(key_event(match action {
                ResultAction::Restart => Keycode::R,
                ResultAction::Continue => Keycode::Return,
            }))?;
        }
        let mut action = None;
        for event in pump.poll_iter() {
            observed_events += 1;
            if game.session.result().is_some() {
                if let Some(request) = input::result_action(&event) {
                    action.get_or_insert(request);
                    continue;
                }
                if action.is_some()
                    && !matches!(
                        event,
                        Event::Quit { .. }
                            | Event::Window { .. }
                            | Event::KeyDown {
                                keycode: Some(Keycode::Escape),
                                ..
                            }
                    )
                {
                    continue;
                }
            }
            if !input::handle(
                event,
                &mut game.session,
                &mut game.view,
                game.assets.width,
                options.replay.is_some(),
            )? {
                break 'running;
            }
        }
        if let Some(action) = action {
            let scripted = !options.result_actions.is_empty();
            match game.prepare(action, pacing, |level| load_assets(&options, level)) {
                Ok(Some(next)) => {
                    if let Some(c) = &mut capture {
                        c.finish(
                            &game.session,
                            &run_summary(
                                &game,
                                rendered,
                                observed_events,
                                (video.current_video_driver(), canvas.info().name),
                                (options.fast, options.synthetic, pacing),
                            ),
                        )?;
                    }
                    if let Some(log) = &mut transitions {
                        writeln!(
                            log,
                            "{{\"attempt\":{attempt},\"level\":{},\"step\":{},\"action\":\"{action:?}\",\"target\":{}}}",
                            game.level,
                            game.session.scheduler.step_state().control_step,
                            next.level
                        )?;
                    }
                    attempt += 1;
                    capture = options
                        .capture
                        .as_ref()
                        .map(|root| {
                            Capture::new(
                                &root.join(format!("attempt-{attempt:04}")),
                                options.synthetic,
                            )
                        })
                        .transpose()?;
                    game = next;
                    if let Some(c) = &mut capture {
                        initialize_capture(c, &game.assets, &game.session, game.level)?;
                    }
                    replay = Vec::new().into_iter().peekable();
                    options.replay = None;
                    options.demo_input = false;
                    if scripted {
                        options.result_actions.pop_front();
                    }
                    canvas
                        .window_mut()
                        .set_title(&format!("Oh My Lemmings - {}", game.assets.title))?;
                    pending_due = 0;
                    rendered = 0;
                    observed_events = 0;
                    initial = true;
                    last_captured = None;
                    result_rendered = false;
                    demo_at = None;
                    // Inputs already queued for the old result must not assign a skill in the new game.
                    for event in pump.poll_iter() {
                        if matches!(event, Event::Quit { .. }) {
                            break 'running;
                        }
                    }
                    previous = Instant::now();
                }
                Ok(None) if scripted => {
                    return Err("requested result transition is unavailable".into());
                }
                Ok(None) => {}
                Err(error) if scripted => return Err(error),
                Err(error) => {
                    eprintln!("Cannot load level: {error}");
                    game.view.result_error = true;
                    result_rendered = false;
                }
            }
        }
        let now = Instant::now();
        let elapsed = now.duration_since(previous);
        previous = now;
        let due = if initial || game.session.result().is_some() {
            0
        } else if options.fast {
            1
        } else {
            game.session.due(elapsed)?
        };
        pending_due = pending_due
            .checked_add(due)
            .ok_or("pending tick overflow")?;
        let remaining = options.steps.map_or(u64::MAX, |end| {
            end - game.session.scheduler.step_state().control_step
        });
        let mut advanced = false;
        for _ in 0..pending_due.min(remaining) {
            if game.session.result().is_some() {
                break;
            }
            apply_replay_boundary(&mut game.session, &mut replay);
            panel::advance_input(&mut game.session, &mut game.view, options.replay.is_some())?;
            game.session.tick()?;
            pending_due -= 1;
            advanced = true;
            if let Some(c) = &mut capture {
                c.snapshot(&game.session)?;
                if game.session.result().is_some()
                    || matches!(
                        game.session.scheduler.step_state().control_step,
                        16 | 17 | 34 | 35 | 54 | 60 | 100 | 170 | 340
                    )
                    || game
                        .session
                        .events
                        .iter()
                        .any(|event| event.starts_with("Input:"))
                    || basher_capture(&game.session)
                    || bomber_capture(&game.session)
                    || climber_capture(&game.session)
                    || floater_capture(&game.session)
                    || exit_capture(&game.session)
                    || hazard_capture(&game.session)
                {
                    break;
                }
            }
        }
        let step = game.session.scheduler.step_state().control_step;
        let finished = options.steps.is_some_and(|end| {
            if game.session.result().is_some() {
                options.result_actions.is_empty()
            } else {
                step >= end
            }
        });
        let milestone = game.view.panel.capture_pending
            || game.session.result().is_some()
            || matches!(step, 0 | 16 | 17 | 34 | 35 | 54 | 60 | 100 | 170 | 340)
            || basher_capture(&game.session)
            || bomber_capture(&game.session)
            || climber_capture(&game.session)
            || floater_capture(&game.session)
            || exit_capture(&game.session)
            || hazard_capture(&game.session)
            || game.session.actors.iter().any(|a| {
                a.state().action == lemmings_core::MotionAction::Building
                    && matches!(a.state().frame, 0 | 9)
            })
            || finished
            || game.session.actors.iter().any(|a| {
                a.state().action == lemmings_core::MotionAction::Digging
                    && matches!(a.state().frame, 0 | 8)
            })
            || game
                .session
                .events
                .iter()
                .any(|event| event.starts_with("Input:"));
        let take_capture = capture.is_some() && milestone && last_captured != Some(step);
        let render = initial
            || (game.session.result().is_some() && !result_rendered)
            || take_capture
            || finished
            || if options.fast {
                advanced && step % options.render_every == 0
            } else {
                last_render.elapsed() >= Duration::from_millis(16 * options.render_every)
            };
        if render {
            let frame = game.view.frame(&game.assets, &game.session)?;
            texture.update(None, &frame, WIDTH * 3)?;
            canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
            canvas.clear();
            canvas.copy(&texture, None, None)?;
            if take_capture {
                // Read the full output, including the letterbox after a WM
                // resize. SDL's implicit read rectangle follows the viewport.
                let size = canvas.output_size()?;
                canvas.set_logical_size(0, 0)?;
                let pixels = canvas.read_pixels(None, PixelFormatEnum::RGB24);
                canvas.set_logical_size(WIDTH as u32, HEIGHT as u32)?;
                capture
                    .as_mut()
                    .unwrap()
                    .frame(step, &frame, &pixels?, size)?;
                last_captured = Some(step);
                game.view.panel.capture_pending = false;
            }
            canvas.present();
            result_rendered = game.session.result().is_some();
            rendered += 1;
            last_render = Instant::now();
        }
        initial = false;
        if finished {
            break;
        }
        if !options.fast {
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    if !options.result_actions.is_empty() {
        return Err("run ended before all result actions were applied".into());
    }
    if replay.peek().is_some() && game.session.result().is_none() {
        return Err("run ended before all replay commands were applied".into());
    }
    let summary = run_summary(
        &game,
        rendered,
        observed_events,
        (video.current_video_driver(), canvas.info().name),
        (options.fast, options.synthetic, pacing),
    );
    if let Some(c) = &mut capture {
        c.finish(&game.session, &summary)?;
    }
    println!("{summary}");
    Ok(())
}
fn main() -> Result<(), Box<dyn Error>> {
    match Options::parse(std::env::args().skip(1))? {
        Some(options) => run(options),
        None => {
            println!("{USAGE}");
            Ok(())
        }
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn completed_session_does_not_consume_next_replay_boundary() {
        use lemmings_core::{InputCommand, StartupCommand, TimedInput};
        let a = super::Assets::synthetic_completion("zero");
        let mut s =
            super::Session::new(a.config, a.entrances, a.width, a.height, a.terrain).unwrap();
        s.configure_completion(a.required.unwrap()).unwrap();
        s.tick().unwrap();
        assert!(s.result().is_some());
        let mut replay = vec![TimedInput {
            control_step: 1,
            sequence: 0,
            command: InputCommand::Custom(StartupCommand::Nuke),
        }]
        .into_iter()
        .peekable();
        super::apply_replay_boundary(&mut s, &mut replay);
        assert!(s.inputs.is_empty());
        assert!(replay.peek().is_some());
    }

    use super::*;
    #[test]
    fn invalid_cli_combinations_and_missing_values_are_rejected() {
        for args in [
            vec!["--unknown"],
            vec!["--result-actions"],
            vec!["--result-actions", "restart"],
            vec!["--steps", "10", "--result-actions", "invalid"],
            vec!["--fast"],
            vec!["--steps", "0"],
            vec!["--render-every", "0"],
            vec!["--capture"],
            vec!["--pacing"],
            vec!["--pacing", "unknown"],
            vec!["--replay", "a", "--demo-input"],
            vec!["--assets", "a", "--synthetic"],
        ] {
            assert!(Options::parse(args.into_iter().map(str::to_owned)).is_err());
        }
        assert!(
            Options::parse(["--help".into()].into_iter())
                .unwrap()
                .is_none()
        );
    }
}
