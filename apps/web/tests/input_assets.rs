mod support;
use lemmings_web::{WebAssets, WebGame};

fn bank() -> WebAssets {
    let mut bank = WebAssets::new();
    for (name, bytes) in support::bank(false) {
        bank.insert(&name.to_lowercase(), &bytes).unwrap();
    }
    bank
}

#[test]
fn named_bytes_validate_before_mutation_and_game_retains_immutable_snapshot() {
    let mut files = bank();
    let count = files.count();
    for name in [
        "../MAIN.DAT",
        "dir/MAIN.DAT",
        "MAIN.DAT ",
        "",
        "VGASPEC4.DAT",
        "LEVEL010.DAT",
    ] {
        assert!(files.insert(name, &[1]).is_err(), "{name}");
    }
    assert!(files.insert("main.dat", &[1]).is_err());
    assert!(files.insert("VGALEMMI.EXE", &vec![0; 1_048_577]).is_err());
    assert_eq!(files.count(), count);
    let mut game = WebGame::new("builder", "reference").unwrap();
    game.load_original(&files, 1.0).unwrap();
    let state = game.state();
    let frame = game.frame().unwrap();
    // Optional corrupt EXE added after load must not poison the game's bank.
    files.insert("VGALEMMI.EXE", &[0]).unwrap();
    game.restart_level().unwrap();
    assert_eq!(game.state(), state);
    assert_eq!(game.frame().unwrap(), frame);
    assert!(game.load_original(&files, 1.0).is_err());
    assert_eq!(game.state(), state);
}

#[test]
fn missing_damaged_and_invalid_level_loads_leave_session_view_and_clock_intact() {
    let mut game = WebGame::new("builder", "reference").unwrap();
    game.step(80.0).unwrap();
    game.key("KeyU", false).unwrap();
    game.key("ArrowRight", false).unwrap();
    game.due(58823.0).unwrap();
    let before = (game.state(), game.view_state(), game.frame().unwrap());
    for number in [0.0, 31.0, 1.5, f64::NAN, f64::INFINITY] {
        assert!(game.load_original(&bank(), number).is_err());
    }
    assert!(
        game.load_original(&WebAssets::new(), 1.0)
            .unwrap_err()
            .contains("LEVEL009.DAT")
    );
    let mut broken = WebAssets::new();
    broken.insert("LEVEL009.DAT", &[0]).unwrap();
    assert!(game.load_original(&broken, 1.0).is_err());
    assert_eq!(
        (game.state(), game.view_state(), game.frame().unwrap()),
        before
    );
    assert_eq!(game.due(1.0).unwrap(), 1);
}

#[test]
fn keys_panel_pointer_hold_and_focus_release_use_control_boundaries() {
    let mut game = WebGame::new("builder", "reference").unwrap();
    for (key, skill) in [
        ("KeyC", "Climber"),
        ("KeyF", "Floater"),
        ("KeyE", "Bomber"),
        ("KeyK", "Blocker"),
        ("KeyU", "Builder"),
        ("KeyB", "Basher"),
        ("KeyM", "Miner"),
        ("KeyD", "Digger"),
    ] {
        game.key(key, false).unwrap();
        assert!(
            game.view_state()
                .contains(&format!("\"skill\":\"{skill}\""))
        );
    }
    game.key("Space", false).unwrap();
    game.key("Space", true).unwrap();
    assert!(game.paused());
    game.key("Space", false).unwrap();
    assert!(!game.paused());
    game.pointer("down", 24.0, 185.0, 0.0, 1.0, 1.0).unwrap();
    assert!(game.state().contains("SetReleaseRate(51)"));
    game.frame().unwrap();
    game.frame().unwrap();
    game.step(1.0).unwrap();
    assert!(!game.state().contains("SetReleaseRate(52)"));
    game.step(1.0).unwrap();
    assert!(game.state().contains("SetReleaseRate(52)"));
    game.pointer("down", 64.0, 80.0, 2.0, 3.0, 1.0).unwrap();
    assert!(game.view_state().contains("\"right\":true"));
    game.release_input();
    assert!(game.view_state().contains("\"right\":false"));
    let before = game.state();
    game.step(1.0).unwrap();
    assert_eq!(
        game.state().matches("SetReleaseRate").count(),
        before.matches("SetReleaseRate").count()
    );
    let view = game.view_state();
    for (kind, x, y, button, buttons, clicks) in [
        ("oops", 0.0, 0.0, 0.0, 0.0, 1.0),
        ("down", f64::NAN, 0.0, 0.0, 1.0, 1.0),
        ("down", 0.5, 0.0, 0.0, 1.0, 1.0),
        ("down", 0.0, 0.0, 4294967296.0, 1.0, 1.0),
    ] {
        assert!(game.pointer(kind, x, y, button, buttons, clicks).is_err());
        assert_eq!(game.view_state(), view);
    }
}

#[test]
fn result_continue_retries_advances_and_restart_resets_all_presentation() {
    let mut game = WebGame::new("win", "reference").unwrap();
    assert!(!game.transition("continue").unwrap());
    game.step(1000.0).unwrap();
    assert!(game.state().contains("\"phase\":\"Won\""));
    assert_eq!(game.key("Enter", true).unwrap(), "none");
    assert_eq!(game.key("Enter", false).unwrap(), "continue");
    assert!(game.transition("continue").unwrap());
    assert_eq!(game.level(), 2);
    game.step(1000.0).unwrap();
    assert!(game.state().contains("\"phase\":\"Lost\""));
    assert!(game.transition("continue").unwrap());
    assert_eq!(game.level(), 2);
    assert!(game.state().contains("\"phase\":\"Playing\""));
    game.key("KeyU", false).unwrap();
    game.key("ArrowRight", false).unwrap();
    game.due(58823.0).unwrap();
    game.restart_level().unwrap();
    assert!(game.view_state().contains("\"skill\":\"Digger\""));
    assert_eq!(game.due(1.0).unwrap(), 0);
    assert!(game.state().contains("\"inputs\":\"[]\""));
    assert!(game.transition("bad").is_err());
}

#[test]
fn original_bytes_match_direct_native_game_and_level_switch_resets_clock_and_view() {
    use lemmings_game::{Assets, LevelRun, pacing::Pacing};
    let mut bytes = support::bank(false);
    let level_bytes = bytes["LEVEL009.DAT"].clone();
    for number in 0..10 {
        bytes.insert(format!("LEVEL{number:03}.DAT"), level_bytes.clone());
    }
    let mut files = WebAssets::new();
    for (name, data) in &bytes {
        files.insert(name, data).unwrap();
    }
    let mut game = WebGame::new("builder", "reference").unwrap();
    game.load_original(&files, 1.0).unwrap();
    for level in [1, 2, 30] {
        game.start_level(f64::from(level)).unwrap();
        let assets =
            Assets::from_source(&mut |name: &str| Ok(bytes.get(name).cloned()), level).unwrap();
        let mut native = LevelRun::new(
            assets,
            level,
            (level < 30).then_some(level + 1),
            Pacing::Dos,
        )
        .unwrap();
        assert!(game.view_state().contains("\"skill\":\"Digger\""));
        assert_eq!(game.due(1.0).unwrap(), 0);
        for step in 0..85 {
            assert!(
                game.state()
                    .contains(&format!("\"session\":{}", native.session.snapshot()))
            );
            assert_eq!(game.terrain(), native.session.terrain());
            assert_eq!(game.objects(), native.session.object_map_data());
            if step % 17 == 0 {
                assert_eq!(
                    game.frame().unwrap(),
                    native.view.frame(&native.assets, &native.session).unwrap()
                );
            }
            game.step(1.0).unwrap();
            native.session.tick().unwrap();
        }
        game.key("KeyU", false).unwrap();
        game.key("End", false).unwrap();
        game.due(69999.0).unwrap();
    }
    assert!(game.view_state().contains("\"next\":1"));
    assert!(game.view_state().contains("\"next_difficulty\":\"tricky\""));
    let before = (game.state(), game.view_state(), game.frame().unwrap());
    let mut special = WebAssets::new();
    for (name, data) in support::bank(true) {
        special.insert(&name, &data).unwrap();
    }
    assert!(game.load_original(&special, 1.0).is_err());
    assert_eq!(
        (game.state(), game.view_state(), game.frame().unwrap()),
        before
    );
}

#[test]
fn special_asset_names_are_bounded_and_missing_or_damaged_replacement_is_atomic() {
    let mut game = WebGame::new("builder", "reference").unwrap();
    for (index, music) in [(0, 0), (1, 8), (2, 2), (3, 1)] {
        let mut files = WebAssets::new();
        let bytes = support::special_bank(index);
        for (name, data) in &bytes {
            files.insert(name, data).unwrap();
        }
        game.load_original(&files, 1.0).unwrap();
        assert!(game.sound_state().contains(&format!("\"music\":{music}")));
        game.step(80.0).unwrap();
        game.key("KeyU", false).unwrap();
        game.due(69999.0).unwrap();
        let before = (
            game.state(),
            game.view_state(),
            game.frame().unwrap(),
            game.sound_state(),
        );
        for replacement in [None, Some(vec![0; 4])] {
            let mut broken = WebAssets::new();
            for (name, data) in &bytes {
                if name == &format!("VGASPEC{index}.DAT") {
                    if let Some(data) = &replacement {
                        broken.insert(name, data).unwrap();
                    }
                } else {
                    broken.insert(name, data).unwrap();
                }
            }
            assert!(game.load_original(&broken, 1.0).is_err());
            assert_eq!(
                (
                    game.state(),
                    game.view_state(),
                    game.frame().unwrap(),
                    game.sound_state()
                ),
                before
            );
        }
        assert_eq!(game.due(1428.0).unwrap(), 1); // failed loads kept fractional clock
        let generation = game.sound_state();
        game.restart_level().unwrap();
        assert_ne!(game.sound_state(), generation);
        assert!(game.sound_state().contains(&format!("\"music\":{music}")));
        assert_eq!(
            game.terrain(),
            lemmings_game::Assets::from_source(&mut |n: &str| Ok(bytes.get(n).cloned()), 1)
                .unwrap()
                .terrain
        );
    }
}
