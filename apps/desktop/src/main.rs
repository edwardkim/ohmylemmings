mod assets;
mod input;
mod pacing;
mod render;
mod session;
use assets::Assets;
use lemmings_core::{InputCommand, StartupCommand};
use pacing::Pacing;
use render::{HEIGHT, View, WIDTH};
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

const USAGE: &str = "Usage: lemmings-desktop [--assets DIR | --synthetic] [--steps N] [--fast] [--render-every N] [--pacing dos|reference] [--capture NEW_DIR] [--replay FILE | --demo-input]\nSpace: pause/resume; Up/Down: release rate; Left/Right/Home/End: camera; Esc: quit.\nReplay files contain CONTROL_STEP SEQUENCE Pause|Resume|SetReleaseRate [RATE].";
struct Options {
    assets: PathBuf,
    synthetic: bool,
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
                "--synthetic" => o.synthetic = true,
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
        })
    }
    fn snapshot(&mut self, s: &Session) -> Result<(), Box<dyn Error>> {
        writeln!(self.trace, "{}", s.snapshot())?;
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
        let mut inputs = String::new();
        for i in &s.inputs {
            let command = match i.command {
                InputCommand::Pause => "Pause".to_owned(),
                InputCommand::Resume => "Resume".to_owned(),
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
fn run(options: Options) -> Result<(), Box<dyn Error>> {
    let assets = if options.synthetic {
        Assets::synthetic()
    } else {
        Assets::load(&options.assets)?
    };
    let replay = if let Some(path) = &options.replay {
        input::parse_replay(&fs::read_to_string(path)?, options.steps)?
    } else {
        Vec::new()
    };
    let mut replay = replay.into_iter().peekable();
    let mut session = Session::new(
        assets.config,
        assets.entrances.clone(),
        assets.width,
        assets.height,
        assets.terrain.clone(),
    )?;
    let pacing = options.pacing.unwrap_or(if options.synthetic {
        Pacing::Reference
    } else {
        Pacing::Dos
    });
    session.set_pacing(pacing);
    let mut view = View::new(assets.screen_x);
    view.replaying = options.replay.is_some();
    let mut capture = options
        .capture
        .as_ref()
        .map(|p| Capture::new(p, options.synthetic))
        .transpose()?;
    if let Some(c) = &mut capture {
        c.snapshot(&session)?;
    }
    let sdl = sdl2::init()?;
    let video = sdl.video()?;
    sdl2::hint::set("SDL_RENDER_SCALE_QUALITY", "0");
    let window = video
        .window(
            "Oh My Lemmings - Just dig!",
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
    'running: loop {
        let step = session.scheduler.step_state().control_step;
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
        for event in pump.poll_iter() {
            observed_events += 1;
            if !input::handle(
                event,
                &mut session,
                &mut view,
                assets.width,
                options.replay.is_some(),
            )? {
                break 'running;
            }
        }
        let now = Instant::now();
        let elapsed = now.duration_since(previous);
        previous = now;
        let due = if initial {
            0
        } else if options.fast {
            1
        } else {
            session.due(elapsed)?
        };
        pending_due = pending_due
            .checked_add(due)
            .ok_or("pending tick overflow")?;
        let remaining = options.steps.map_or(u64::MAX, |end| {
            end - session.scheduler.step_state().control_step
        });
        let mut advanced = false;
        for _ in 0..pending_due.min(remaining) {
            let step = session.scheduler.step_state().control_step;
            while replay
                .peek()
                .is_some_and(|input| input.control_step == step)
            {
                // Scripted gameplay uses its exact sequence; live game controls are disabled.
                session.inputs.push(replay.next().unwrap());
            }
            session.tick()?;
            pending_due -= 1;
            advanced = true;
            if let Some(c) = &mut capture {
                c.snapshot(&session)?;
                if matches!(
                    session.scheduler.step_state().control_step,
                    16 | 17 | 34 | 35 | 54 | 60 | 100 | 170 | 340
                ) || session
                    .events
                    .iter()
                    .any(|event| event.starts_with("Input:"))
                {
                    break;
                }
            }
        }
        let step = session.scheduler.step_state().control_step;
        let finished = options.steps.is_some_and(|end| step >= end);
        let milestone = matches!(step, 0 | 16 | 17 | 34 | 35 | 54 | 60 | 100 | 170 | 340)
            || finished
            || session
                .events
                .iter()
                .any(|event| event.starts_with("Input:"));
        let take_capture = capture.is_some() && milestone && last_captured != Some(step);
        let render = initial
            || take_capture
            || finished
            || if options.fast {
                advanced && step % options.render_every == 0
            } else {
                last_render.elapsed() >= Duration::from_millis(16 * options.render_every)
            };
        if render {
            let frame = view.frame(&assets, &session)?;
            texture.update(None, &frame, WIDTH * 3)?;
            canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
            canvas.clear();
            canvas.copy(&texture, None, None)?;
            if take_capture {
                let pixels = canvas.read_pixels(None, PixelFormatEnum::RGB24)?;
                capture
                    .as_mut()
                    .unwrap()
                    .frame(step, &frame, &pixels, canvas.output_size()?)?;
                last_captured = Some(step);
            }
            canvas.present();
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
    if replay.peek().is_some() {
        return Err("run ended before all replay commands were applied".into());
    }
    let state = session.scheduler.step_state();
    let runtime = sdl2::version::version();
    let summary = format!(
        "{{\"control_step\":{},\"iteration\":{},\"frames\":{rendered},\"input_count\":{},\"events_seen\":{observed_events},\"camera\":{},\"video_driver\":{:?},\"renderer\":{:?},\"sdl_version\":\"{}.{}.{}\",\"timing\":\"{}\",\"synthetic\":{},\"pacing\":{:?},\"rate_numerator\":{},\"rate_denominator\":{}}}",
        state.control_step,
        state.iteration,
        session.inputs.len(),
        view.camera,
        video.current_video_driver(),
        canvas.info().name,
        runtime.major,
        runtime.minor,
        runtime.patch,
        if options.fast {
            "fixed-step"
        } else {
            "realtime"
        },
        options.synthetic,
        pacing.label(),
        pacing.rate().0,
        pacing.rate().1
    );
    if let Some(c) = &mut capture {
        c.finish(&session, &summary)?;
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
    use super::*;
    #[test]
    fn invalid_cli_combinations_and_missing_values_are_rejected() {
        for args in [
            vec!["--unknown"],
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
