use lemmings_web::WebGame;

#[test]
fn frames_are_owned_copies_and_rendering_does_not_tick() {
    let mut game = WebGame::new("builder", "reference").unwrap();
    let state = game.state();
    let original = game.frame().unwrap();
    assert_eq!(
        (game.width(), game.height(), original.len()),
        (320, 200, 192000)
    );
    let mut edited = original.clone();
    edited.fill(99);
    assert_eq!(game.frame().unwrap(), original);
    assert_eq!(game.state(), state);
}

#[test]
fn invalid_commands_and_batches_leave_state_and_frame_unchanged() {
    let mut game = WebGame::new("builder", "reference").unwrap();
    game.command("Pause", 0.0).unwrap();
    let before = game.state();
    let frame = game.frame().unwrap();
    for (name, value) in [
        ("unknown", 0.0),
        ("Pause", 1.0),
        ("Nuke", f64::NAN),
        ("SetReleaseRate", 49.0),
        ("SetReleaseRate", 100.0),
        ("AssignDigger", -1.0),
        ("AssignBuilder", 0.5),
        ("AssignBomber", 4294967296.0),
    ] {
        assert!(game.command(name, value).is_err(), "{name}/{value}");
        assert_eq!(game.state(), before);
        assert_eq!(game.frame().unwrap(), frame);
    }
    for count in [1001.0, -1.0, 0.5, f64::NAN, f64::INFINITY] {
        assert!(game.step(count).is_err());
    }
    assert_eq!(game.state(), before);
    assert!(game.paused());
    game.command("Resume", 0.0).unwrap();
    assert!(!game.paused());
    game.command("Pause", 0.0).unwrap();
    assert!(game.paused());
}

#[test]
fn elapsed_delivery_is_explicit_fractional_and_rejects_invalid_time_atomically() {
    let mut game = WebGame::new("builder", "reference").unwrap();
    let before = game.state();
    assert_eq!(game.due(58823.0).unwrap(), 0);
    for value in [f64::NAN, f64::INFINITY, -1.0, 0.5, 3600000001.0] {
        assert!(game.due(value).is_err());
    }
    assert_eq!(game.due(1.0).unwrap(), 1);
    assert_eq!(game.state(), before);
    assert_eq!(game.step(1.0).unwrap(), 1);
    assert_ne!(game.state(), before);
    let mut dos = WebGame::new("builder", "dos").unwrap();
    assert_eq!(dos.due(1000000.0).unwrap(), 14);
    let mut reference = WebGame::new("builder", "reference").unwrap();
    assert_eq!(reference.due(1000000.0).unwrap(), 17);
}

#[test]
fn reset_validates_before_replacing_session_view_and_clock() {
    let mut game = WebGame::new("builder", "reference").unwrap();
    game.step(80.0).unwrap();
    game.command("AssignBuilder", 0.0).unwrap();
    game.due(58823.0).unwrap();
    let before = game.state();
    let frame = game.frame().unwrap();
    assert!(game.reset("unknown", "reference").is_err());
    assert!(game.reset("win", "unknown").is_err());
    assert_eq!(game.state(), before);
    assert_eq!(game.frame().unwrap(), frame);
    assert_eq!(game.due(1.0).unwrap(), 1);
    game.reset("builder", "reference").unwrap();
    let mut fresh = WebGame::new("builder", "reference").unwrap();
    assert_eq!(game.state(), fresh.state());
    assert_eq!(game.frame().unwrap(), fresh.frame().unwrap());
    assert_eq!(game.due(1.0).unwrap(), 0);
}

#[test]
fn finished_session_freezes_rejects_commands_and_reports_actual_steps() {
    let mut game = WebGame::new("win", "reference").unwrap();
    let steps = game.step(1000.0).unwrap();
    assert!(steps > 0 && steps < 1000);
    assert!(game.state().contains("\"phase\":\"Won\""));
    let before = game.state();
    let frame = game.frame().unwrap();
    assert_eq!(game.step(1000.0).unwrap(), 0);
    assert!(game.command("Nuke", 0.0).is_err());
    assert_eq!(game.state(), before);
    assert_eq!(game.frame().unwrap(), frame);
}

#[test]
fn all_eight_commands_share_direct_core_ids_and_native_frames() {
    use lemmings_core::{InputCommand, StartupCommand};
    use lemmings_game::{Assets, LevelRun, pacing::Pacing};
    for (name, command) in [
        ("AssignClimber", StartupCommand::AssignClimber(0)),
        ("AssignFloater", StartupCommand::AssignFloater(0)),
        ("AssignBomber", StartupCommand::AssignBomber(0)),
        ("AssignBlocker", StartupCommand::AssignBlocker(0)),
        ("AssignBuilder", StartupCommand::AssignBuilder(0)),
        ("AssignBasher", StartupCommand::AssignBasher(0)),
        ("AssignMiner", StartupCommand::AssignMiner(0)),
        ("AssignDigger", StartupCommand::AssignDigger(0)),
    ] {
        let mut web = WebGame::new("builder", "reference").unwrap();
        let mut direct =
            LevelRun::new(Assets::synthetic_builder(), 1, None, Pacing::Reference).unwrap();
        for step in 0..230 {
            if step == 80 {
                web.command(name, 0.0).unwrap();
                direct.session.queue(InputCommand::Custom(command)).unwrap();
            }
            assert!(
                web.state()
                    .contains(&format!("\"session\":{}", direct.session.snapshot()))
            );
            assert_eq!(web.terrain(), direct.session.terrain());
            assert_eq!(web.bricks(), direct.session.brick_mask());
            assert_eq!(web.objects(), direct.session.object_map_data());
            if step % 17 == 0 {
                assert_eq!(
                    web.frame().unwrap(),
                    direct.view.frame(&direct.assets, &direct.session).unwrap()
                );
            }
            web.step(1.0).unwrap();
            direct.session.tick().unwrap();
        }
    }
}
