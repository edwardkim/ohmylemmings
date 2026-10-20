//! Own asset journey through the same input, view and level-lifecycle adapters.
use crate::{
    assets::Assets,
    input,
    lifecycle::{LevelRun, ResultAction},
    pacing::Pacing,
    render::SelectedSkill,
};
use lemmings_core::{InputCommand, StartupCommand};
use sdl2::{
    event::Event,
    keyboard::Keycode,
    mouse::{MouseButton, MouseState},
};

fn input(run: &mut LevelRun, event: Event) {
    assert!(
        input::handle(
            event,
            &mut run.session,
            &mut run.view,
            run.assets.width,
            false
        )
        .unwrap()
    );
}
fn mouse(x: i32, y: i32, down: bool, clicks: u8) -> Event {
    if down {
        Event::MouseButtonDown {
            timestamp: 0,
            window_id: 0,
            which: 0,
            mouse_btn: MouseButton::Left,
            clicks,
            x,
            y,
        }
    } else {
        Event::MouseButtonUp {
            timestamp: 0,
            window_id: 0,
            which: 0,
            mouse_btn: MouseButton::Left,
            clicks,
            x,
            y,
        }
    }
}
fn click(run: &mut LevelRun, x: i32, y: i32, clicks: u8) {
    input(run, mouse(x, y, true, clicks));
    input(run, mouse(x, y, false, clicks));
}
fn advance(run: &mut LevelRun) {
    crate::panel::advance_input(&mut run.session, &mut run.view, false).unwrap();
    crate::pointer::refresh(&run.session, &mut run.view);
    run.session.tick().unwrap();
    crate::pointer::refresh(&run.session, &mut run.view);
}
fn until(run: &mut LevelRun, step: u64) {
    while run.session.scheduler.step_state().control_step < step {
        assert!(run.session.result().is_none());
        advance(run);
    }
}
fn finish(run: &mut LevelRun, expect_success: bool) {
    for _ in 0..1200 {
        if run.session.result().is_some() {
            break;
        }
        advance(run);
    }
    let result = run
        .session
        .result()
        .expect("bounded own fixture must finish");
    assert_eq!(result.success, expect_success);
    let state = run.session.snapshot();
    let frame = run.view.frame(&run.assets, &run.session).unwrap();
    assert_eq!(&frame[..3], &[12, 18, 32]);
    for _ in 0..4 {
        advance(run);
    }
    assert_eq!(run.session.snapshot(), state);
    assert_eq!(run.view.frame(&run.assets, &run.session).unwrap(), frame);
}
fn transition(old: &LevelRun, event: Event) -> LevelRun {
    let action = input::result_action(&event).unwrap();
    let next = old
        .prepare(action, Pacing::Reference, |level| {
            Ok((
                Assets::synthetic_completion(if level == 1 { "win" } else { "loss" }),
                Some(level + 1),
            ))
        })
        .unwrap()
        .unwrap();
    let fresh = LevelRun::new(
        Assets::synthetic_completion(if next.level == 1 { "win" } else { "loss" }),
        next.level,
        Some(next.level + 1),
        Pacing::Reference,
    )
    .unwrap();
    assert_eq!(next.session.snapshot(), fresh.session.snapshot());
    assert_eq!(next.session.terrain(), fresh.session.terrain());
    assert_eq!(
        next.session.object_map_data(),
        fresh.session.object_map_data()
    );
    assert_eq!(next.view.selected_skill, SelectedSkill::Digger);
    assert_eq!(next.view.panel.pointer, None);
    assert!(!next.view.right_mouse);
    assert_eq!(
        next.view.selection,
        lemmings_core::PointerSelection::default()
    );
    assert!(next.session.inputs.is_empty());
    assert_eq!(next.view.panel.held_rate, None);
    assert!(!next.view.panel.dragging_map);
    next
}

#[test]
fn panel_pointer_win_nuke_loss_retry_win_and_next_level_start_fresh() {
    let mut run = LevelRun::new(
        Assets::synthetic_completion("win"),
        1,
        Some(2),
        Pacing::Reference,
    )
    .unwrap();
    until(&mut run, 80);
    let actor = run.session.actors[0].state();
    click(&mut run, 36, 190, 1);
    click(&mut run, actor.x, actor.y - 5, 1);
    advance(&mut run);
    assert!(run.session.actors[0].state().is_climber);
    click(&mut run, 164, 190, 1);
    advance(&mut run);
    assert!(run.session.desired_paused());
    let iteration = run.session.scheduler.step_state().iteration;
    input(&mut run, mouse(20, 190, true, 1));
    until(&mut run, 88);
    input(&mut run, mouse(20, 190, false, 1));
    assert!(run.session.desired_rate() > run.assets.config.initial_release_rate);
    assert_eq!(run.session.scheduler.step_state().iteration, iteration);
    click(&mut run, 310, 190, 1);
    assert!(run.view.camera > 0);
    input(&mut run, mouse(208, 190, true, 1));
    input(
        &mut run,
        Event::MouseMotion {
            timestamp: 0,
            window_id: 0,
            which: 0,
            mousestate: MouseState::from_sdl_state(1),
            x: 208,
            y: 190,
            xrel: 0,
            yrel: 0,
        },
    );
    input(&mut run, mouse(208, 190, false, 1));
    assert_eq!(run.view.camera, 0);
    click(&mut run, 164, 190, 1);
    finish(&mut run, true);
    let original_terrain = run.assets.terrain.clone();
    let old_result = run.session.result();
    let old_inputs = run.session.inputs.len();
    run = transition(&run, crate::key_event(Keycode::R));
    assert_eq!(run.level, 1);
    assert_eq!(run.session.terrain(), original_terrain);
    until(&mut run, 80);
    let a = run.session.actors[0].state();
    click(&mut run, 148, 190, 1);
    click(&mut run, a.x, a.y - 5, 1);
    click(&mut run, 180, 190, 1);
    assert!(
        !run.session
            .inputs
            .iter()
            .any(|i| i.command == InputCommand::Custom(StartupCommand::Nuke))
    );
    click(&mut run, 180, 190, 2);
    finish(&mut run, false);
    assert_eq!(run.session.scheduler.nuke_state().max_lemmings, 1);
    assert_ne!(run.session.terrain(), original_terrain);
    assert!(old_result.unwrap().success);
    assert!(old_inputs > 0);
    run = transition(&run, mouse(160, 130, true, 1));
    assert_eq!(run.level, 1);
    assert_eq!(run.session.terrain(), original_terrain);
    assert!(!run.session.scheduler.nuke_state().is_nuked);
    finish(&mut run, true);
    run = transition(&run, crate::key_event(Keycode::Return));
    assert_eq!(run.level, 2);
    assert_eq!(run.target(ResultAction::Continue), None);
    until(&mut run, 80);
    assert_eq!(run.session.actors[0].state().id, 0);
    assert!(run.session.inputs.is_empty());
}
