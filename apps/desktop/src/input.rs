use crate::{render::View, session::Session};
use lemmings_core::{InputCommand, InputSchedule, StartupCommand, TimedInput};
use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Keycode,
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
        Event::KeyDown {
            keycode: Some(key),
            repeat,
            ..
        } => match key {
            Keycode::Escape => return Ok(false),
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
