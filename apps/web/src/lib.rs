//! Owned JS/WASM boundary; game rules and CPU rendering remain in shared crates.
use lemmings_core::{AssignmentSkill, InputCommand, StartupCommand};
use lemmings_game::{Assets, LevelRun, pacing::Pacing};
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
        let assets = match scene {
            "builder" => Assets::synthetic_builder(),
            "basher" => Assets::synthetic_basher(),
            "climber" => Assets::synthetic_climber(),
            "floater" => Assets::synthetic_floater(),
            "win" | "loss" | "mixed" | "clock" | "zero" | "particles" => {
                Assets::synthetic_completion(scene)
            }
            _ => return Err("unknown synthetic scene".into()),
        };
        Ok(Self {
            run: LevelRun::new(assets, 1, None, pacing).map_err(|e| e.to_string())?,
        })
    }

    /// Construct the replacement completely before discarding the current game.
    pub fn reset(&mut self, scene: &str, pacing: &str) -> Result<(), String> {
        *self = Self::new(scene, pacing)?;
        Ok(())
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
            self.run.session.tick().map_err(|e| e.to_string())?;
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

    /// Versioned diagnostic JSON; includes every actor field and queued input.
    /// Terrain, brick and raw object-map bytes are exposed separately as copies.
    pub fn state(&self) -> String {
        let s = &self.run.session;
        // Debug values below contain only fixed ASCII field/enum names, numbers,
        // booleans and punctuation, so Rust's quoted String output is valid JSON.
        let actors = format!(
            "{:?}",
            s.actors.iter().map(|a| a.state()).collect::<Vec<_>>()
        );
        let inputs = format!("{:?}", s.inputs);
        format!(
            "{{\"version\":1,\"session\":{},\"digger\":{},\"builder\":{},\"blocker\":{},\"bomber\":{},\"climber\":{},\"floater\":{},\"exit\":{},\"hazards\":{},\"completion\":{},\"remaining_bashers\":{},\"remaining_miners\":{},\"actor_details\":{actors:?},\"inputs\":{inputs:?}}}",
            s.snapshot(),
            s.digger_snapshot(),
            s.builder_snapshot(),
            s.blocker_snapshot(),
            s.bomber_snapshot(),
            s.climber_snapshot(),
            s.floater_snapshot(),
            s.exit_snapshot(),
            s.hazards_snapshot(),
            s.completion_snapshot(),
            s.remaining_bashers,
            s.remaining_miners,
        )
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
