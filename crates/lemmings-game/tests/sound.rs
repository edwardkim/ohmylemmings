use lemmings_core::{InputCommand, SoundCue};
use lemmings_game::{Assets, LevelRun, ResultAction, pacing::Pacing, sound::SOUND_EVENT_CAPACITY};
use lemmings_resources::AdlibCommand;
#[test]
fn shared_run_preserves_all_startup_commands_when_host_drains_after_many_ticks() {
    let mut run = LevelRun::new(Assets::synthetic(), 1, None, Pacing::Reference).unwrap();
    for _ in 0..55 {
        run.tick().unwrap();
    }
    let batch = run.sound.drain();
    assert!(batch.reset);
    assert_eq!(batch.generation, 0);
    assert_eq!(batch.dropped, 0);
    assert_eq!(
        batch
            .events
            .iter()
            .map(|e| (e.source.iteration, e.command))
            .collect::<Vec<_>>(),
        [
            (15, Some(AdlibCommand::Effect(2))),
            (35, Some(AdlibCommand::Effect(1))),
            (55, Some(AdlibCommand::Music(5)))
        ]
    );
    assert_eq!(
        batch.events.iter().map(|e| e.sequence).collect::<Vec<_>>(),
        [0, 1, 2]
    );
    assert!(!run.sound.drain().reset);
}
#[test]
fn sound_queue_is_bounded_and_reports_eviction_without_advancing_paused_game() {
    let mut run = LevelRun::new(Assets::synthetic(), 1, None, Pacing::Reference).unwrap();
    run.session.queue(InputCommand::Pause).unwrap();
    for _ in 0..SOUND_EVENT_CAPACITY + 9 {
        run.session.notify_skill_selection();
        run.tick().unwrap();
    }
    assert_eq!(run.session.scheduler.step_state().iteration, 0);
    let batch = run.sound.drain();
    assert_eq!(batch.events.len(), SOUND_EVENT_CAPACITY);
    assert_eq!(batch.dropped, 9);
    assert_eq!(batch.events[0].sequence, 9);
    assert!(
        batch
            .events
            .iter()
            .all(|e| e.source.cue == SoundCue::SkillButtonSelect)
    );
    assert!(run.sound.drain().events.is_empty());
}
#[test]
fn result_stops_once_and_replacement_discards_old_queue_with_new_generation_and_music_policy() {
    let mut old = LevelRun::new(
        Assets::synthetic_completion("win"),
        1,
        Some(2),
        Pacing::Reference,
    )
    .unwrap();
    for _ in 0..1200 {
        old.tick().unwrap();
        if old.session.result().is_some() {
            break;
        }
    }
    assert!(old.session.result().unwrap().success);
    assert_eq!(
        old.sound.drain().events.last().unwrap().command,
        Some(AdlibCommand::Stop)
    );
    old.tick().unwrap();
    assert!(old.sound.drain().events.is_empty());
    let mut next = old
        .prepare(ResultAction::Continue, Pacing::Reference, |_| {
            Ok((Assets::synthetic(), None))
        })
        .unwrap()
        .unwrap();
    assert_eq!(next.sound.music_index(), 6);
    assert_eq!(next.sound.drain().generation, 1);
    let mut retry = old
        .prepare(ResultAction::Restart, Pacing::Reference, |_| {
            Ok((Assets::synthetic(), None))
        })
        .unwrap()
        .unwrap();
    assert_eq!(retry.sound.music_index(), 5);
    assert!(retry.sound.drain().reset);
    let before = old.sound.diagnostic_json();
    let mut bad = Assets::synthetic();
    bad.music_special = 5;
    assert!(
        old.prepare(ResultAction::Restart, Pacing::Reference, |_| Ok((
            bad, None
        )))
        .is_err()
    );
    assert_eq!(old.sound.diagnostic_json(), before);
}

#[test]
fn excessive_ui_before_one_tick_reports_drops_to_the_consumer_without_growing_the_queue() {
    let mut run = LevelRun::new(Assets::synthetic(), 1, None, Pacing::Reference).unwrap();
    run.session.queue(InputCommand::Pause).unwrap();
    for _ in 0..1100 {
        run.session.notify_skill_selection();
    }
    run.tick().unwrap();
    let batch = run.sound.drain();
    assert_eq!(batch.dropped, 76);
    assert_eq!(batch.events.len(), 1024);
    assert_eq!(run.session.scheduler.step_state().iteration, 0);
}
