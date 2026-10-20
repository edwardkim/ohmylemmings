use lemmings_web::WebGame;
#[test]
fn batched_steps_preserve_startup_events_and_consumption_keeps_state_and_frame_intact() {
    let mut a = WebGame::new("builder", "reference").unwrap();
    let mut b = WebGame::new("builder", "reference").unwrap();
    a.step(55.0).unwrap();
    b.step(55.0).unwrap();
    let state = a.state();
    let frame = a.frame().unwrap();
    let batch = a.drain_sound_events();
    assert!(batch.contains("\"reset\":true"));
    for cue in ["LetsGo", "EntranceOpening", "MusicStart"] {
        assert!(batch.contains(cue));
    }
    assert_eq!(batch.matches("\"cue\":").count(), 3);
    assert_eq!(a.state(), state);
    assert_eq!(a.frame().unwrap(), frame);
    assert_eq!(a.state(), b.state());
    assert_eq!(a.terrain(), b.terrain());
    assert!(a.drain_sound_events().contains("\"events\":[]"));
}
#[test]
fn repeat_and_focus_release_do_not_duplicate_paused_selection_cues() {
    let mut a = WebGame::new("builder", "reference").unwrap();
    a.command("Pause", 0.0).unwrap();
    a.key("KeyD", false).unwrap();
    a.key("KeyD", true).unwrap();
    a.release_input();
    a.step(1.0).unwrap();
    let batch = a.drain_sound_events();
    assert_eq!(batch.matches("SkillButtonSelect").count(), 1);
    assert!(batch.contains("\"iteration\":0"));
    a.pointer("down", 40.0, 185.0, 0.0, 1.0, 1.0).unwrap();
    a.pointer("up", 40.0, 185.0, 0.0, 0.0, 1.0).unwrap();
    a.step(1.0).unwrap();
    assert_eq!(
        a.drain_sound_events().matches("SkillButtonSelect").count(),
        1
    );
}
#[test]
fn failed_replacement_preserves_pending_events_and_successful_reset_changes_generation() {
    let mut a = WebGame::new("builder", "reference").unwrap();
    a.step(55.0).unwrap();
    let before = a.sound_state();
    let state = a.state();
    assert!(a.reset("unknown", "dos").is_err());
    assert_eq!(a.sound_state(), before);
    assert_eq!(a.state(), state);
    a.reset("basher", "dos").unwrap();
    let batch = a.drain_sound_events();
    assert!(batch.contains("\"generation\":1"));
    assert!(batch.contains("\"reset\":true"));
    assert!(batch.contains("\"events\":[]"));
    a.restart_level().unwrap();
    assert!(a.drain_sound_events().contains("\"generation\":2"));
}
