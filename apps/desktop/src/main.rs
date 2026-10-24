#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod app_icon;
mod assets;
mod audio;
mod frame_clock;
mod frame_feedback;
mod hd;
mod input;
mod lifecycle;
mod local_music;
mod music_dialog;
mod pacing;
mod panel;
mod performance;
mod pointer;
mod render;
mod replay_capture;
mod session;
use assets::Assets;
use lemmings_core::{Difficulty, InputCommand, LevelId, StartupCommand};
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

const USAGE: &str = "Usage: lemmings-desktop [--hd | --original] [--hd-pack FILE_OR_DIRECTORY] [--fullscreen] [--assets DIR [--difficulty fun|tricky|taxing|mayhem] --level NUMBER | --synthetic | --synthetic-basher | --synthetic-builder | --synthetic-blocker | --synthetic-bomber | --synthetic-climber | --synthetic-floater | --synthetic-nuke | --synthetic-exit | --synthetic-exit-drop | --synthetic-water | --synthetic-fire | --synthetic-trap | --synthetic-trap-slow | --synthetic-trap-continuous | --synthetic-water-drop | --synthetic-fire-drop | --synthetic-water-late | --synthetic-fire-late | --synthetic-outcome win|loss|clock|particles|zero|mixed] [--performance NEW_JSON --performance-seconds 60] [--steps N] [--fast] [--render-every N] [--pacing dos|reference] [--capture NEW_DIR] [--replay FILE | --demo-input] [--capture-stream --elapsed-replay FILE] [--result-actions restart,continue]\nF3: FPS on/off; F2: copy performance feedback; F8: original/HD; F11: fullscreen; --hd starts HD; --original starts original presentation. S: sound on/off; PageUp/PageDown: volume; F4/Ctrl+O/Cmd+O: choose MP3; drop MP3 or --bgm FILE: local music; F6: original/local/off; F7: retry; F5: clear local music; [ ]: music volume; , .: effects volume; --mute/--volume 0..100; --fast disables devices. F/C/E/K/U/M/B/D: select Floater/Climber/Bomber/Blocker/Builder/Miner/Basher/Digger; left click: assign selected skill; N: nuke; Space: pause/resume; Up/Down: release rate; Left/Right/Home/End: camera; Esc: quit. On results: Enter/left click: retry or next; R: restart; Esc/right click: quit.\nReplay: CONTROL_STEP SEQUENCE Pause|Resume|Nuke|SetReleaseRate RATE|AssignDigger ACTOR_ID|AssignBasher ACTOR_ID|AssignMiner ACTOR_ID|AssignBuilder ACTOR_ID|AssignBlocker ACTOR_ID|AssignBomber ACTOR_ID|AssignClimber ACTOR_ID|AssignFloater ACTOR_ID.";
struct Options {
    hd: bool,
    fullscreen: bool,
    hd_pack: Option<PathBuf>,
    performance: Option<PathBuf>,
    performance_seconds: f64,
    assets: PathBuf,
    mute: bool,
    volume: u8,
    bgm: Option<PathBuf>,
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
    difficulty: Difficulty,
    steps: Option<u64>,
    fast: bool,
    render_every: u64,
    capture: Option<PathBuf>,
    capture_stream: bool,
    elapsed_replay: Option<PathBuf>,
    replay: Option<PathBuf>,
    demo_input: bool,
    pacing: Option<Pacing>,
}
impl Options {
    fn parse(args: impl Iterator<Item = String>) -> Result<Option<Self>, Box<dyn Error>> {
        let mut args = args.peekable();
        let mut o = Self {
            hd: false,
            fullscreen: false,
            hd_pack: None,
            performance: None,
            performance_seconds: 60.0,
            assets: if cfg!(windows) {
                assets::bundled_directory(&std::env::current_exe()?)
            } else if cfg!(target_os = "macos") {
                assets::macos_bundle_directory(&std::env::current_exe()?)
                    .unwrap_or_else(|| "assets/local/dos-original".into())
            } else {
                assets::linux_directory(&std::env::current_exe()?)
            },
            mute: false,
            volume: 40,
            bgm: None,
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
            difficulty: Difficulty::Fun,
            steps: None,
            fast: false,
            render_every: 1,
            capture: None,
            capture_stream: false,
            elapsed_replay: None,
            replay: None,
            demo_input: false,
            pacing: None,
        };
        let mut specified_assets = false;
        let mut original = false;
        while let Some(flag) = args.next() {
            match flag.as_str() {
                "--hd" => o.hd = true,
                "--original" => original = true,
                "--fullscreen" => o.fullscreen = true,
                "--hd-pack" => {
                    o.hd_pack = Some(args.next().ok_or("--hd-pack needs a path")?.into())
                }
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
                "--difficulty" => {
                    o.difficulty = args.next().ok_or("--difficulty requires a name")?.parse()?;
                }
                "--level" => {
                    o.level = args
                        .next()
                        .ok_or("--level requires number 1..30")?
                        .parse()?
                }
                "--performance" => {
                    o.performance = Some(
                        args.next()
                            .ok_or("--performance requires a new JSON file")?
                            .into(),
                    )
                }
                "--performance-seconds" => {
                    o.performance_seconds = args
                        .next()
                        .ok_or("--performance-seconds requires duration")?
                        .parse()?
                }
                "--bgm" => o.bgm = Some(args.next().ok_or("--bgm requires an MP3 path")?.into()),
                "--mute" => o.mute = true,
                "--volume" => {
                    o.volume = args.next().ok_or("--volume requires 0..100")?.parse()?;
                    if o.volume > 100 {
                        return Err("--volume requires 0..100".into());
                    }
                }
                "--fast" => o.fast = true,
                "--capture-stream" => o.capture_stream = true,
                "--elapsed-replay" => {
                    o.elapsed_replay = Some(
                        args.next()
                            .ok_or("--elapsed-replay requires a file")?
                            .into(),
                    )
                }
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
        if original && (o.hd || o.hd_pack.is_some()) {
            return Err("choose --original or --hd/--hd-pack".into());
        }
        if !original
            && !o.synthetic
            && o.performance.is_none()
            && let Some(directory) = assets::packaged_hd_directory(&std::env::current_exe()?)?
        {
            o.hd = true;
            if o.hd_pack.is_none() {
                o.hd_pack = Some(directory);
            }
        }
        if o.performance.is_some()
            && (o.hd
                || o.hd_pack.is_some()
                || o.fast
                || o.mute
                || o.capture.is_some()
                || o.steps.is_some()
                || !o.performance_seconds.is_finite()
                || !(0.1..=600.0).contains(&o.performance_seconds))
        {
            return Err("performance requires visible/audio-on real-time execution, no capture/steps, 0.1..600 seconds".into());
        }
        if o.hd && o.capture_stream {
            return Err(
                "HD preview supports --capture; --capture-stream is the original RGB contract"
                    .into(),
            );
        }
        if o.synthetic && specified_assets {
            return Err("choose --assets or --synthetic".into());
        }
        if !(1..=30).contains(&o.level)
            || (o.synthetic && (o.level != 1 || o.difficulty != Difficulty::Fun))
        {
            return Err("--level: 1..30; difficulty selection requires original assets".into());
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
        if (o.capture_stream || o.elapsed_replay.is_some()) && (!o.fast || o.capture.is_none()) {
            return Err(
                "capture stream/elapsed replay require --fast, --steps and --capture".into(),
            );
        }
        if (o.capture_stream || o.elapsed_replay.is_some())
            && (!o.result_actions.is_empty() || o.demo_input)
        {
            return Err(
                "capture stream/elapsed replay cannot combine with result-actions/demo-input"
                    .into(),
            );
        }
        Ok(Some(o))
    }
}
struct Capture {
    root: PathBuf,
    full: BufWriter<File>,
    frames: Option<BufWriter<std::io::Stdout>>,
    probes: u64,
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
    pointer_trace: BufWriter<File>,
    previous_object_map: Vec<u8>,
    previous_terrain: Vec<u8>,
}
fn write_new(path: &Path, data: &[u8]) -> Result<(), Box<dyn Error>> {
    let mut f = OpenOptions::new().write(true).create_new(true).open(path)?;
    f.write_all(data)?;
    Ok(())
}
impl Capture {
    fn new(root: &Path, synthetic: bool, stream: bool) -> Result<Self, Box<dyn Error>> {
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
            full: BufWriter::new(
                OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(root.join("full.jsonl"))?,
            ),
            frames: stream.then(|| BufWriter::new(std::io::stdout())),
            probes: 0,
            trace,
            previous_terrain: Vec::new(),
            previous_object_map: Vec::new(),
            pointer_trace: BufWriter::new(
                OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(root.join("pointer.jsonl"))?,
            ),
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
    fn pointer(&mut self, s: &Session, v: &render::View) -> Result<(), Box<dyn Error>> {
        let opt = |id: Option<u32>| id.map_or_else(|| "null".into(), |id| id.to_string());
        let point = v
            .panel
            .pointer
            .map_or_else(|| "null".into(), |(x, y)| format!("[{x},{y}]"));
        writeln!(
            self.pointer_trace,
            "{{\"control_step\":{},\"point\":{point},\"right\":{},\"count\":{},\"primary\":{},\"secondary\":{},\"camera\":{},\"caption\":{:?}}}",
            s.scheduler.step_state().control_step,
            v.right_mouse,
            v.selection.hits.count,
            opt(v.selection.hits.primary),
            opt(v.selection.hits.secondary),
            v.camera,
            crate::pointer::caption(s, v)
        )?;
        self.pointer_trace.flush()?;
        Ok(())
    }
    fn snapshot(&mut self, s: &Session) -> Result<(), Box<dyn Error>> {
        writeln!(self.full, "{}", lemmings_game::diagnostics::state(s))?;
        self.probes += 1;
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
        if let Some(stream) = &mut self.frames {
            replay_capture::verify_readback(size, pixels, logical)?;
            replay_capture::write_frame(stream, self.probes - 1, logical)?;
            return Ok(());
        }
        write_new(&self.root.join(format!("frame-{step:06}.rgb")), logical)?;
        let mut ppm = format!("P6\n{} {}\n255\n", size.0, size.1).into_bytes();
        ppm.extend_from_slice(pixels);
        write_new(&self.root.join(format!("frame-{step:06}.ppm")), &ppm)
    }
    fn finish(&mut self, s: &Session, summary: &str) -> Result<(), Box<dyn Error>> {
        self.full.flush()?;
        if let Some(stream) = &mut self.frames {
            replay_capture::write_frame(stream, u64::MAX, &[])?;
            stream.flush()?;
        }
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
                InputCommand::Custom(StartupCommand::AssignPointer(r)) => {
                    format!(
                        "AssignPointer {:?} {} {} {}",
                        r.skill,
                        r.primary,
                        r.secondary
                            .map(|id| id.to_string())
                            .unwrap_or_else(|| "-".into()),
                        r.right_click
                    )
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
    level: LevelId,
) -> Result<(), Box<dyn Error>> {
    c.snapshot(session)?;
    write_new(
        &c.root.join("asset-info.json"),
        lemmings_game::diagnostics::asset_info(assets).as_bytes(),
    )?;
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
        level.number()
    );
    let description = format!(
        "{},\"difficulty\":{:?},\"required\":{}}}",
        description.trim_end_matches('}'),
        level.difficulty().name(),
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
fn load_assets(
    options: &Options,
    level: LevelId,
) -> Result<(Assets, Option<LevelId>), Box<dyn Error>> {
    let assets = if let Some(kind) = &options.outcome_scene {
        Assets::synthetic_completion(if kind == "win" && level.number() == 2 {
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
    } else if level.difficulty() == Difficulty::Fun {
        if level.number() == 1 {
            assets::load(&options.assets)?
        } else {
            assets::load_level(&options.assets, level.number())?
        }
    } else {
        assets::load_id(&options.assets, level)?
    };
    let next = if options.synthetic {
        level
            .next()
            .filter(|_| options.outcome_scene.as_deref() == Some("win") && level.number() == 1)
    } else {
        level.next()
    };
    Ok((assets, next))
}
fn run(mut options: Options) -> Result<(), Box<dyn Error>> {
    let level = LevelId::new(options.difficulty, options.level)?;
    let (assets, next) = load_assets(&options, level)?;
    let replay = if let Some(path) = &options.replay {
        input::parse_replay(&fs::read_to_string(path)?, options.steps)?
    } else {
        Vec::new()
    };
    let mut replay = replay.into_iter().peekable();
    let elapsed = options
        .elapsed_replay
        .as_ref()
        .map(|p| replay_capture::elapsed(&fs::read_to_string(p)?))
        .transpose()?;
    let mut elapsed_chunks = elapsed.map(|v| v.into_iter());
    let pacing = options.pacing.unwrap_or(if options.synthetic {
        Pacing::Reference
    } else {
        Pacing::Dos
    });
    let mut game = LevelRun::new_for_level(assets, level, next, pacing)?;
    game.view.replaying = options.replay.is_some();
    let mut capture = options
        .capture
        .as_ref()
        .map(|p| Capture::new(p, options.synthetic, options.capture_stream))
        .transpose()?;
    if let Some(c) = &mut capture {
        initialize_capture(c, &game.assets, &game.session, level)?;
        c.pointer(&game.session, &game.view)?;
    }
    let sdl = sdl2::init()?;
    let video = sdl.video()?;
    let mut audio = audio::AudioService::new(&sdl, &options);
    let mut audio_visible = true;
    sdl2::hint::set("SDL_RENDER_SCALE_QUALITY", "0");
    let mut window = video
        .window(
            &format!("Oh My Lemmings - {}", game.assets.title),
            WIDTH as u32 * 3,
            HEIGHT as u32 * 3,
        )
        .position_centered()
        .resizable()
        .build()?;
    app_icon::apply(&mut window)?;
    let mut canvas = if options.hd || options.hd_pack.is_some() {
        window.into_canvas().build()?
    } else {
        window.into_canvas().software().build()?
    };
    if options.fullscreen {
        canvas
            .window_mut()
            .set_fullscreen(sdl2::video::FullscreenType::Desktop)?;
    }
    canvas.set_logical_size(WIDTH as u32, HEIGHT as u32)?;
    canvas.set_integer_scale(true)?;
    let creator = canvas.texture_creator();
    let mut texture =
        creator.create_texture_streaming(PixelFormatEnum::RGB24, WIDTH as u32, HEIGHT as u32)?;
    let mut feedback = frame_feedback::Hud::default();
    let feedback_allowed = !options.fast && options.performance.is_none() && capture.is_none();
    let mut feedback_texture =
        creator.create_texture_streaming(PixelFormatEnum::RGBA32, 160, 28)?;
    feedback_texture.set_blend_mode(sdl2::render::BlendMode::Blend);
    let hd_path = options.hd_pack.clone().unwrap_or_else(|| {
        let bundled = assets::presentation_root(&std::env::current_exe().unwrap_or_default());
        if bundled.join("HD-PRESENTATION").exists() {
            return bundled.join("hd-assets");
        }
        let directory = PathBuf::from("assets/local/hd-runtime/atlas");
        if directory.join("catalog.hdi").is_file() {
            directory
        } else {
            "assets/local/hd-runtime/assets.hdp".into()
        }
    });
    let mut hd_host = if options.hd || options.hd_pack.is_some() {
        Some(hd::Host::new(
            &creator,
            &hd::read_pack(&hd_path, &game.assets)?,
        )?)
    } else {
        None
    };
    let mut hd_active = options.hd;
    eprintln!(
        "LEMMINGS_PRESENTATION mode={} assets={}",
        if hd_active { "hd" } else { "original" },
        hd_path.display()
    );
    if hd_active {
        eprintln!(
            "LEMMINGS_HD {}",
            hd_host.as_ref().ok_or("missing HD host")?.renderer.info()
        );
        canvas.set_integer_scale(false)?;
        canvas.set_logical_size(1920, 1080)?;
    }
    let default_cursor = sdl2::mouse::Cursor::from_system(sdl2::mouse::SystemCursor::Arrow).ok();
    let play_cursor = sdl2::mouse::Cursor::from_system(sdl2::mouse::SystemCursor::Crosshair).ok();
    let selected_cursor = sdl2::mouse::Cursor::from_system(sdl2::mouse::SystemCursor::Hand).ok();
    let events = sdl.event()?;
    let mut pump = sdl.event_pump()?;
    let mut upload_samples = Vec::new();
    if options.performance.is_some() {
        let frame = game.view.frame(&game.assets, &game.session)?;
        for _ in 0..33 {
            let now = Instant::now();
            for _ in 0..32 {
                texture.update(None, &frame, WIDTH * 3)?;
                canvas.clear();
                canvas.copy(&texture, None, None)?;
            }
            upload_samples.push(now.elapsed().as_nanos());
        }
    }
    let mut performance = options
        .performance
        .as_ref()
        .map(|p| performance::Recorder::new(p, options.performance_seconds, pacing))
        .transpose()?;
    if let Some(p) = &mut performance {
        p.upload_samples = upload_samples;
    }
    let mut previous = Instant::now();
    let mut frame_clock = frame_clock::FrameClock::new(previous, options.render_every);
    let mut rendered = 0u64;
    let mut observed_events = 0u64;
    let mut initial = true;
    let mut presentation_dirty = false;
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
        let mut dialog_opened = false;
        for event in pump.poll_iter() {
            observed_events += 1;
            // The modal dialog may have queued Escape/click/repeated keys.
            // Retain close/visibility events, discard stale game commands.
            if dialog_opened && !matches!(event, Event::Quit { .. } | Event::Window { .. }) {
                continue;
            }
            if let Event::Window { win_event, .. } = &event {
                use sdl2::event::WindowEvent;
                match win_event {
                    WindowEvent::Hidden | WindowEvent::Minimized => {
                        feedback.set_active(false);
                        audio_visible = false;
                        frame_clock.set_visible(false, Instant::now());
                    }
                    WindowEvent::Shown | WindowEvent::Restored => {
                        feedback.set_active(false);
                        audio_visible = true;
                        frame_clock.set_visible(true, Instant::now());
                    }
                    _ => {}
                }
            }
            if !options.fast && music_dialog::requested(&event) {
                feedback.set_active(false);
                game.view.panel.release();
                game.view.panel.pointer = None;
                game.view.right_mouse = false;
                game.view.selection = Default::default();
                audio.pause_for_dialog();
                match music_dialog::pick(canvas.window_mut(), audio.selected_music()) {
                    Ok(Some(path)) => audio.select_music(path, &sdl, &options),
                    Ok(None) => {} // Cancel preserves the current track/mode/volume.
                    Err(error) => {
                        eprintln!("MP3 chooser unavailable: {error}; drop a file or use --bgm");
                    }
                }
                previous = Instant::now();
                frame_clock.reset(previous);
                presentation_dirty = true;
                dialog_opened = true;
                continue;
            }
            if let Event::KeyDown {
                keycode: Some(key @ (Keycode::F2 | Keycode::F3)),
                keymod,
                repeat: false,
                ..
            } = &event
                && !keymod.intersects(
                    sdl2::keyboard::Mod::LCTRLMOD
                        | sdl2::keyboard::Mod::RCTRLMOD
                        | sdl2::keyboard::Mod::LALTMOD
                        | sdl2::keyboard::Mod::RALTMOD
                        | sdl2::keyboard::Mod::LGUIMOD
                        | sdl2::keyboard::Mod::RGUIMOD
                        | sdl2::keyboard::Mod::LSHIFTMOD
                        | sdl2::keyboard::Mod::RSHIFTMOD,
                )
            {
                if feedback_allowed {
                    if *key == Keycode::F3 {
                        feedback.toggle();
                    } else if feedback.enabled {
                        let report = feedback.feedback(
                            &format!("{} {}", game.level.difficulty().name(), game.level.number()),
                            hd_active,
                            canvas.output_size()?,
                            &format!(
                                "SDL {} / {}",
                                video.current_video_driver(),
                                canvas.info().name
                            ),
                        );
                        feedback.copied(video.clipboard().set_clipboard_text(&report).is_ok());
                    }
                    presentation_dirty = true;
                }
                continue;
            }
            if matches!(
                event,
                Event::KeyDown {
                    keycode: Some(Keycode::F8),
                    repeat: false,
                    ..
                }
            ) {
                if options.capture_stream || options.performance.is_some() {
                    continue;
                }
                if hd_host.is_none() {
                    match hd::read_pack(&hd_path, &game.assets)
                        .map_err(|e| e.to_string())
                        .and_then(|b| hd::Host::new(&creator, &b).map_err(|e| e.to_string()))
                    {
                        Ok(host) => hd_host = Some(host),
                        Err(error) => {
                            eprintln!(
                                "HD assets unavailable: {error}; prepare a pack with tools/hd_assets.py build and --hd-pack PATH"
                            );
                            continue;
                        }
                    }
                }
                hd_active = !hd_active;
                feedback.reset();
                game.view.panel.release();
                game.view.panel.pointer = None;
                game.view.right_mouse = false;
                game.view.selection = Default::default();
                canvas.set_integer_scale(!hd_active)?;
                let (w, h) = if hd_active {
                    (1920, 1080)
                } else {
                    (WIDTH as u32, HEIGHT as u32)
                };
                canvas.set_logical_size(w, h)?;
                presentation_dirty = true;
                continue;
            }
            if matches!(
                event,
                Event::KeyDown {
                    keycode: Some(Keycode::F11),
                    repeat: false,
                    ..
                }
            ) {
                let fullscreen = canvas.window().fullscreen_state();
                canvas.window_mut().set_fullscreen(
                    if fullscreen == sdl2::video::FullscreenType::Off {
                        sdl2::video::FullscreenType::Desktop
                    } else {
                        sdl2::video::FullscreenType::Off
                    },
                )?;
                feedback.reset();
                presentation_dirty = true;
                continue;
            }
            let event = if hd_active {
                match hd::event(event, &mut game) {
                    hd::Routed::Event(event) => event,
                    hd::Routed::Restart => {
                        action = Some(ResultAction::Restart);
                        continue;
                    }
                }
            } else {
                event
            };
            if audio.input(&event, &sdl, &options) {
                continue;
            }
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
            let replacement = if action == ResultAction::Restart && game.session.result().is_none()
            {
                game.restart(pacing, |level| load_assets(&options, level))
                    .map(Some)
            } else {
                game.prepare(action, pacing, |level| load_assets(&options, level))
            };
            match replacement {
                Ok(Some(next)) => {
                    let theme_changed = hd_host.is_some()
                        && hd_path.is_dir()
                        && lemmings_game::hd::catalog::level_scope(&game.assets)?
                            != lemmings_game::hd::catalog::level_scope(&next.assets)?;
                    let next_hd_host = if theme_changed && hd_active {
                        match hd::read_pack(&hd_path, &next.assets)
                            .and_then(|bytes| hd::Host::new(&creator, &bytes))
                        {
                            Ok(host) => Some(host),
                            Err(error) => {
                                if scripted {
                                    return Err(error);
                                }
                                eprintln!("HD theme unavailable; keeping current level: {error}");
                                continue 'running;
                            }
                        }
                    } else {
                        None
                    };
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
                            "{{\"attempt\":{attempt},\"level\":{},\"step\":{},\"action\":\"{action:?}\",\"target\":{},\"difficulty\":{:?},\"target_difficulty\":{:?}}}",
                            game.level.number(),
                            game.session.scheduler.step_state().control_step,
                            next.level.number(),
                            game.level.difficulty().name(),
                            next.level.difficulty().name()
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
                                options.capture_stream,
                            )
                        })
                        .transpose()?;
                    if let Some(host) = next_hd_host {
                        hd_host = Some(host);
                    } else if theme_changed {
                        // Original mode must not depend on optional HD files.
                        hd_host = None;
                    }
                    game = next;
                    if let Some(host) = &mut hd_host {
                        host.invalidate();
                    }
                    if let Some(c) = &mut capture {
                        initialize_capture(c, &game.assets, &game.session, game.level)?;
                        c.pointer(&game.session, &game.view)?;
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
                    frame_clock.reset(previous);
                    feedback.reset();
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
        } else if let Some(chunks) = &mut elapsed_chunks {
            game.session.due(Duration::from_micros(
                chunks
                    .next()
                    .ok_or("elapsed replay exhausted before final boundary")?,
            ))?
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
            let tick_begin = performance.as_ref().map(|_| Instant::now());
            apply_replay_boundary(&mut game.session, &mut replay);
            panel::advance_input(&mut game.session, &mut game.view, options.replay.is_some())?;
            pointer::refresh(&game.session, &mut game.view);
            game.tick()?;
            pointer::refresh(&game.session, &mut game.view);
            if let (Some(p), Some(begin)) = (&mut performance, tick_begin) {
                p.tick(
                    begin,
                    game.session.scheduler.step_state().control_step,
                    game.session
                        .actors
                        .iter()
                        .filter(|a| a.state().removed.is_none())
                        .count(),
                );
            }
            pending_due -= 1;
            advanced = true;
            if let Some(c) = &mut capture {
                c.snapshot(&game.session)?;
                c.pointer(&game.session, &game.view)?;
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
        let audio_begin = performance.as_ref().map(|_| Instant::now());
        feedback.set_active(
            audio_visible
                && !game.session.scheduler.step_state().paused
                && game.session.result().is_none(),
        );
        let audio_state = performance.as_ref().and_then(|_| audio.performance_state());
        audio.service(&mut game, audio_visible);
        if let (Some(p), Some(begin)) = (&mut performance, audio_begin) {
            p.audio(audio_state, begin.elapsed().as_nanos());
        }
        if !options.fast {
            let title = format!(
                "Oh My Lemmings - {} · F3 FPS · {}",
                game.assets.title,
                audio.label()
            );
            if canvas.window().title() != title {
                canvas.window_mut().set_title(&title)?;
            }
        }
        let step = game.session.scheduler.step_state().control_step;
        let finished = performance.as_ref().is_some_and(|p| p.done())
            || options.steps.is_some_and(|end| {
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
        let cursor = if game.session.result().is_some()
            || !game
                .view
                .panel
                .pointer
                .is_some_and(|(x, y)| (0..320).contains(&x) && (0..160).contains(&y))
        {
            &default_cursor
        } else if game.view.selection.hits.primary.is_some() {
            &selected_cursor
        } else {
            &play_cursor
        };
        if let Some(cursor) = cursor {
            cursor.set();
        }
        let take_capture = capture.is_some() && milestone && last_captured != Some(step);
        let render = initial
            || presentation_dirty
            || (game.session.result().is_some() && !result_rendered)
            || take_capture
            || finished
            || if options.fast {
                advanced && step % options.render_every == 0
            } else {
                frame_clock.due(Instant::now())
            };
        if render {
            let budget = if options.fast {
                None
            } else {
                frame_clock.begin(Instant::now())
            };
            let frame_begin = performance.as_ref().map(|_| Instant::now());
            let frame = game.view.frame(&game.assets, &game.session)?;
            let cpu_ns = frame_begin.map_or(0, |t| t.elapsed().as_nanos());
            let upload_begin = performance.as_ref().map(|_| Instant::now());
            texture.update(None, &frame, WIDTH * 3)?;
            canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
            canvas.clear();
            if hd_active {
                hd_host
                    .as_mut()
                    .ok_or("missing HD host")?
                    .draw(&mut canvas, &game)?;
            } else {
                canvas.copy(&texture, None, None)?;
            }
            if take_capture || options.capture_stream {
                // Read the full output, including the letterbox after a WM
                // resize. SDL's implicit read rectangle follows the viewport.
                let size = canvas.output_size()?;
                canvas.set_logical_size(0, 0)?;
                let pixels = canvas.read_pixels(None, PixelFormatEnum::RGB24);
                let (w, h) = if hd_active {
                    (1920, 1080)
                } else {
                    (WIDTH as u32, HEIGHT as u32)
                };
                canvas.set_logical_size(w, h)?;
                capture
                    .as_mut()
                    .unwrap()
                    .frame(step, &frame, &pixels?, size)?;
                last_captured = Some(step);
                game.view.panel.capture_pending = false;
            }
            let upload_ns = upload_begin.map_or(0, |t| t.elapsed().as_nanos());
            let present_begin = performance.as_ref().map(|_| Instant::now());
            feedback.draw(&mut canvas, &mut feedback_texture, hd_active)?;
            canvas.present();
            if feedback.enabled {
                feedback.frame(Instant::now());
            }
            let present_ns = present_begin.map_or(0, |t| t.elapsed().as_nanos());
            let skipped = if budget.is_some() {
                frame_clock.finish(Instant::now())
            } else {
                0
            };
            if let (Some(p), Some(begin)) = (&mut performance, frame_begin) {
                p.frame(begin, cpu_ns, upload_ns, present_ns, budget, skipped);
            }
            presentation_dirty = false;
            result_rendered = game.session.result().is_some();
            rendered += 1;
        }
        initial = false;
        if finished {
            break;
        }
        if !options.fast {
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    if options.capture_stream && game.session.result().is_some() {
        let before = lemmings_game::diagnostics::state(&game.session);
        for probe in game.session.scheduler.step_state().control_step + 1..=options.steps.unwrap() {
            game.session.tick()?;
            if lemmings_game::diagnostics::state(&game.session) != before {
                return Err("finished session changed during frozen probe".into());
            }
            if let Some(c) = &mut capture {
                c.snapshot(&game.session)?;
                if probe % options.render_every == 0 || probe == options.steps.unwrap() {
                    let frame = game.view.frame(&game.assets, &game.session)?;
                    texture.update(None, &frame, WIDTH * 3)?;
                    canvas.clear();
                    if hd_active {
                        hd_host
                            .as_mut()
                            .ok_or("missing HD host")?
                            .draw(&mut canvas, &game)?;
                    } else {
                        canvas.copy(&texture, None, None)?;
                    }
                    let size = canvas.output_size()?;
                    canvas.set_logical_size(0, 0)?;
                    let pixels = canvas.read_pixels(None, PixelFormatEnum::RGB24)?;
                    let (w, h) = if hd_active {
                        (1920, 1080)
                    } else {
                        (WIDTH as u32, HEIGHT as u32)
                    };
                    canvas.set_logical_size(w, h)?;
                    c.frame(probe, &frame, &pixels, size)?;
                    canvas.present();
                    rendered += 1;
                }
            }
        }
    }
    if !options.result_actions.is_empty() {
        return Err("run ended before all result actions were applied".into());
    }
    if performance.is_none() && replay.peek().is_some() && game.session.result().is_none() {
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
    if let Some(p) = &performance {
        p.finish(&game, video.current_video_driver(), canvas.info().name)?;
    }
    eprintln!(
        "LEMMINGS_AUDIO submitted={} nonzero={} active={} original={}",
        audio.submitted,
        audio.nonzero,
        audio.performance_state().is_some(),
        !options.synthetic
    );
    println!("{summary}");
    Ok(())
}
fn entry() -> Result<(), Box<dyn Error>> {
    if std::env::args().nth(1).as_deref() == Some("--local-music-probe") {
        if std::env::args().count() > 3 {
            return Err("local music probe accepts one fixture path".into());
        }
        println!(
            "{}",
            audio::local_probe(std::env::args().nth(2).map(PathBuf::from))?
        );
        return Ok(());
    }
    if std::env::args().skip(1).eq(["--audio-probe"]) {
        println!("{}", audio::probe()?);
        return Ok(());
    }
    match Options::parse(std::env::args().skip(1))? {
        Some(options) => run(options),
        None => {
            println!("{USAGE}");
            Ok(())
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let result = entry();
    #[cfg(any(windows, target_os = "macos"))]
    if let Err(error) = &result {
        #[cfg(windows)]
        let directory = std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join("OhMyLemmings");
        #[cfg(target_os = "macos")]
        let directory = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join("Library/Logs/OhMyLemmings");
        let log = directory.join("last-error.log");
        let hint = if cfg!(target_os = "macos") {
            "Extract the complete ZIP and keep OhMyLemmings.app intact."
        } else {
            "Extract the complete ZIP, including the data folder and SDL2.dll."
        };
        let message = format!(
            "Could not start Oh My Lemmings.\n\n{error}\n\n{hint}\nLog: {}",
            log.display()
        );
        let _ = fs::create_dir_all(&directory);
        let _ = fs::write(&log, &message);
        // Dummy video is an explicitly headless host, so never open a modal UI.
        if std::env::var("SDL_VIDEODRIVER").as_deref() != Ok("dummy") {
            let _ = sdl2::messagebox::show_simple_message_box(
                sdl2::messagebox::MessageBoxFlag::ERROR,
                "Oh My Lemmings",
                &message,
                None,
            );
        }
    }
    result
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
    fn explicit_original_mode_and_hd_options_cannot_conflict() {
        let options = Options::parse(["--original"].into_iter().map(str::to_owned))
            .unwrap()
            .unwrap();
        assert!(!options.hd);
        assert!(options.hd_pack.is_none());
        for arguments in [
            vec!["--original", "--hd"],
            vec!["--original", "--hd-pack", "pack"],
        ] {
            assert!(Options::parse(arguments.into_iter().map(str::to_owned)).is_err());
        }
    }
    #[test]
    fn cli_retains_fun_default_and_validates_rank_and_level_boundaries() {
        for rank in ["fun", "tricky", "taxing", "mayhem"] {
            for number in ["1", "30"] {
                let o = Options::parse(
                    ["--level", number, "--difficulty", rank]
                        .into_iter()
                        .map(str::to_owned),
                )
                .unwrap()
                .unwrap();
                assert_eq!(o.difficulty.name(), rank);
                assert_eq!(o.level.to_string(), number);
            }
        }
        assert_eq!(
            Options::parse(["--level", "2"].into_iter().map(str::to_owned))
                .unwrap()
                .unwrap()
                .difficulty,
            Difficulty::Fun
        );
        for args in [
            vec!["--difficulty"],
            vec!["--difficulty", "easy"],
            vec!["--difficulty", "tricky", "--level", "0"],
            vec!["--difficulty", "mayhem", "--level", "31"],
            vec!["--difficulty", "taxing", "--synthetic"],
        ] {
            assert!(Options::parse(args.into_iter().map(str::to_owned)).is_err());
        }
    }

    #[test]
    fn streamed_capture_and_virtual_elapsed_require_bounded_offline_capture() {
        for args in [
            vec!["--capture-stream"],
            vec!["--elapsed-replay", "time.txt"],
            vec!["--capture-stream", "--fast", "--steps", "10"],
            vec![
                "--capture-stream",
                "--fast",
                "--steps",
                "10",
                "--capture",
                "new",
                "--result-actions",
                "restart",
            ],
        ] {
            assert!(Options::parse(args.into_iter().map(str::to_owned)).is_err());
        }
        let args = [
            "--capture-stream",
            "--fast",
            "--steps",
            "10",
            "--capture",
            "new",
            "--elapsed-replay",
            "time.txt",
        ];
        let o = Options::parse(args.into_iter().map(str::to_owned))
            .unwrap()
            .unwrap();
        assert!(o.capture_stream && o.elapsed_replay.is_some());
    }

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

#[cfg(test)]
mod playthrough_tests;
