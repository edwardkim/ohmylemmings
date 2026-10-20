//! Browser events translated to the same shared panel, pointer and tick commands as SDL.
use lemmings_core::{InputCommand, StartupCommand};
use lemmings_game::{LevelRun, panel, pointer, render::SelectedSkill};

pub fn release(run: &mut LevelRun) {
    run.view.panel.release();
    run.view.panel.pointer = None;
    run.view.right_mouse = false;
    run.view.selection = Default::default();
}
fn camera(run: &mut LevelRun, code: &str) {
    match code {
        "ArrowLeft" => run.view.scroll(-16, run.assets.width),
        "ArrowRight" => run.view.scroll(16, run.assets.width),
        "Home" => run.view.scroll(i32::MIN, run.assets.width),
        "End" => run.view.scroll(i32::MAX, run.assets.width),
        _ => {}
    }
}
pub fn key(run: &mut LevelRun, code: &str, repeat: bool) -> Result<String, String> {
    camera(run, code);
    if code == "Escape" {
        release(run);
        return Ok("exit".into());
    }
    if run.session.result().is_some() {
        release(run);
        return Ok(match (code, repeat) {
            ("KeyR", false) => "restart",
            ("Enter", false) => "continue",
            _ => "none",
        }
        .into());
    }
    let s = &mut run.session;
    match code {
        "KeyB" => panel::select_skill(s, &mut run.view, SelectedSkill::Basher, !repeat),
        "KeyD" => panel::select_skill(s, &mut run.view, SelectedSkill::Digger, !repeat),
        "KeyM" => panel::select_skill(s, &mut run.view, SelectedSkill::Miner, !repeat),
        "KeyU" => panel::select_skill(s, &mut run.view, SelectedSkill::Builder, !repeat),
        "KeyF" => panel::select_skill(s, &mut run.view, SelectedSkill::Floater, !repeat),
        "KeyC" => panel::select_skill(s, &mut run.view, SelectedSkill::Climber, !repeat),
        "KeyE" => panel::select_skill(s, &mut run.view, SelectedSkill::Bomber, !repeat),
        "KeyK" => panel::select_skill(s, &mut run.view, SelectedSkill::Blocker, !repeat),
        "Space" if !repeat => {
            s.queue(if s.desired_paused() {
                InputCommand::Resume
            } else {
                InputCommand::Pause
            })
            .map_err(|e| e.to_string())?;
        }
        "KeyN" if !repeat && !s.desired_paused() => {
            s.queue(InputCommand::Custom(StartupCommand::Nuke))
                .map_err(|e| e.to_string())?;
        }
        "ArrowUp" | "ArrowDown" => {
            let rate = if code == "ArrowUp" {
                s.desired_rate().saturating_add(1).min(99)
            } else {
                s.desired_rate()
                    .saturating_sub(1)
                    .max(s.scheduler.config().initial_release_rate)
            };
            s.queue(InputCommand::Custom(StartupCommand::SetReleaseRate(rate)))
                .map_err(|e| e.to_string())?;
        }
        _ => {}
    }
    pointer::refresh(&run.session, &mut run.view);
    Ok("none".into())
}
fn integer(value: f64, low: f64, high: f64) -> bool {
    value.is_finite() && value.fract() == 0.0 && (low..=high).contains(&value)
}
// All numeric values are checked before any cast or input/view mutation.
#[allow(clippy::too_many_arguments)]
pub fn event(
    run: &mut LevelRun,
    kind: &str,
    x: f64,
    y: f64,
    button: f64,
    buttons: f64,
    clicks: f64,
) -> Result<String, String> {
    if !matches!(kind, "down" | "up" | "move")
        || !integer(x, i32::MIN as f64, i32::MAX as f64)
        || !integer(y, i32::MIN as f64, i32::MAX as f64)
        || !integer(button, 0.0, 2.0)
        || !integer(buttons, 0.0, 31.0)
        || !integer(clicks, 0.0, 255.0)
    {
        return Err("invalid pointer event".into());
    }
    let (x, y, button, buttons, clicks) = (
        x as i32,
        y as i32,
        button as u8,
        buttons as u8,
        clicks as u8,
    );
    if run.session.result().is_some() {
        release(run);
        return Ok(
            if kind == "down" && (0..320).contains(&x) && (0..200).contains(&y) {
                match button {
                    0 => "continue",
                    2 => "exit",
                    _ => "none",
                }
            } else {
                "none"
            }
            .into(),
        );
    }
    let s = &mut run.session;
    let v = &mut run.view;
    match (kind, button) {
        ("down", 2) | ("up", 2) => {
            v.panel.pointer = Some((x, y));
            v.right_mouse = kind == "down";
        }
        ("up", 0) => v.panel.release(),
        ("move", _) => {
            v.panel.pointer = Some((x, y));
            v.right_mouse = buttons & 2 != 0;
            if buttons & 1 == 0 {
                v.panel.release();
            }
            if v.panel.dragging_map
                && let Some(camera) = panel::minimap_camera(x, y, run.assets.width)
            {
                v.camera = camera;
            }
        }
        ("down", 0) if (0..320).contains(&x) && (160..200).contains(&y) => {
            v.panel.release();
            v.panel.pointer = Some((x, y));
            if let Some(camera) = panel::minimap_camera(x, y, run.assets.width) {
                v.camera = camera;
                v.panel.dragging_map = true;
            } else if let Some(button) = panel::hit(x, y) {
                use panel::Button;
                match button {
                    Button::Skill(skill) => panel::select_skill(s, v, skill, true),
                    Button::Slower | Button::Faster => {
                        v.panel.held_rate = Some(if button == Button::Faster { 1 } else { -1 });
                        panel::advance_input(s, v, false).map_err(|e| e.to_string())?;
                    }
                    Button::Pause => s
                        .queue(if s.desired_paused() {
                            InputCommand::Resume
                        } else {
                            InputCommand::Pause
                        })
                        .map_err(|e| e.to_string())?,
                    Button::Nuke
                        if clicks >= 2
                            && !s.desired_paused()
                            && !s.scheduler.nuke_state().is_nuked =>
                    {
                        s.queue(InputCommand::Custom(StartupCommand::Nuke))
                            .map_err(|e| e.to_string())?
                    }
                    _ => {}
                }
            }
        }
        ("down", 0) if (0..320).contains(&x) && (0..160).contains(&y) => {
            v.panel.pointer = Some((x, y));
            pointer::refresh(s, v);
            if let Some(request) = v.selection.assignment(pointer::skill(v.selected_skill)) {
                let command = if !request.right_click
                    && request.secondary.is_none_or(|id| id == request.primary)
                {
                    pointer::command(request.skill, request.primary)
                } else {
                    StartupCommand::AssignPointer(request)
                };
                s.queue(InputCommand::Custom(command))
                    .map_err(|e| e.to_string())?;
            }
        }
        _ => {}
    }
    pointer::refresh(s, v);
    Ok("none".into())
}
