use super::SCALE;
use crate::{LevelRun, lifecycle::ResultAction, render::SelectedSkill};
use lemmings_core::{ActorState, LevelId, LevelResult, PointerHits, TrapStates, percentage};
use std::fmt;

/// Large maps are borrowed, not copied or resampled. `visible` preserves the
/// original explosion display delay; `collision` is the current physical mask.
#[derive(Debug)]
pub struct TerrainSnapshot<'a> {
    pub width: usize,
    pub height: usize,
    pub collision: &'a [u8],
    pub visible: &'a [u8],
    pub bricks: &'a [u8],
    pub steel: &'a [u8],
    pub one_way_left: &'a [u8],
    pub one_way_right: &'a [u8],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActorSnapshot {
    /// Includes removed actors: their explosion particle counters may still draw.
    pub state: ActorState,
    /// Unclipped foot in HD game coordinates; atlas bounds determine visibility.
    pub foot: (i64, i64),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HudSnapshot {
    pub original_total: u32,
    pub population: u32,
    pub released: u32,
    pub alive: u32,
    pub rescued: u32,
    pub required: Option<u32>,
    pub target_percent: Option<u32>,
    pub rescued_percent: u32,
    pub seconds_remaining: u32,
    pub release_rate: u8,
    pub minimum_release_rate: u8,
    pub paused: bool,
    pub nuked: bool,
    /// Same order as panel::SKILLS, including both permanent abilities.
    pub skills: [u16; 8],
    pub selected: SelectedSkill,
    pub hits: PointerHits,
    pub right_mouse: bool,
    pub dragging_map: bool,
    pub replaying: bool,
    pub result_error: bool,
    pub result: Option<LevelResult>,
    pub can_continue: bool,
}

/// Typed Rust contract v1. This is not the Debug-based diagnostic JSON or a new
/// JS wire format. Native and WASM Rust callers share these semantics.
#[derive(Debug)]
pub struct Snapshot<'a> {
    pub version: u32,
    pub level: LevelId,
    pub title: &'a str,
    pub camera: i32,
    pub control_step: u64,
    pub iteration: u64,
    pub terrain: TerrainSnapshot<'a>,
    pub actors: Vec<ActorSnapshot>,
    pub traps: &'a TrapStates,
    pub hud: HudSnapshot,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapshotError {
    WorldDimensions,
    AssetDimensions,
    MaskLength(&'static str),
    ActorCount,
}

impl fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WorldDimensions => f.write_str("invalid HD logical world dimensions"),
            Self::AssetDimensions => f.write_str("HD asset/session dimensions differ"),
            Self::MaskLength(name) => write!(f, "HD {name} mask length differs"),
            Self::ActorCount => f.write_str("HD actor count exceeds the logical population type"),
        }
    }
}
impl std::error::Error for SnapshotError {}

pub fn snapshot(run: &LevelRun) -> Result<Snapshot<'_>, SnapshotError> {
    let session = &run.session;
    let (width, height) = session.dimensions();
    let area = width
        .checked_mul(height)
        .filter(|&area| area > 0)
        .ok_or(SnapshotError::WorldDimensions)?;
    if (run.assets.width, run.assets.height) != (width, height) {
        return Err(SnapshotError::AssetDimensions);
    }
    let terrain = TerrainSnapshot {
        width,
        height,
        collision: session.terrain(),
        visible: session.render_terrain(),
        bricks: session.render_bricks(),
        steel: &run.assets.steel,
        one_way_left: &run.assets.one_way_left,
        one_way_right: &run.assets.one_way_right,
    };
    for (name, bytes) in [
        ("collision", terrain.collision),
        ("visible", terrain.visible),
        ("bricks", terrain.bricks),
        ("steel", terrain.steel),
        ("one_way_left", terrain.one_way_left),
        ("one_way_right", terrain.one_way_right),
    ] {
        if bytes.len() != area {
            return Err(SnapshotError::MaskLength(name));
        }
    }
    let camera = run
        .view
        .camera
        .clamp(0, width.saturating_sub(320).min(i32::MAX as usize) as i32);
    let actors: Vec<_> = session
        .actors
        .iter()
        .map(|actor| {
            let state = actor.state();
            ActorSnapshot {
                foot: (
                    (i64::from(state.x) - i64::from(camera)) * SCALE,
                    i64::from(state.y) * SCALE,
                ),
                state,
            }
        })
        .collect();
    let alive = u32::try_from(actors.iter().filter(|a| a.state.removed.is_none()).count())
        .map_err(|_| SnapshotError::ActorCount)?;
    let rescued = u32::try_from(session.saved()).map_err(|_| SnapshotError::ActorCount)?;
    let config = session.scheduler.config();
    let startup = session.scheduler.state();
    let step = session.scheduler.step_state();
    let nuke = session.scheduler.nuke_state();
    let rules = session.completion_rules();
    let result = session.result();
    Ok(Snapshot {
        version: 1,
        level: run.level,
        title: &run.assets.title,
        camera,
        control_step: step.control_step,
        iteration: step.iteration,
        terrain,
        actors,
        traps: session.traps(),
        hud: HudSnapshot {
            original_total: config.total_lemmings,
            population: nuke.max_lemmings,
            released: startup.released,
            alive,
            rescued,
            required: rules.map(|rules| rules.required()),
            target_percent: rules.map(|rules| rules.target_percent()),
            rescued_percent: result.map_or_else(
                || percentage(rescued, nuke.max_lemmings) as u32,
                |result| result.rescued_percent,
            ),
            seconds_remaining: startup.seconds_remaining,
            release_rate: session.desired_rate(),
            minimum_release_rate: config.initial_release_rate,
            paused: session.desired_paused(),
            nuked: nuke.is_nuked,
            skills: [
                session.remaining_climbers,
                session.remaining_floaters,
                session.remaining_bombers,
                session.remaining_blockers,
                session.remaining_builders,
                session.remaining_bashers,
                session.remaining_miners,
                session.remaining_diggers,
            ],
            selected: run.view.selected_skill,
            hits: run.view.selection.hits,
            right_mouse: run.view.right_mouse,
            dragging_map: run.view.panel.dragging_map,
            replaying: run.view.replaying,
            result_error: run.view.result_error,
            result,
            can_continue: run.target(ResultAction::Continue).is_some(),
        },
    })
}
