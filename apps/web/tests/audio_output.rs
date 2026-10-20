use lemmings_web::{WebAudio, WebGame};
#[test]
fn audio_consumption_is_independent_of_gameplay_and_replacement_flushes_old_pcm() {
    let mut a = WebGame::new("builder", "reference").unwrap();
    let mut b = WebGame::new("builder", "reference").unwrap();
    let mut output = WebAudio::new(44100.0).unwrap();
    assert!(output.prepare(&mut a, true).unwrap());
    a.step(55.0).unwrap();
    b.step(55.0).unwrap();
    output.prepare(&mut a, true).unwrap();
    let before = a.state();
    let pixels = a.frame().unwrap();
    let samples = output.render(4096.0).unwrap();
    assert!(samples.iter().any(|&s| s != 0));
    assert_eq!(a.state(), before);
    assert_eq!(a.state(), b.state());
    assert_eq!(a.terrain(), b.terrain());
    assert_eq!(a.frame().unwrap(), pixels);
    a.restart_level().unwrap();
    assert!(output.prepare(&mut a, true).unwrap());
    assert!(output.render(512.0).unwrap().iter().all(|&s| s == 0));
}
#[test]
fn invalid_output_requests_are_atomic_and_suspension_does_not_advance_the_game() {
    for n in [0.0, 7999.0, 192001.0, f64::NAN, 44100.5] {
        assert!(WebAudio::new(n).is_err());
    }
    let mut game = WebGame::new("builder", "reference").unwrap();
    let mut output = WebAudio::new(48000.0).unwrap();
    output.prepare(&mut game, true).unwrap();
    let before = output.state();
    for n in [-1.0, 0.5, 8193.0, f64::INFINITY] {
        assert!(output.render(n).is_err());
    }
    for n in [-1.0, 0.5, 101.0, f64::NAN] {
        assert!(output.set_volume(n).is_err());
    }
    assert_eq!(output.state(), before);
    let state = game.state();
    output.suspend();
    output.prepare(&mut game, false).unwrap();
    assert_eq!(output.render(1000.0).unwrap(), vec![0; 1000]);
    assert_eq!(game.state(), state);
}

mod support;
#[test]
fn optional_missing_or_damaged_adlib_leaves_game_and_sound_queue_usable() {
    use lemmings_web::WebAssets;
    let mut files = WebAssets::new();
    for (name, bytes) in support::bank(false) {
        files.insert(&name, &bytes).unwrap();
    }
    let mut game = WebGame::new("builder", "reference").unwrap();
    game.load_original(&files, 1.0).unwrap();
    game.step(55.0).unwrap();
    let mut audio = WebAudio::new(44100.0).unwrap();
    let before = (game.state(), game.sound_state(), game.frame().unwrap());
    assert!(
        audio
            .prepare(&mut game, true)
            .unwrap_err()
            .contains("ADLIB.DAT")
    );
    assert_eq!(
        (game.state(), game.sound_state(), game.frame().unwrap()),
        before
    );
    assert!(WebAssets::accepts("adlib.dat"));
    files.insert("adlib.dat", &[0]).unwrap();
    game.load_original(&files, 1.0).unwrap();
    let before = (game.state(), game.sound_state());
    assert!(
        audio
            .prepare(&mut game, true)
            .unwrap_err()
            .contains("ADLIB.DAT")
    );
    assert_eq!((game.state(), game.sound_state()), before);
    game.step(1.0).unwrap();
}
#[test]
fn music_keeps_its_audio_clock_while_logical_game_is_paused_and_result_holds_output() {
    let mut game = WebGame::new("win", "reference").unwrap();
    let mut audio = WebAudio::new(44100.0).unwrap();
    audio.prepare(&mut game, true).unwrap();
    game.step(55.0).unwrap();
    audio.prepare(&mut game, true).unwrap();
    audio.render(1024.0).unwrap();
    game.command("Pause", 0.0).unwrap();
    game.step(1.0).unwrap();
    audio.prepare(&mut game, true).unwrap();
    let before = game.state();
    let old = audio.state();
    assert!(audio.render(4096.0).unwrap().iter().any(|&v| v != 0));
    assert_ne!(audio.state(), old);
    assert_eq!(game.state(), before);
    game.command("Resume", 0.0).unwrap();
    game.step(1000.0).unwrap();
    audio.prepare(&mut game, false).unwrap();
    assert!(audio.render(4096.0).unwrap().iter().all(|&v| v == 0));
    game.transition("continue").unwrap();
    assert!(audio.prepare(&mut game, true).unwrap());
}
