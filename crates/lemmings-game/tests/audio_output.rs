use lemmings_core::SoundCue;
use lemmings_game::audio::{AudioOutput, decode_adlib, demo_image};
use lemmings_game::sound::{SoundBatch, SoundEvent};
use lemmings_resources::AdlibCommand;
use lemmings_session::SessionSound;
fn batch(generation: u64, sequence: u64, command: Option<AdlibCommand>) -> SoundBatch {
    SoundBatch {
        generation,
        reset: false,
        dropped: 0,
        sequence_exhausted: false,
        music_index: 5,
        music_playing: true,
        events: command
            .map(|command| {
                vec![SoundEvent {
                    sequence,
                    source: SessionSound {
                        control_step: 0,
                        iteration: 0,
                        sequence: 0,
                        input_sequence: None,
                        actor_id: None,
                        cue: SoundCue::MusicStart,
                    },
                    command: Some(command),
                    raw_trap: None,
                    mapping_error: None,
                }]
            })
            .unwrap_or_default(),
    }
}
#[test]
fn activation_and_generation_resync_current_music_without_replaying_old_effects() {
    let mut a = AudioOutput::new(demo_image(), 44100).unwrap();
    let mut b = AudioOutput::new(demo_image(), 44100).unwrap();
    assert!(
        a.synchronize(&batch(0, 0, Some(AdlibCommand::Effect(7))), true)
            .unwrap()
    );
    b.synchronize(&batch(0, 0, None), true).unwrap();
    let mut x = [0; 4096];
    let mut y = [0; 4096];
    a.render(&mut x).unwrap();
    b.render(&mut y).unwrap();
    assert_eq!(x, y);
    assert!(x.iter().any(|&v| v != 0));
    a.synchronize(&batch(1, 0, None), true).unwrap();
    assert_eq!(a.frames(), 0);
}
#[test]
fn mute_hide_and_lost_batches_silence_and_resume_with_bounded_fresh_pcm() {
    let mut a = AudioOutput::new(demo_image(), 48000).unwrap();
    a.synchronize(&batch(0, 0, None), true).unwrap();
    let mut x = [0; 1024];
    a.render(&mut x).unwrap();
    let frames = a.frames();
    assert!(
        a.synchronize(&batch(0, 1, Some(AdlibCommand::Effect(2))), false)
            .unwrap()
    );
    a.render(&mut x).unwrap();
    assert_eq!(x, [0; 1024]);
    assert_eq!(a.frames(), frames);
    assert!(a.synchronize(&batch(0, 2, None), true).unwrap());
    assert_eq!(a.frames(), 0);
    let mut lost = batch(0, 3, None);
    lost.dropped = 9;
    assert!(a.synchronize(&lost, true).unwrap());
    assert!(a.render(&mut [1; 8193]).is_err());
}
#[test]
fn invalid_assets_volume_stale_generation_and_duplicate_commands_preserve_playback() {
    assert!(AudioOutput::new(vec![], 44100).is_err());
    assert!(AudioOutput::new(demo_image(), 0).is_err());
    assert!(decode_adlib(&[]).is_err());
    let mut a = AudioOutput::new(demo_image(), 44100).unwrap();
    let mut b = AudioOutput::new(demo_image(), 44100).unwrap();
    for p in [&mut a, &mut b] {
        p.set_volume(100).unwrap();
        p.synchronize(&batch(1, 2, Some(AdlibCommand::Music(5))), true)
            .unwrap();
    }
    assert!(a.set_volume(101).is_err());
    assert!(a.synchronize(&batch(0, 3, None), true).is_err());
    assert!(
        a.synchronize(&batch(1, 2, Some(AdlibCommand::Effect(0))), true)
            .is_err()
    );
    assert!(
        a.synchronize(&batch(1, 3, Some(AdlibCommand::Effect(18))), true)
            .is_err()
    );
    let mut x = [0; 4096];
    let mut y = [0; 4096];
    a.render(&mut x).unwrap();
    b.render(&mut y).unwrap();
    assert_eq!(x, y);
}
#[test]
fn volume_scaling_and_output_partitioning_are_exact_and_zero_is_silent() {
    let mut a = AudioOutput::new(demo_image(), 44100).unwrap();
    let mut b = AudioOutput::new(demo_image(), 44100).unwrap();
    a.set_volume(100).unwrap();
    b.set_volume(33).unwrap();
    for p in [&mut a, &mut b] {
        p.synchronize(&batch(0, 0, None), true).unwrap();
    }
    let mut x = [0; 4096];
    let mut y = [0; 4096];
    a.render(&mut x).unwrap();
    for c in y.chunks_mut(137) {
        b.render(c).unwrap();
    }
    assert_eq!(y, x.map(|v| (i32::from(v) * 33 / 100) as i16));
    a.set_volume(0).unwrap();
    a.render(&mut x).unwrap();
    assert_eq!(x, [0; 4096]);
}

#[test]
fn music_can_be_silent_while_all_effects_and_the_driver_clock_continue() {
    for effect in 0..18 {
        let mut output = AudioOutput::new(demo_image(), 44100).unwrap();
        output.set_mix(0, 100).unwrap();
        output.set_volume(100).unwrap();
        output.synchronize(&batch(0, 0, None), true).unwrap();
        let mut samples = [0; 4096];
        output.render(&mut samples).unwrap();
        assert!(samples.iter().all(|&v| v == 0), "original music leaked");
        output
            .synchronize(&batch(0, 1, Some(AdlibCommand::Effect(effect))), true)
            .unwrap();
        output.render(&mut samples).unwrap();
        assert!(
            samples.iter().any(|&v| v != 0),
            "effect {effect} was silenced"
        );
        assert_eq!(output.frames(), 8192);
    }
}

#[test]
fn independent_gains_preserve_default_pcm_and_reject_invalid_changes_atomically() {
    let mut unchanged = AudioOutput::new(demo_image(), 48000).unwrap();
    let mut explicit = AudioOutput::new(demo_image(), 48000).unwrap();
    explicit.set_mix(100, 100).unwrap();
    for output in [&mut unchanged, &mut explicit] {
        output.synchronize(&batch(0, 0, None), true).unwrap();
    }
    let mut a = [0; 4096];
    let mut b = [0; 4096];
    assert!(explicit.set_mix(101, 0).is_err());
    assert!(explicit.set_mix(0, 101).is_err());
    unchanged.render(&mut a).unwrap();
    explicit.render(&mut b).unwrap();
    assert_eq!(a, b);
    explicit.set_mix(0, 0).unwrap();
    explicit.render(&mut b).unwrap();
    // The bounded native/resampler pipeline drains before absolute silence.
    explicit.render(&mut b).unwrap();
    assert!(b.iter().all(|&v| v == 0));
    assert_eq!(explicit.frames(), 12288);
}

#[test]
fn digging_whoosh_is_audible_chunk_independent_and_respects_effects_volume() {
    let mut a = AudioOutput::new(demo_image(), 48000).unwrap();
    let mut initial = batch(7, 0, None);
    initial.music_playing = false;
    a.synchronize(&initial, true).unwrap();
    let mut stroke = batch(7, 1, Some(AdlibCommand::Stop));
    stroke.music_playing = false;
    stroke.events[0].command = None;
    stroke.events[0].source.cue = SoundCue::DigStroke;
    a.synchronize(&stroke, true).unwrap();
    let mut pcm = [0; 8192];
    a.render(&mut pcm).unwrap();
    assert!(pcm.iter().any(|&v| v != 0));
    assert!(pcm[6000..].iter().all(|&v| v == 0));
    let mut b = AudioOutput::new(demo_image(), 48000).unwrap();
    b.synchronize(&initial, true).unwrap();
    b.synchronize(&stroke, true).unwrap();
    let mut split = [0; 8192];
    for chunk in split.chunks_mut(127) {
        b.render(chunk).unwrap();
    }
    assert_eq!(pcm, split);
    let mut muted = AudioOutput::new(demo_image(), 48000).unwrap();
    muted.set_mix(100, 0).unwrap();
    muted.synchronize(&initial, true).unwrap();
    muted.synchronize(&stroke, true).unwrap();
    muted.render(&mut pcm).unwrap();
    assert!(pcm.iter().all(|&v| v == 0));
}
