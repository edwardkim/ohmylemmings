//! Owned JS/WASM boundary; game rules and CPU rendering remain in shared crates.
use lemmings_core::{AssignmentSkill, Difficulty, InputCommand, LevelId, StartupCommand};
use lemmings_game::{Assets, LevelRun, lifecycle::ResultAction, pacing::Pacing};
use std::{collections::BTreeMap, error::Error, rc::Rc};
mod audio;
pub use audio::WebAudio;
mod bytes;
mod input;
pub use bytes::WebAssets;
use std::time::Duration;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// Maximum work in one synchronous call. Hosts retain any remaining frame debt.
pub const MAX_STEPS: u32 = 1000;
/// Integer microseconds per delivery, at most one hour; invalid time is atomic.
pub const MAX_ELAPSED_US: f64 = 3_600_000_000.0;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct WebGame {
    run: LevelRun,
    source: Source,
    pacing: Pacing,
}

#[derive(Clone)]
enum Source {
    Synthetic(String),
    Original(Rc<BTreeMap<String, Vec<u8>>>),
}
impl Source {
    fn load(&self, level: LevelId) -> Result<(Assets, Option<LevelId>), Box<dyn Error>> {
        let assets = match self {
            Self::Original(files) => {
                Assets::from_level_source(&mut |name: &str| Ok(files.get(name).cloned()), level)?
            }
            Self::Synthetic(scene) => match scene.as_str() {
                "builder" => Assets::synthetic_builder(),
                "basher" => Assets::synthetic_basher(),
                "climber" => Assets::synthetic_climber(),
                "floater" => Assets::synthetic_floater(),
                "win" if level.number() == 2 => Assets::synthetic_completion("loss"),
                "win" | "loss" | "mixed" | "clock" | "zero" | "particles" => {
                    Assets::synthetic_completion(scene)
                }
                _ => return Err("unknown synthetic scene".into()),
            },
        };
        let next = match self {
            Self::Original(_) => level.next(),
            Self::Synthetic(scene) => level
                .next()
                .filter(|_| scene == "win" && level.number() == 1),
        };
        Ok((assets, next))
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl WebGame {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    pub fn new(scene: &str, pacing: &str) -> Result<Self, String> {
        let pacing = match pacing {
            "dos" => Pacing::Dos,
            "reference" => Pacing::Reference,
            _ => return Err("unknown pacing; expected dos or reference".into()),
        };
        let source = Source::Synthetic(scene.into());
        let level = LevelId::new(Difficulty::Fun, 1).unwrap();
        let (assets, next) = source.load(level).map_err(|e| e.to_string())?;
        Ok(Self {
            run: LevelRun::new_for_level(assets, level, next, pacing).map_err(|e| e.to_string())?,
            source,
            pacing,
        })
    }

    /// Construct the replacement completely before discarding the current game.
    pub fn reset(&mut self, scene: &str, pacing: &str) -> Result<(), String> {
        let mut replacement = Self::new(scene, pacing)?;
        replacement
            .run
            .sound
            .fresh_generation_after(self.run.sound.generation())
            .map_err(str::to_string)?;
        *self = replacement;
        Ok(())
    }

    /// Parsing and construction finish before replacing any live state or clock.
    pub fn load_original(&mut self, files: &WebAssets, number: f64) -> Result<(), String> {
        self.load_original_rank(files, "fun", number)
    }

    pub fn load_original_rank(
        &mut self,
        files: &WebAssets,
        difficulty: &str,
        number: f64,
    ) -> Result<(), String> {
        let level = checked_level(difficulty, number)?;
        let source = Source::Original(Rc::clone(&files.files));
        let (assets, next) = source.load(level).map_err(|e| e.to_string())?;
        let mut run =
            LevelRun::new_for_level(assets, level, next, Pacing::Dos).map_err(|e| e.to_string())?;
        run.sound
            .fresh_generation_after(self.run.sound.generation())
            .map_err(str::to_string)?;
        *self = Self {
            run,
            source,
            pacing: Pacing::Dos,
        };
        Ok(())
    }
    pub fn start_level(&mut self, number: f64) -> Result<(), String> {
        self.start_level_rank("fun", number)
    }
    pub fn start_level_rank(&mut self, difficulty: &str, number: f64) -> Result<(), String> {
        let level = checked_level(difficulty, number)?;
        if !self.original() {
            return Err("select original files first".into());
        }
        self.replace_level(level)
    }
    pub fn restart_level(&mut self) -> Result<(), String> {
        self.replace_level(self.run.level)
    }
    fn replace_level(&mut self, level: LevelId) -> Result<(), String> {
        let (assets, next) = self.source.load(level).map_err(|e| e.to_string())?;
        let mut replacement =
            LevelRun::new_for_level(assets, level, next, self.pacing).map_err(|e| e.to_string())?;
        replacement
            .sound
            .continue_from(&self.run.sound, false)
            .map_err(|e| e.to_string())?;
        self.run = replacement;
        Ok(())
    }
    /// Result-only scene actions never enter the deterministic tick-input log.
    pub fn transition(&mut self, action: &str) -> Result<bool, String> {
        let action = match action {
            "restart" => ResultAction::Restart,
            "continue" => ResultAction::Continue,
            _ => return Err("unknown result action".into()),
        };
        match self
            .run
            .prepare(action, self.pacing, |level| self.source.load(level))
        {
            Ok(Some(run)) => {
                self.run = run;
                Ok(true)
            }
            Ok(None) => Ok(false),
            Err(error) => {
                self.run.view.result_error = true;
                Err(error.to_string())
            }
        }
    }
    /// Bounded commands collected at every logical boundary, even in step batches.
    /// A new generation requires discarding old PCM/device queues. No device I/O.
    pub fn drain_sound_events(&mut self) -> String {
        self.run.sound.drain_json()
    }
    pub fn sound_state(&self) -> String {
        self.run.sound.diagnostic_json()
    }
    pub fn level(&self) -> u8 {
        self.run.level.number()
    }
    pub fn difficulty(&self) -> String {
        self.run.level.difficulty().name().into()
    }
    pub fn original(&self) -> bool {
        matches!(self.source, Source::Original(_))
    }
    pub fn can_continue(&self) -> bool {
        self.run.target(ResultAction::Continue).is_some()
    }
    pub fn key(&mut self, code: &str, repeat: bool) -> Result<String, String> {
        input::key(&mut self.run, code, repeat)
    }
    #[allow(clippy::too_many_arguments)]
    pub fn pointer(
        &mut self,
        kind: &str,
        x: f64,
        y: f64,
        button: f64,
        buttons: f64,
        clicks: f64,
    ) -> Result<String, String> {
        input::event(&mut self.run, kind, x, y, button, buttons, clicks)
    }
    pub fn release_input(&mut self) {
        input::release(&mut self.run);
    }
    /// Presentation diagnostics are separate from version-1 simulation snapshots.
    pub fn view_state(&self) -> String {
        let v = &self.run.view;
        let next = self
            .run
            .next
            .map_or_else(|| "null".into(), |n| n.number().to_string());
        let next_difficulty = self
            .run
            .next
            .map_or_else(|| "null".into(), |n| format!("{:?}", n.difficulty().name()));
        let pointer = format!("{:?}", v.panel.pointer);
        let held = format!("{:?}", v.panel.held_rate);
        let selection = format!("{:?}", v.selection);
        format!(
            "{{\"level\":{},\"next\":{next},\"original\":{},\"camera\":{},\"skill\":\"{:?}\",\"right\":{},\"dragging\":{},\"error\":{},\"pointer\":{pointer:?},\"held\":{held:?},\"selection\":{selection:?},\"difficulty\":{:?},\"next_difficulty\":{next_difficulty}}}",
            self.run.level.number(),
            self.original(),
            v.camera,
            v.selected_skill,
            v.right_mouse,
            v.panel.dragging_map,
            v.result_error,
            self.run.level.difficulty().name()
        )
    }

    pub fn width(&self) -> u32 {
        lemmings_game::render::WIDTH as u32
    }
    pub fn height(&self) -> u32 {
        lemmings_game::render::HEIGHT as u32
    }

    /// Includes queued pause/resume commands at the current control boundary.
    pub fn paused(&self) -> bool {
        self.run.session.desired_paused()
    }

    /// Return due ticks without simulating. The host explicitly consumes them.
    pub fn due(&mut self, elapsed_us: f64) -> Result<u32, String> {
        if !elapsed_us.is_finite()
            || !(0.0..=MAX_ELAPSED_US).contains(&elapsed_us)
            || elapsed_us.fract() != 0.0
        {
            return Err("elapsed time must be integer microseconds in 0..=3600000000".into());
        }
        // Fixed supported rates return at most 61200 calls per admitted hour.
        self.run
            .session
            .due(Duration::from_micros(elapsed_us as u64))
            .map(|v| v as u32)
            .map_err(|e| e.to_string())
    }

    /// Explicitly execute a bounded batch, stopping at the first frozen result.
    pub fn step(&mut self, count: f64) -> Result<u32, String> {
        if !count.is_finite()
            || count.fract() != 0.0
            || !(0.0..=f64::from(MAX_STEPS)).contains(&count)
        {
            return Err("step batch must be an integer in 0..=1000".into());
        }
        let mut executed = 0;
        for _ in 0..count as u32 {
            if self.run.session.result().is_some() {
                break;
            }
            lemmings_game::panel::advance_input(&mut self.run.session, &mut self.run.view, false)
                .map_err(|e| e.to_string())?;
            lemmings_game::pointer::refresh(&self.run.session, &mut self.run.view);
            self.run.tick().map_err(|e| e.to_string())?;
            lemmings_game::pointer::refresh(&self.run.session, &mut self.run.view);
            if self.run.session.result().is_some() {
                self.run.view.panel.release();
            }
            executed += 1;
        }
        Ok(executed)
    }

    /// Queue at the current control boundary. IDs/rates are validated before cast.
    pub fn command(&mut self, name: &str, value: f64) -> Result<(), String> {
        if !value.is_finite() || value.fract() != 0.0 || !(0.0..=u32::MAX as f64).contains(&value) {
            return Err("command argument must be an integer in 0..=4294967295".into());
        }
        let value = value as u32;
        let command = match name {
            "Pause" | "Resume" | "Nuke" if value != 0 => {
                return Err("argument must be zero for Pause, Resume or Nuke".into());
            }
            "Pause" => InputCommand::Pause,
            "Resume" => InputCommand::Resume,
            "Nuke" => InputCommand::Custom(StartupCommand::Nuke),
            "SetReleaseRate" => {
                let initial = self.run.session.scheduler.config().initial_release_rate;
                if !(u32::from(initial)..=99).contains(&value) {
                    return Err("release rate is outside the level's initial rate..=99".into());
                }
                InputCommand::Custom(StartupCommand::SetReleaseRate(value as u8))
            }
            _ => {
                let skill = match name {
                    "AssignClimber" => AssignmentSkill::Climber,
                    "AssignFloater" => AssignmentSkill::Floater,
                    "AssignBomber" => AssignmentSkill::Bomber,
                    "AssignBlocker" => AssignmentSkill::Blocker,
                    "AssignBuilder" => AssignmentSkill::Builder,
                    "AssignBasher" => AssignmentSkill::Basher,
                    "AssignMiner" => AssignmentSkill::Miner,
                    "AssignDigger" => AssignmentSkill::Digger,
                    _ => return Err("unknown command".into()),
                };
                InputCommand::Custom(lemmings_game::pointer::command(skill, value))
            }
        };
        self.run.session.queue(command).map_err(|e| e.to_string())
    }

    /// Stable, ordered text accepted by the independent desktop replay adapter.
    pub fn input_log(&self) -> String {
        let mut text = String::new();
        for input in &self.run.session.inputs {
            let command = match input.command {
                InputCommand::Pause => "Pause".into(),
                InputCommand::Resume => "Resume".into(),
                InputCommand::Custom(command) => match command {
                    StartupCommand::Nuke => "Nuke".into(),
                    StartupCommand::SetReleaseRate(rate) => format!("SetReleaseRate {rate}"),
                    StartupCommand::AssignDigger(id) => format!("AssignDigger {id}"),
                    StartupCommand::AssignBasher(id) => format!("AssignBasher {id}"),
                    StartupCommand::AssignMiner(id) => format!("AssignMiner {id}"),
                    StartupCommand::AssignBuilder(id) => format!("AssignBuilder {id}"),
                    StartupCommand::AssignBlocker(id) => format!("AssignBlocker {id}"),
                    StartupCommand::AssignBomber(id) => format!("AssignBomber {id}"),
                    StartupCommand::AssignClimber(id) => format!("AssignClimber {id}"),
                    StartupCommand::AssignFloater(id) => format!("AssignFloater {id}"),
                    StartupCommand::AssignPointer(r) => format!(
                        "AssignPointer {:?} {} {} {}",
                        r.skill,
                        r.primary,
                        r.secondary.map_or_else(|| "-".into(), |id| id.to_string()),
                        r.right_click
                    ),
                },
            };
            text.push_str(&format!(
                "{} {} {command}\n",
                input.control_step, input.sequence
            ));
        }
        text
    }

    /// Versioned diagnostic JSON; includes every actor field and queued input.
    /// Terrain, brick and raw object-map bytes are exposed separately as copies.
    pub fn state(&self) -> String {
        lemmings_game::diagnostics::state(&self.run.session)
    }
    /// Immutable initial header, inventories, palette and material flags.
    pub fn asset_info(&self) -> String {
        lemmings_game::diagnostics::asset_info(&self.run.assets)
    }
    pub fn materials(&self) -> Vec<u8> {
        lemmings_game::diagnostics::materials(&self.run.assets)
    }

    pub fn terrain(&self) -> Vec<u8> {
        self.run.session.terrain().to_vec()
    }
    pub fn bricks(&self) -> Vec<u8> {
        self.run.session.brick_mask().to_vec()
    }
    pub fn objects(&self) -> Vec<u8> {
        self.run.session.object_map_data().to_vec()
    }
    /// Owned RGB bytes, row major, three channels per logical pixel.
    pub fn frame(&mut self) -> Result<Vec<u8>, String> {
        self.run
            .view
            .frame(&self.run.assets, &self.run.session)
            .map_err(|e| e.to_string())
    }
}

fn checked_level(difficulty: &str, number: f64) -> Result<LevelId, String> {
    if !number.is_finite() || number.fract() != 0.0 || !(1.0..=30.0).contains(&number) {
        return Err("level must be an integer in 1..=30".into());
    }
    LevelId::new(
        difficulty
            .parse::<Difficulty>()
            .map_err(|e| e.to_string())?,
        number as u8,
    )
    .map_err(|e| e.to_string())
}
