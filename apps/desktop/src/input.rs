use crate::{
    render::{SelectedSkill, View},
    session::Session,
};
use lemmings_core::{InputCommand, InputSchedule, StartupCommand, TimedInput};
use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Keycode,
    mouse::MouseButton,
};
use std::error::Error;

/// Return false on quit. Only gameplay commands enter the deterministic log.
pub fn handle(
    event: Event,
    session: &mut Session,
    view: &mut View,
    width: usize,
    replaying: bool,
) -> Result<bool, Box<dyn Error>> {
    match event {
        Event::Quit { .. }
        | Event::Window {
            win_event: WindowEvent::Close,
            ..
        } => return Ok(false),
        Event::MouseButtonDown {
            mouse_btn: MouseButton::Left,
            x,
            y,
            ..
        } if !replaying && (0..320).contains(&x) && (0..160).contains(&y) => {
            let wx = i64::from(x) + i64::from(view.camera);
            let target = session
                .actors
                .iter()
                .filter(|a| {
                    let a = a.state();
                    a.removed.is_none()
                        && (i64::from(a.x) - wx).abs() <= 6
                        && (i64::from(a.y) - i64::from(y) - 5).abs() <= 7
                })
                .min_by_key(|a| (i64::from(a.state().x) - wx).abs())
                .map(|a| a.state().id);
            if let Some(id) = target {
                session.queue(InputCommand::Custom(match view.selected_skill {
                    SelectedSkill::Digger => StartupCommand::AssignDigger(id),
                    SelectedSkill::Basher => StartupCommand::AssignBasher(id),
                }))?;
            }
        }
        Event::KeyDown {
            keycode: Some(key),
            repeat,
            ..
        } => match key {
            Keycode::Escape => return Ok(false),
            Keycode::B => view.selected_skill = SelectedSkill::Basher,
            Keycode::D => view.selected_skill = SelectedSkill::Digger,
            Keycode::Left => view.scroll(-16, width),
            Keycode::Right => view.scroll(16, width),
            Keycode::Home => view.scroll(i32::MIN, width),
            Keycode::End => view.scroll(i32::MAX, width),
            Keycode::Space if !repeat && !replaying => {
                let command = if session.desired_paused() {
                    InputCommand::Resume
                } else {
                    InputCommand::Pause
                };
                session.queue(command)?;
            }
            Keycode::Up | Keycode::Down if !replaying => {
                let old = session.desired_rate();
                let initial = session.scheduler.config().initial_release_rate;
                let rate = if key == Keycode::Up {
                    old.saturating_add(1).min(99)
                } else {
                    old.saturating_sub(1).max(initial)
                };
                session.queue(InputCommand::Custom(StartupCommand::SetReleaseRate(rate)))?;
            }
            _ => {}
        },
        _ => {}
    }
    Ok(true)
}
pub fn parse_replay(
    text: &str,
    steps: Option<u64>,
) -> Result<Vec<TimedInput<StartupCommand>>, Box<dyn Error>> {
    let mut result = Vec::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let words: Vec<_> = line.split_whitespace().collect();
        if words.len() < 3 {
            return Err(
                "replay line: CONTROL_STEP SEQUENCE Pause|Resume|SetReleaseRate [RATE]".into(),
            );
        }
        let control_step = words[0].parse()?;
        let sequence = words[1].parse()?;
        let command = match words[2..] {
            ["Pause"] => InputCommand::Pause,
            ["Resume"] => InputCommand::Resume,
            ["AssignDigger", id] => InputCommand::Custom(StartupCommand::AssignDigger(id.parse()?)),
            ["AssignBasher", id] => InputCommand::Custom(StartupCommand::AssignBasher(id.parse()?)),
            ["SetReleaseRate", rate] => {
                InputCommand::Custom(StartupCommand::SetReleaseRate(rate.parse()?))
            }
            _ => return Err("unknown replay command or argument".into()),
        };
        if steps.is_some_and(|end| control_step >= end) {
            return Err("replay input outside requested run".into());
        }
        result.push(TimedInput {
            control_step,
            sequence,
            command,
        });
    }
    InputSchedule::new(&result)?;
    Ok(result)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::Assets;
    use sdl2::keyboard::Mod;
    pub fn key(key: Keycode, repeat: bool) -> Event {
        Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(key),
            scancode: None,
            keymod: Mod::NOMOD,
            repeat,
        }
    }
    fn setup() -> (Session, View) {
        let a = Assets::synthetic();
        (
            Session::new(a.config, a.entrances, a.width, a.height, a.terrain).unwrap(),
            View::new(0),
        )
    }
    #[test]
    fn basher_selection_records_id_and_digger_selection_restores_default() {
        let (mut s, mut v) = setup();
        for _ in 0..80 {
            s.tick().unwrap();
        }
        let a = s.actors[0].state();
        let click = Event::MouseButtonDown {
            timestamp: 0,
            window_id: 0,
            which: 0,
            mouse_btn: MouseButton::Left,
            clicks: 1,
            x: a.x,
            y: a.y - 5,
        };
        handle(key(Keycode::B, false), &mut s, &mut v, 640, false).unwrap();
        handle(click.clone(), &mut s, &mut v, 640, false).unwrap();
        handle(key(Keycode::D, false), &mut s, &mut v, 640, false).unwrap();
        handle(click, &mut s, &mut v, 640, false).unwrap();
        assert_eq!(
            parse_replay("80 0 AssignBasher 0\n80 1 AssignDigger 0", Some(340)).unwrap(),
            s.inputs
        );
    }
    #[test]
    fn rapid_toggles_and_rates_preserve_event_order_and_clamp() {
        let (mut s, mut v) = setup();
        for e in [
            key(Keycode::Space, false),
            key(Keycode::Space, true),
            key(Keycode::Space, false),
            key(Keycode::Up, false),
            key(Keycode::Up, false),
        ] {
            assert!(handle(e, &mut s, &mut v, 640, false).unwrap());
        }
        s.tick().unwrap();
        assert!(!s.scheduler.step_state().paused);
        assert_eq!(s.scheduler.state().release_rate, 52);
        assert_eq!(s.inputs.len(), 4);
        for _ in 0..200 {
            handle(key(Keycode::Down, true), &mut s, &mut v, 640, false).unwrap();
        }
        assert_eq!(s.desired_rate(), 50);
    }
    #[test]
    fn replay_disables_live_game_commands_but_keeps_camera_and_quit() {
        let (mut s, mut v) = setup();
        for keycode in [Keycode::Space, Keycode::Up, Keycode::Right] {
            handle(key(keycode, false), &mut s, &mut v, 640, true).unwrap();
        }
        assert!(s.inputs.is_empty());
        assert_eq!(v.camera, 16);
        assert!(!handle(key(Keycode::Escape, false), &mut s, &mut v, 640, true).unwrap());
        assert!(!handle(Event::Quit { timestamp: 0 }, &mut s, &mut v, 640, false).unwrap());
    }
    #[test]
    fn left_click_records_stable_id_and_replay_blocks_live_assignments() {
        let (mut s, mut v) = setup();
        for _ in 0..80 {
            s.tick().unwrap();
        }
        let a = s.actors[0].state();
        let click = Event::MouseButtonDown {
            timestamp: 0,
            window_id: 0,
            which: 0,
            mouse_btn: MouseButton::Left,
            clicks: 1,
            x: a.x - v.camera,
            y: a.y - 5,
        };
        handle(click.clone(), &mut s, &mut v, 640, true).unwrap();
        assert!(s.inputs.is_empty());
        handle(click, &mut s, &mut v, 640, false).unwrap();
        assert_eq!(
            s.inputs[0].command,
            InputCommand::Custom(StartupCommand::AssignDigger(0))
        );
        assert_eq!(
            parse_replay("80 0 AssignDigger 0", Some(340)).unwrap(),
            s.inputs
        );
    }
    #[test]
    fn clicks_in_the_letterbox_or_above_the_playfield_cannot_assign() {
        for above in [false, true] {
            let (mut s, mut v) = setup();
            for _ in 0..80 {
                s.tick().unwrap();
            }
            let a = s.actors[0].state();
            let (x, y) = if above {
                s.actors[0] = lemmings_core::Actor::from_state(
                    lemmings_core::ActorState { x: 1, y: 0, ..a },
                    lemmings_core::MotionRules::default(),
                )
                .unwrap();
                (1, -5)
            } else {
                v.camera = a.x + 1;
                (-1, a.y - 5)
            };
            let click = Event::MouseButtonDown {
                timestamp: 0,
                window_id: 0,
                which: 0,
                mouse_btn: MouseButton::Left,
                clicks: 1,
                x,
                y,
            };
            handle(click, &mut s, &mut v, 640, false).unwrap();
            assert!(s.inputs.is_empty());
        }
    }
    #[test]
    fn malformed_duplicate_and_outside_replays_are_rejected() {
        assert_eq!(
            parse_replay("16 3 Pause\n34 0 Resume\n54 0 SetReleaseRate 99", Some(340))
                .unwrap()
                .len(),
            3
        );
        for text in [
            "0 0 Unknown",
            "0 0 Pause\n0 0 Resume",
            "34 0 Resume\n16 0 Pause",
            "340 0 Pause",
            "0 0 SetReleaseRate 999",
            "0 0 Resume 1",
        ] {
            assert!(parse_replay(text, Some(340)).is_err(), "{text}");
        }
    }
}
