//! HD coordinates resolve to existing input semantics. No ticks or I/O.
use super::{Control, Hit, Viewport};
use crate::{LevelRun, panel};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Legacy(i32, i32),
    Camera(i32),
    Restart,
    Outside,
}

pub fn map_point(x: f64, y: f64, width: usize, result: bool) -> Target {
    let Some(hit) = Viewport::fit(1920., 1080.).and_then(|v| v.hit(x, y, width)) else {
        return Target::Outside;
    };
    if hit == Hit::Control(Control::Restart) {
        return Target::Restart;
    }
    if result {
        return Target::Legacy(160, 100);
    }
    match hit {
        Hit::Game { x, y } => Target::Legacy(x, y),
        Hit::Minimap { camera } => Target::Camera(camera),
        Hit::Control(c) => {
            let index = match c {
                Control::Slower => 0,
                Control::Faster => 1,
                Control::Skill(s) => {
                    2 + panel::SKILLS
                        .iter()
                        .position(|&v| v == s)
                        .expect("known skill")
                }
                Control::Pause => 10,
                Control::Nuke => 11,
                _ => return Target::Outside,
            };
            Target::Legacy(index as i32 * 16 + 8, 180)
        }
    }
}

/// Handle map dragging at HD precision, then let each host reuse its original
/// pointer state machine. `None` asks the host to restart without a tick command.
pub fn route(
    run: &mut LevelRun,
    kind: &str,
    x: f64,
    y: f64,
    button: u8,
    buttons: u8,
) -> Option<(i32, i32)> {
    match map_point(x, y, run.assets.width, run.session.result().is_some()) {
        Target::Legacy(x, y) => Some((x, y)),
        Target::Restart if kind == "down" && button == 0 && !run.view.replaying => None,
        Target::Camera(camera) => {
            if kind == "down" && button == 0 {
                run.view.panel.release();
                run.view.panel.dragging_map = true;
                run.view.camera = camera;
            } else if kind == "move" && run.view.panel.dragging_map && buttons & 1 != 0 {
                run.view.camera = camera;
            }
            Some((-1, -1))
        }
        _ => Some((-1, -1)),
    }
}
