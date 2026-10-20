mod support;
use lemmings_web::{WebAssets, WebGame};

fn bank() -> WebAssets {
    bank_with_exit(false)
}
fn bank_with_exit(exit: bool) -> WebAssets {
    let mut bytes = if !exit {
        support::bank(false)
    } else {
        support::bank_with_exit(false, exit)
    };
    let dat = bytes["LEVEL009.DAT"].clone();
    for n in 0..10 {
        bytes.insert(format!("LEVEL{n:03}.DAT"), dat.clone());
    }
    let mut files = WebAssets::new();
    for (name, data) in bytes {
        files.insert(&name, &data).unwrap();
    }
    files
}

#[test]
fn successful_rank_end_advances_and_terminal_mayhem_preserves_result() {
    let files = bank_with_exit(true);
    let mut game = WebGame::new("builder", "reference").unwrap();
    for (rank, next_rank) in [
        ("fun", Some("tricky")),
        ("tricky", Some("taxing")),
        ("taxing", Some("mayhem")),
        ("mayhem", None),
    ] {
        game.load_original_rank(&files, rank, 30.0).unwrap();
        game.step(1000.0).unwrap();
        assert!(game.state().contains("\"phase\":\"Won\""));
        let before = (game.state(), game.sound_state());
        assert_eq!(game.transition("continue").unwrap(), next_rank.is_some());
        if let Some(next_rank) = next_rank {
            assert_eq!(game.difficulty(), next_rank);
            assert_eq!(game.level(), 1);
            assert!(game.sound_state().contains("\"music\":6"));
        } else {
            assert_eq!((game.state(), game.sound_state()), before);
        }
    }
}

#[test]
fn rank_selection_restart_and_legacy_fun_entry_keep_distinct_identity() {
    let files = bank();
    let mut game = WebGame::new("builder", "reference").unwrap();
    for rank in ["fun", "tricky", "taxing", "mayhem"] {
        for number in [1.0, 30.0] {
            game.load_original_rank(&files, rank, number).unwrap();
            assert_eq!(game.difficulty(), rank);
            assert_eq!(game.level(), number as u8);
            let initial = game.state();
            game.step(80.0).unwrap();
            game.restart_level().unwrap();
            assert_eq!(game.difficulty(), rank);
            assert_eq!(game.state(), initial);
            assert!(
                game.view_state()
                    .contains(&format!("\"difficulty\":\"{rank}\""))
            );
        }
    }
    game.load_original(&files, 2.0).unwrap();
    assert_eq!(game.difficulty(), "fun");
    game.start_level_rank("taxing", 3.0).unwrap();
    assert_eq!(game.difficulty(), "taxing");
    game.start_level(1.0).unwrap();
    assert_eq!(game.difficulty(), "fun");
}

#[test]
fn invalid_or_missing_rank_replacement_preserves_state_view_sound_and_clock() {
    let files = bank();
    let mut game = WebGame::new("builder", "reference").unwrap();
    game.load_original_rank(&files, "tricky", 1.0).unwrap();
    game.step(80.0).unwrap();
    game.due(71000.0).unwrap();
    let before = (
        game.state(),
        game.view_state(),
        game.sound_state(),
        game.frame().unwrap(),
    );
    for (rank, number) in [
        ("invalid", 1.0),
        ("fun", 0.0),
        ("taxing", 31.0),
        ("mayhem", 1.5),
        ("tricky", f64::NAN),
    ] {
        assert!(game.start_level_rank(rank, number).is_err());
    }
    assert!(
        game.load_original_rank(&WebAssets::new(), "mayhem", 1.0)
            .is_err()
    );
    assert_eq!(
        (
            game.state(),
            game.view_state(),
            game.sound_state(),
            game.frame().unwrap()
        ),
        before
    );
    assert_eq!(game.due(341.0).unwrap(), 1);
}
