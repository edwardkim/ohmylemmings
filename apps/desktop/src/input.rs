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

/// Scene commands are separate from tick commands and are checked only at a frozen result.
pub fn result_action(event: &Event) -> Option<crate::lifecycle::ResultAction> {
    use crate::lifecycle::ResultAction;
    match event {
        Event::KeyDown {
            keycode: Some(Keycode::R),
            repeat: false,
            ..
        } => Some(ResultAction::Restart),
        Event::KeyDown {
            keycode: Some(Keycode::Return),
            repeat: false,
            ..
        }
        | Event::MouseButtonDown {
            mouse_btn: MouseButton::Left,
            ..
        } => Some(ResultAction::Continue),
        _ => None,
    }
}

/// Return false on quit. Only gameplay commands enter the deterministic log.
pub fn handle(
    event: Event,
    session: &mut Session,
    view: &mut View,
    width: usize,
    replaying: bool,
) -> Result<bool, Box<dyn Error>> {
    view.panel.capture_pending = true;
    if session.result().is_some() {
        view.panel.release();
        if let Event::KeyDown {
            keycode: Some(key), ..
        } = event
        {
            match key {
                Keycode::Left => view.scroll(-16, width),
                Keycode::Right => view.scroll(16, width),
                Keycode::Home => view.scroll(i32::MIN, width),
                Keycode::End => view.scroll(i32::MAX, width),
                _ => {}
            }
        }
        return Ok(!matches!(
            event,
            Event::Quit { .. }
                | Event::Window {
                    win_event: WindowEvent::Close,
                    ..
                }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                }
                | Event::MouseButtonDown {
                    mouse_btn: MouseButton::Right,
                    ..
                }
        ));
    }
    let replaying = replaying || session.result().is_some();
    match event {
        Event::Quit { .. }
        | Event::Window {
            win_event: WindowEvent::Close,
            ..
        } => return Ok(false),
        Event::MouseButtonDown {
            mouse_btn: MouseButton::Right,
            ..
        } if session.result().is_some() => return Ok(false),
        Event::Window {
            win_event: WindowEvent::FocusLost,
            ..
        } => {
            view.panel.release();
            view.panel.pointer = None;
            view.selection = Default::default();
            view.right_mouse = false;
        }
        Event::MouseButtonDown {
            mouse_btn: MouseButton::Right,
            x,
            y,
            ..
        } => {
            view.panel.pointer = Some((x, y));
            view.right_mouse = true;
        }
        Event::MouseButtonUp {
            mouse_btn: MouseButton::Right,
            x,
            y,
            ..
        } => {
            view.panel.pointer = Some((x, y));
            view.right_mouse = false;
        }
        Event::MouseButtonUp {
            mouse_btn: MouseButton::Left,
            ..
        } => view.panel.release(),
        Event::MouseMotion {
            x, y, mousestate, ..
        } => {
            view.panel.pointer = Some((x, y));
            view.right_mouse = mousestate.right();
            if !mousestate.left() {
                view.panel.release();
            }
            if view.panel.dragging_map
                && let Some(camera) = crate::panel::minimap_camera(x, y, width)
            {
                view.camera = camera;
            }
        }
        Event::MouseButtonDown {
            mouse_btn: MouseButton::Left,
            x,
            y,
            clicks,
            ..
        } if (0..320).contains(&x) && (160..200).contains(&y) => {
            view.panel.release();
            view.panel.pointer = Some((x, y));
            if let Some(camera) = crate::panel::minimap_camera(x, y, width) {
                view.camera = camera;
                view.panel.dragging_map = true;
            } else if !replaying && let Some(button) = crate::panel::hit(x, y) {
                use crate::panel::Button;
                match button {
                    Button::Skill(skill) => crate::panel::select_skill(session, view, skill, true),
                    Button::Slower | Button::Faster => {
                        view.panel.held_rate = Some(if button == Button::Faster { 1 } else { -1 });
                        crate::panel::advance_input(session, view, false)?;
                    }
                    Button::Pause => {
                        session.queue(if session.desired_paused() {
                            InputCommand::Resume
                        } else {
                            InputCommand::Pause
                        })?;
                    }
                    Button::Nuke
                        if clicks >= 2
                            && !session.desired_paused()
                            && !session.scheduler.nuke_state().is_nuked =>
                    {
                        session.queue(InputCommand::Custom(StartupCommand::Nuke))?;
                    }
                    _ => {}
                }
            }
        }
        Event::MouseButtonDown {
            mouse_btn: MouseButton::Left,
            x,
            y,
            ..
        } if !replaying && (0..320).contains(&x) && (0..160).contains(&y) => {
            view.panel.pointer = Some((x, y));
            crate::pointer::refresh(session, view);
            if let Some(request) = view
                .selection
                .assignment(crate::pointer::skill(view.selected_skill))
            {
                let command = if !request.right_click
                    && request.secondary.is_none_or(|id| id == request.primary)
                {
                    crate::pointer::command(request.skill, request.primary)
                } else {
                    StartupCommand::AssignPointer(request)
                };
                session.queue(InputCommand::Custom(command))?;
            }
        }
        Event::KeyDown {
            keycode: Some(key),
            repeat,
            ..
        } => match key {
            Keycode::Escape => return Ok(false),
            Keycode::B => crate::panel::select_skill(
                session,
                view,
                SelectedSkill::Basher,
                !repeat && !replaying,
            ),
            Keycode::D => crate::panel::select_skill(
                session,
                view,
                SelectedSkill::Digger,
                !repeat && !replaying,
            ),
            Keycode::M => crate::panel::select_skill(
                session,
                view,
                SelectedSkill::Miner,
                !repeat && !replaying,
            ),
            Keycode::U => crate::panel::select_skill(
                session,
                view,
                SelectedSkill::Builder,
                !repeat && !replaying,
            ),
            Keycode::F => crate::panel::select_skill(
                session,
                view,
                SelectedSkill::Floater,
                !repeat && !replaying,
            ),
            Keycode::C => crate::panel::select_skill(
                session,
                view,
                SelectedSkill::Climber,
                !repeat && !replaying,
            ),
            Keycode::E => crate::panel::select_skill(
                session,
                view,
                SelectedSkill::Bomber,
                !repeat && !replaying,
            ),
            Keycode::K => crate::panel::select_skill(
                session,
                view,
                SelectedSkill::Blocker,
                !repeat && !replaying,
            ),
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
            Keycode::N if !repeat && !replaying && !session.desired_paused() => {
                session.queue(InputCommand::Custom(StartupCommand::Nuke))?;
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
    crate::pointer::refresh(session, view);
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
            ["Nuke"] => InputCommand::Custom(StartupCommand::Nuke),
            ["Resume"] => InputCommand::Resume,
            ["AssignDigger", id] => InputCommand::Custom(StartupCommand::AssignDigger(id.parse()?)),
            ["AssignBasher", id] => InputCommand::Custom(StartupCommand::AssignBasher(id.parse()?)),
            ["AssignMiner", id] => InputCommand::Custom(StartupCommand::AssignMiner(id.parse()?)),
            ["AssignBuilder", id] => {
                InputCommand::Custom(StartupCommand::AssignBuilder(id.parse()?))
            }
            ["AssignFloater", id] => {
                InputCommand::Custom(StartupCommand::AssignFloater(id.parse()?))
            }
            ["AssignClimber", id] => {
                InputCommand::Custom(StartupCommand::AssignClimber(id.parse()?))
            }
            ["AssignBomber", id] => InputCommand::Custom(StartupCommand::AssignBomber(id.parse()?)),
            ["AssignBlocker", id] => {
                InputCommand::Custom(StartupCommand::AssignBlocker(id.parse()?))
            }
            ["AssignPointer", skill, primary, secondary, right_click] => {
                use lemmings_core::AssignmentSkill::*;
                let skill = match skill {
                    "Climber" => Climber,
                    "Floater" => Floater,
                    "Bomber" => Bomber,
                    "Blocker" => Blocker,
                    "Builder" => Builder,
                    "Basher" => Basher,
                    "Miner" => Miner,
                    "Digger" => Digger,
                    _ => return Err("unknown pointer skill".into()),
                };
                InputCommand::Custom(StartupCommand::AssignPointer(
                    lemmings_core::PointerAssignment {
                        skill,
                        primary: primary.parse()?,
                        secondary: if secondary == "-" {
                            None
                        } else {
                            Some(secondary.parse()?)
                        },
                        right_click: right_click.parse()?,
                    },
                ))
            }
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
    #[test]
    fn browser_input_matches_sdl_state_maps_selection_and_frames_for_all_skills() {
        use crate::{lifecycle::LevelRun, pacing::Pacing};
        use lemmings_web::WebGame;
        for (code, keycode) in [
            ("KeyC", Keycode::C),
            ("KeyF", Keycode::F),
            ("KeyE", Keycode::E),
            ("KeyK", Keycode::K),
            ("KeyU", Keycode::U),
            ("KeyB", Keycode::B),
            ("KeyM", Keycode::M),
            ("KeyD", Keycode::D),
        ] {
            let mut native =
                LevelRun::new(Assets::synthetic_builder(), 1, None, Pacing::Reference).unwrap();
            let mut web = WebGame::new("builder", "reference").unwrap();
            for step in 0..150 {
                let mut events = Vec::new();
                if step == 0 {
                    web.key(code, false).unwrap();
                    events.push(key(keycode, false));
                    for code in ["Space", "Space", "ArrowUp"] {
                        web.key(code, false).unwrap();
                    }
                    events.extend([
                        key(Keycode::Space, false),
                        key(Keycode::Space, false),
                        key(Keycode::Up, false),
                    ]);
                }
                if step == 10 {
                    web.pointer("down", 24.0, 185.0, 0.0, 1.0, 1.0).unwrap();
                    events.push(click(24, 185, 1));
                }
                if step == 14 {
                    web.release_input();
                    events.push(Event::Window {
                        timestamp: 0,
                        window_id: 0,
                        win_event: WindowEvent::FocusLost,
                    });
                }
                if step == 80 {
                    let actor = native.session.actors[0].state();
                    let (x, y) = (actor.x, actor.y - 5);
                    web.pointer("move", x.into(), y.into(), 0.0, 0.0, 0.0)
                        .unwrap();
                    events.push(Event::MouseMotion {
                        timestamp: 0,
                        window_id: 0,
                        which: 0,
                        mousestate: sdl2::mouse::MouseState::from_sdl_state(0),
                        x,
                        y,
                        xrel: 0,
                        yrel: 0,
                    });
                    web.pointer("down", x.into(), y.into(), 2.0, 2.0, 1.0)
                        .unwrap();
                    events.push(Event::MouseButtonDown {
                        timestamp: 0,
                        window_id: 0,
                        which: 0,
                        mouse_btn: MouseButton::Right,
                        clicks: 1,
                        x,
                        y,
                    });
                    web.pointer("down", x.into(), y.into(), 0.0, 3.0, 1.0)
                        .unwrap();
                    events.push(click(x, y, 1));
                }
                if step == 82 {
                    web.release_input();
                    events.push(Event::Window {
                        timestamp: 0,
                        window_id: 0,
                        win_event: WindowEvent::FocusLost,
                    });
                }
                if step == 90 {
                    web.pointer("down", 280.0, 185.0, 0.0, 1.0, 1.0).unwrap();
                    events.push(click(280, 185, 1));
                }
                if step == 92 {
                    web.pointer("move", 250.0, 185.0, 0.0, 1.0, 0.0).unwrap();
                    events.push(Event::MouseMotion {
                        timestamp: 0,
                        window_id: 0,
                        which: 0,
                        mousestate: sdl2::mouse::MouseState::from_sdl_state(1),
                        x: 250,
                        y: 185,
                        xrel: -30,
                        yrel: 0,
                    });
                }
                if step == 94 {
                    web.release_input();
                    events.push(Event::Window {
                        timestamp: 0,
                        window_id: 0,
                        win_event: WindowEvent::FocusLost,
                    });
                }
                for event in events {
                    handle(
                        event,
                        &mut native.session,
                        &mut native.view,
                        native.assets.width,
                        false,
                    )
                    .unwrap();
                }
                assert!(
                    web.state()
                        .contains(&format!("\"session\":{}", native.session.snapshot())),
                    "{code}/{step}"
                );
                assert!(web.state().contains(&format!(
                    "\"inputs\":{:?}",
                    format!("{:?}", native.session.inputs)
                )));
                assert!(web.view_state().contains(&format!(
                    "\"selection\":{:?}",
                    format!("{:?}", native.view.selection)
                )));
                assert_eq!(web.terrain(), native.session.terrain());
                assert_eq!(web.bricks(), native.session.brick_mask());
                assert_eq!(web.objects(), native.session.object_map_data());
                if step % 17 == 0 || step == 80 || step == 92 {
                    assert_eq!(
                        web.frame().unwrap(),
                        native.view.frame(&native.assets, &native.session).unwrap(),
                        "{code}/{step}"
                    );
                }
                web.step(1.0).unwrap();
                crate::panel::advance_input(&mut native.session, &mut native.view, false).unwrap();
                crate::pointer::refresh(&native.session, &mut native.view);
                native.tick().unwrap();
                assert_eq!(
                    native.sound.drain_json(),
                    web.drain_sound_events(),
                    "sound at {code}/{step}"
                );
                crate::pointer::refresh(&native.session, &mut native.view);
            }
        }
    }
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
        // Input-selection tests need the fixture's actual skill inventory.
        let run = crate::lifecycle::LevelRun::new(
            Assets::synthetic(),
            1,
            None,
            crate::pacing::Pacing::Reference,
        )
        .unwrap();
        (run.session, run.view)
    }
    #[test]
    fn empty_skill_is_disabled_for_both_keyboard_and_toolbar() {
        let (mut s, mut v) = setup();
        s.remaining_diggers = 0;
        handle(key(Keycode::F, false), &mut s, &mut v, 640, false).unwrap();
        handle(key(Keycode::D, false), &mut s, &mut v, 640, false).unwrap();
        handle(click(145, 190, 1), &mut s, &mut v, 640, false).unwrap();
        assert_eq!(v.selected_skill, SelectedSkill::Floater);
        assert!(s.inputs.is_empty());
    }
    fn click(x: i32, y: i32, clicks: u8) -> Event {
        Event::MouseButtonDown {
            timestamp: 0,
            window_id: 0,
            which: 0,
            mouse_btn: MouseButton::Left,
            clicks,
            x,
            y,
        }
    }
    #[test]
    fn all_panel_skill_buttons_match_keyboard_and_do_not_enter_gameplay_log() {
        for (i, keycode) in [
            Keycode::C,
            Keycode::F,
            Keycode::E,
            Keycode::K,
            Keycode::U,
            Keycode::B,
            Keycode::M,
            Keycode::D,
        ]
        .into_iter()
        .enumerate()
        {
            let (mut s, mut v) = setup();
            handle(
                click(i as i32 * 16 + 33, 190, 1),
                &mut s,
                &mut v,
                640,
                false,
            )
            .unwrap();
            assert_eq!(v.selected_skill, crate::panel::SKILLS[i]);
            let mut keyboard = View::new(0);
            handle(key(keycode, false), &mut s, &mut keyboard, 640, false).unwrap();
            assert_eq!(v.selected_skill, keyboard.selected_skill);
            assert!(s.inputs.is_empty());
        }
    }
    #[test]
    fn panel_rate_hold_is_once_per_boundary_and_stops_on_release_focus_and_replay() {
        let (mut s, mut v) = setup();
        handle(click(20, 190, 1), &mut s, &mut v, 640, false).unwrap();
        for _ in 0..10 {
            crate::panel::advance_input(&mut s, &mut v, false).unwrap();
        }
        assert_eq!(s.desired_rate(), 51);
        assert_eq!(s.inputs.len(), 1);
        s.tick().unwrap();
        crate::panel::advance_input(&mut s, &mut v, false).unwrap();
        assert_eq!(s.desired_rate(), 52);
        handle(
            Event::MouseButtonUp {
                timestamp: 0,
                window_id: 0,
                which: 0,
                mouse_btn: MouseButton::Left,
                clicks: 1,
                x: 319,
                y: 0,
            },
            &mut s,
            &mut v,
            640,
            false,
        )
        .unwrap();
        assert_eq!(v.panel.held_rate, None);
        handle(click(20, 190, 1), &mut s, &mut v, 640, false).unwrap();
        handle(
            Event::Window {
                timestamp: 0,
                window_id: 0,
                win_event: WindowEvent::FocusLost,
            },
            &mut s,
            &mut v,
            640,
            false,
        )
        .unwrap();
        assert_eq!(v.panel.held_rate, None);
        handle(click(20, 190, 1), &mut s, &mut v, 640, false).unwrap();
        crate::panel::advance_input(&mut s, &mut v, true).unwrap();
        assert_eq!(v.panel.held_rate, None);
    }
    #[test]
    fn panel_pause_toggle_and_double_click_nuke_share_keyboard_commands() {
        let (mut s, mut v) = setup();
        handle(click(164, 190, 1), &mut s, &mut v, 640, false).unwrap();
        assert!(s.desired_paused());
        handle(click(180, 190, 2), &mut s, &mut v, 640, false).unwrap();
        assert_eq!(s.inputs.len(), 1);
        handle(click(164, 190, 1), &mut s, &mut v, 640, false).unwrap();
        handle(click(180, 190, 1), &mut s, &mut v, 640, false).unwrap();
        assert_eq!(s.inputs.len(), 2);
        handle(click(180, 190, 2), &mut s, &mut v, 640, false).unwrap();
        assert_eq!(
            s.inputs,
            parse_replay("0 0 Pause\n0 1 Resume\n0 2 Nuke", Some(10)).unwrap()
        );
    }
    #[test]
    fn replay_blocks_panel_commands_and_minimap_never_enqueues_assignments() {
        let (mut s, mut v) = setup();
        for x in [2, 20, 34, 164, 180] {
            handle(click(x, 190, 2), &mut s, &mut v, 1584, true).unwrap();
        }
        assert_eq!(v.selected_skill, SelectedSkill::Digger);
        assert!(s.inputs.is_empty());
        handle(click(250, 180, 1), &mut s, &mut v, 1584, true).unwrap();
        assert_eq!(v.camera, 496);
        assert!(v.panel.dragging_map);
        handle(
            Event::MouseMotion {
                timestamp: 0,
                window_id: 0,
                which: 0,
                mousestate: sdl2::mouse::MouseState::from_sdl_state(1),
                x: 310,
                y: 190,
                xrel: 60,
                yrel: 10,
            },
            &mut s,
            &mut v,
            1584,
            true,
        )
        .unwrap();
        assert_eq!(v.camera, 1264);
        assert!(s.inputs.is_empty());
    }
    #[test]
    fn panel_hold_replay_has_identical_logic_under_dense_and_sparse_rendering() {
        let assets = Assets::synthetic();
        let (mut live, mut view) = setup();
        let (mut replay, mut replay_view) = setup();
        for step in 0..100 {
            if step == 4 {
                handle(click(20, 190, 1), &mut live, &mut view, 640, false).unwrap();
            }
            if step == 24 {
                view.panel.release();
            }
            if step == 30 || step == 40 {
                handle(click(164, 190, 1), &mut live, &mut view, 640, false).unwrap();
            }
            crate::panel::advance_input(&mut live, &mut view, false).unwrap();
            for input in live.inputs.iter().filter(|i| i.control_step == step) {
                replay
                    .queue(match &input.command {
                        InputCommand::Pause => InputCommand::Pause,
                        InputCommand::Resume => InputCommand::Resume,
                        InputCommand::Custom(c) => InputCommand::Custom(*c),
                    })
                    .unwrap();
            }
            live.tick().unwrap();
            replay.tick().unwrap();
            view.frame(&assets, &live).unwrap();
            if step % 7 == 0 {
                replay_view.frame(&assets, &replay).unwrap();
            }
            assert_eq!(live.snapshot(), replay.snapshot());
            assert_eq!(live.terrain(), replay.terrain());
        }
        assert_eq!(live.inputs, replay.inputs);
        assert_eq!(live.scheduler.state().release_rate, 70);
    }
    #[test]
    fn result_controls_ignore_repeat_and_use_separate_scene_actions() {
        use crate::lifecycle::ResultAction;
        assert_eq!(
            result_action(&key(Keycode::R, false)),
            Some(ResultAction::Restart)
        );
        assert_eq!(
            result_action(&key(Keycode::Return, false)),
            Some(ResultAction::Continue)
        );
        assert_eq!(result_action(&key(Keycode::Return, true)), None);
        assert_eq!(result_action(&key(Keycode::R, true)), None);
        let (mut session, mut view) = setup();
        for code in [Keycode::R, Keycode::Return] {
            handle(key(code, false), &mut session, &mut view, 640, false).unwrap();
        }
        assert!(session.inputs.is_empty());
        let click = |mouse_btn| Event::MouseButtonDown {
            timestamp: 0,
            window_id: 0,
            which: 0,
            mouse_btn,
            clicks: 1,
            x: 120,
            y: 80,
        };
        assert_eq!(
            result_action(&click(MouseButton::Left)),
            Some(ResultAction::Continue)
        );
        assert_eq!(result_action(&click(MouseButton::Right)), None);
        assert!(
            handle(
                click(MouseButton::Right),
                &mut session,
                &mut view,
                640,
                false
            )
            .unwrap()
        );
        session.configure_completion(1).unwrap();
        session
            .queue(InputCommand::Custom(StartupCommand::Nuke))
            .unwrap();
        session.tick().unwrap();
        assert!(
            !handle(
                click(MouseButton::Right),
                &mut session,
                &mut view,
                640,
                false
            )
            .unwrap()
        );
    }
    #[test]
    fn finished_level_ignores_gameplay_keys_but_keeps_camera_and_quit() {
        let (mut s, mut v) = setup();
        s.configure_completion(1).unwrap();
        s.queue(InputCommand::Custom(StartupCommand::Nuke)).unwrap();
        s.tick().unwrap();
        assert!(s.result().is_some());
        let before = s.snapshot();
        let count = s.inputs.len();
        for code in [Keycode::Space, Keycode::N, Keycode::Up, Keycode::Down] {
            assert!(handle(key(code, false), &mut s, &mut v, 640, false).unwrap());
        }
        assert_eq!(s.inputs.len(), count);
        assert_eq!(s.snapshot(), before);
        handle(key(Keycode::Right, false), &mut s, &mut v, 640, false).unwrap();
        assert_eq!(v.camera, 16);
        assert!(!handle(key(Keycode::Escape, false), &mut s, &mut v, 640, false).unwrap());
    }
    #[test]
    fn nuke_key_records_id_free_command_ignores_repeat_replay_and_pause() {
        let (mut s, mut v) = setup();
        handle(key(Keycode::N, true), &mut s, &mut v, 640, false).unwrap();
        handle(key(Keycode::N, false), &mut s, &mut v, 640, true).unwrap();
        assert!(s.inputs.is_empty());
        handle(key(Keycode::N, false), &mut s, &mut v, 640, false).unwrap();
        assert_eq!(s.inputs, parse_replay("0 0 Nuke", Some(10)).unwrap());
        assert!(parse_replay("0 0 Nuke 0", Some(10)).is_err());
        s.queue(InputCommand::Pause).unwrap();
        let count = s.inputs.len();
        handle(key(Keycode::N, false), &mut s, &mut v, 640, false).unwrap();
        assert_eq!(s.inputs.len(), count);
    }
    #[test]
    fn floater_selection_click_and_replay_keep_stable_actor_id() {
        let (mut s, mut v) = setup();
        for _ in 0..80 {
            s.tick().unwrap();
        }
        let a = s.actors[0].state();
        handle(key(Keycode::F, false), &mut s, &mut v, 640, false).unwrap();
        handle(
            Event::MouseButtonDown {
                timestamp: 0,
                window_id: 0,
                which: 0,
                mouse_btn: MouseButton::Left,
                clicks: 1,
                x: a.x,
                y: a.y - 5,
            },
            &mut s,
            &mut v,
            640,
            false,
        )
        .unwrap();
        assert_eq!(v.selected_skill, SelectedSkill::Floater);
        assert_eq!(
            parse_replay("80 0 AssignFloater 0", Some(340)).unwrap(),
            s.inputs
        );
        for input in [
            "80 0 AssignFloater",
            "80 0 AssignFloater -1",
            "80 0 AssignFloater 0 1",
        ] {
            assert!(parse_replay(input, Some(340)).is_err());
        }
    }
    #[test]
    fn climber_selection_click_and_replay_keep_stable_actor_id() {
        let (mut s, mut v) = setup();
        for _ in 0..80 {
            s.tick().unwrap();
        }
        let a = s.actors[0].state();
        handle(key(Keycode::C, false), &mut s, &mut v, 640, false).unwrap();
        handle(
            Event::MouseButtonDown {
                timestamp: 0,
                window_id: 0,
                which: 0,
                mouse_btn: MouseButton::Left,
                clicks: 1,
                x: a.x,
                y: a.y - 5,
            },
            &mut s,
            &mut v,
            640,
            false,
        )
        .unwrap();
        assert_eq!(v.selected_skill, SelectedSkill::Climber);
        assert_eq!(
            parse_replay("80 0 AssignClimber 0", Some(340)).unwrap(),
            s.inputs
        );
        for input in [
            "80 0 AssignClimber",
            "80 0 AssignClimber -1",
            "80 0 AssignClimber 0 1",
        ] {
            assert!(parse_replay(input, Some(340)).is_err());
        }
    }
    #[test]
    fn bomber_selection_click_and_replay_keep_stable_actor_id() {
        let (mut s, mut v) = setup();
        for _ in 0..80 {
            s.tick().unwrap();
        }
        let a = s.actors[0].state();
        handle(key(Keycode::E, false), &mut s, &mut v, 640, false).unwrap();
        handle(
            Event::MouseButtonDown {
                timestamp: 0,
                window_id: 0,
                which: 0,
                mouse_btn: MouseButton::Left,
                clicks: 1,
                x: a.x,
                y: a.y - 5,
            },
            &mut s,
            &mut v,
            640,
            false,
        )
        .unwrap();
        assert_eq!(v.selected_skill, SelectedSkill::Bomber);
        assert_eq!(
            parse_replay("80 0 AssignBomber 0", Some(340)).unwrap(),
            s.inputs
        );
        for input in [
            "80 0 AssignBomber",
            "80 0 AssignBomber -1",
            "80 0 AssignBomber 0 1",
        ] {
            assert!(parse_replay(input, Some(340)).is_err());
        }
    }
    #[test]
    fn blocker_selection_click_and_replay_keep_stable_actor_id() {
        let (mut s, mut v) = setup();
        for _ in 0..80 {
            s.tick().unwrap();
        }
        let a = s.actors[0].state();
        handle(key(Keycode::K, false), &mut s, &mut v, 640, false).unwrap();
        handle(
            Event::MouseButtonDown {
                timestamp: 0,
                window_id: 0,
                which: 0,
                mouse_btn: MouseButton::Left,
                clicks: 1,
                x: a.x,
                y: a.y - 5,
            },
            &mut s,
            &mut v,
            640,
            false,
        )
        .unwrap();
        assert_eq!(v.selected_skill, SelectedSkill::Blocker);
        assert_eq!(
            parse_replay("80 0 AssignBlocker 0", Some(340)).unwrap(),
            s.inputs
        );
        for input in [
            "80 0 AssignBlocker",
            "80 0 AssignBlocker -1",
            "80 0 AssignBlocker 0 1",
        ] {
            assert!(parse_replay(input, Some(340)).is_err());
        }
    }
    #[test]
    fn builder_key_and_click_record_stable_id_and_validate_replay_arguments() {
        let (mut s, mut v) = setup();
        for _ in 0..80 {
            s.tick().unwrap();
        }
        let a = s.actors[0].state();
        handle(key(Keycode::U, false), &mut s, &mut v, 640, false).unwrap();
        let click = Event::MouseButtonDown {
            timestamp: 0,
            window_id: 0,
            which: 0,
            mouse_btn: MouseButton::Left,
            clicks: 1,
            x: a.x,
            y: a.y - 5,
        };
        handle(click, &mut s, &mut v, 640, false).unwrap();
        assert_eq!(v.selected_skill, SelectedSkill::Builder);
        assert_eq!(
            parse_replay("80 0 AssignBuilder 0", Some(340)).unwrap(),
            s.inputs
        );
        for text in [
            "80 0 AssignBuilder",
            "80 0 AssignBuilder -1",
            "80 0 AssignBuilder 0 1",
        ] {
            assert!(parse_replay(text, Some(340)).is_err());
        }
    }
    #[test]
    fn miner_selection_records_id_and_replay_rejects_malformed_miner_arguments() {
        let (mut s, mut v) = setup();
        for _ in 0..80 {
            s.tick().unwrap();
        }
        let a = s.actors[0].state();
        handle(key(Keycode::M, false), &mut s, &mut v, 640, false).unwrap();
        let click = Event::MouseButtonDown {
            timestamp: 0,
            window_id: 0,
            which: 0,
            mouse_btn: MouseButton::Left,
            clicks: 1,
            x: a.x,
            y: a.y - 5,
        };
        handle(click, &mut s, &mut v, 640, false).unwrap();
        assert_eq!(
            parse_replay("80 0 AssignMiner 0", Some(340)).unwrap(),
            s.inputs
        );
        for text in [
            "80 0 AssignMiner",
            "80 0 AssignMiner -1",
            "80 0 AssignMiner 0 1",
        ] {
            assert!(parse_replay(text, Some(340)).is_err());
        }
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
