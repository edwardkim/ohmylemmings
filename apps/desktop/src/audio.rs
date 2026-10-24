//! SDL device adapter; simulation and PCM synthesis never consult device time.
use super::{
    Options,
    local_music::{MusicMode, Stream, mix_frame},
};
use lemmings_game::{
    LevelRun,
    audio::{AudioOutput, decode_adlib, demo_image},
    sound::SoundBatch,
};
use sdl2::{
    Sdl,
    audio::{AudioQueue, AudioSpecDesired},
    event::Event,
    keyboard::Keycode,
};
use std::{error::Error, fs, path::PathBuf, time::Duration};
const CHUNK: usize = 512;
const TARGET: usize = 2048;
pub const MAX_QUEUED_FRAMES: usize = 3072;
pub struct AudioService {
    output: Option<AudioOutput>,
    queue: Option<AudioQueue<i16>>,
    pub enabled: bool,
    pub volume: u8,
    failed: bool,
    music_mode: MusicMode,
    music_path: Option<PathBuf>,
    music: Option<Stream>,
    music_volume: u8,
    effects_volume: u8,
    music_active: bool,
    music_generation: Option<u64>,
    music_error: Option<String>,
    pub submitted: u64,
    pub nonzero: u64,
}
impl AudioService {
    pub fn new(sdl: &Sdl, options: &Options) -> Self {
        let mut result = Self {
            output: None,
            queue: None,
            enabled: !options.fast && !options.mute,
            volume: options.volume,
            failed: false,
            music_mode: if options.bgm.is_some() {
                MusicMode::Custom
            } else {
                MusicMode::Original
            },
            music_path: options.bgm.clone(),
            music: None,
            music_volume: 100,
            effects_volume: 100,
            music_active: false,
            music_generation: None,
            music_error: None,
            submitted: 0,
            nonzero: 0,
        };
        if result.enabled {
            result.start(sdl, options);
        }
        result
    }
    fn start(&mut self, sdl: &Sdl, options: &Options) {
        self.silence();
        self.queue = None;
        self.output = None;
        let channels = if self.music_path.is_some() { 2 } else { 1 };
        let attempt = (|| -> Result<_, Box<dyn Error>> {
            let image = if options.synthetic {
                demo_image()
            } else {
                decode_adlib(&fs::read(options.assets.join("ADLIB.DAT"))?)?
            };
            let audio = sdl.audio()?;
            let queue = audio.open_queue::<i16, _>(
                None,
                &AudioSpecDesired {
                    freq: Some(44100),
                    channels: Some(channels),
                    samples: Some(512),
                },
            )?;
            if queue.spec().channels != channels {
                return Err("requested audio channel count unavailable".into());
            }
            let mut output = AudioOutput::new(image, queue.spec().freq as u32)?;
            output.set_volume(self.volume)?;
            Ok((queue, output))
        })();
        match attempt {
            Ok((queue, output)) => {
                self.queue = Some(queue);
                self.output = Some(output);
                self.failed = false;
                self.restart_music();
            }
            Err(error) => self.failure(error.to_string()),
        }
    }
    fn silence(&mut self) {
        self.music = None;
        self.music_active = false;
        if let Some(queue) = &self.queue {
            queue.pause();
            queue.clear();
        }
        if let Some(output) = &mut self.output {
            output.suspend();
        }
    }
    fn failure(&mut self, error: String) {
        self.silence();
        self.queue = None;
        self.output = None;
        self.enabled = false;
        self.failed = true;
        eprintln!("Audio unavailable: {error}. Game continues; S retries sound.");
    }
    pub fn label(&self) -> String {
        if self.failed {
            "Sound unavailable (S retries) | F4 choose MP3".into()
        } else if self.enabled {
            let mode = match self.music_mode {
                MusicMode::Original => "Original",
                MusicMode::Custom => "MP3",
                MusicMode::Off => "Off",
            };
            let state = if self.music_error.is_some() {
                "error: F7 retries"
            } else if self.music.as_ref().is_some_and(|m| !m.ready) {
                "loading"
            } else if self.music_active {
                "playing/loop"
            } else {
                "ready"
            };
            let file = self
                .music_path
                .as_ref()
                .and_then(|p| p.file_name())
                .map(|s| s.to_string_lossy())
                .unwrap_or_default();
            format!(
                "Sound {}% | {mode} BGM {}% FX {}% {file} {state} | F4 choose MP3 / F6 music",
                self.volume, self.music_volume, self.effects_volume
            )
        } else {
            "Sound off (S enables) | F4 choose MP3".into()
        }
    }
    pub fn input(&mut self, event: &Event, sdl: &Sdl, options: &Options) -> bool {
        if options.fast {
            return false;
        }
        if let Event::DropFile { filename, .. } = event {
            self.select_music(filename.into(), sdl, options);
            return true;
        }
        if let Event::KeyDown {
            keycode: Some(key),
            repeat: false,
            ..
        } = event
        {
            match *key {
                Keycode::F6 => {
                    self.music_mode = match self.music_mode {
                        MusicMode::Original => {
                            if self.music_path.is_some() {
                                MusicMode::Custom
                            } else {
                                MusicMode::Off
                            }
                        }
                        MusicMode::Custom => MusicMode::Off,
                        MusicMode::Off => MusicMode::Original,
                    };
                    self.music = None;
                    self.music_active = false;
                    self.restart_music();
                    self.clear_queue();
                    return true;
                }
                Keycode::F7 => {
                    self.restart_music();
                    self.clear_queue();
                    return true;
                }
                Keycode::F5 => {
                    self.music_path = None;
                    self.music = None;
                    self.music_mode = MusicMode::Original;
                    self.music_error = None;
                    self.music_active = false;
                    self.clear_queue();
                    return true;
                }
                Keycode::LeftBracket | Keycode::RightBracket | Keycode::Comma | Keycode::Period => {
                    let value = if matches!(*key, Keycode::LeftBracket | Keycode::RightBracket) {
                        &mut self.music_volume
                    } else {
                        &mut self.effects_volume
                    };
                    *value = if matches!(*key, Keycode::RightBracket | Keycode::Period) {
                        value.saturating_add(10).min(100)
                    } else {
                        value.saturating_sub(10)
                    };
                    self.clear_queue();
                    return true;
                }

                Keycode::S => {
                    self.enabled = !self.enabled;
                    if !self.enabled {
                        self.silence();
                    } else if self.queue.is_none()
                        || self.failed
                        || self.queue.as_ref().is_some_and(|q| {
                            q.spec().channels != if self.music_path.is_some() { 2 } else { 1 }
                        })
                    {
                        self.start(sdl, options);
                    }
                    return true;
                }
                Keycode::PageUp | Keycode::PageDown => {
                    self.volume = if *key == Keycode::PageUp {
                        self.volume.saturating_add(10).min(100)
                    } else {
                        self.volume.saturating_sub(10)
                    };
                    if let Some(output) = &mut self.output {
                        output.set_volume(self.volume).expect("bounded volume");
                    }
                    if let Some(queue) = &self.queue {
                        queue.clear();
                    }
                    return true;
                }
                _ => {}
            }
        }
        false
    }
    /// Opt-in measurement reads the real device queue before service/refill.
    pub fn performance_state(&self) -> Option<(i32, u16, u32, bool)> {
        self.queue.as_ref().map(|q| {
            (
                q.spec().freq,
                q.spec().samples,
                q.size() / (2 * u32::from(q.spec().channels)),
                q.status() == sdl2::audio::AudioStatus::Playing,
            )
        })
    }
    fn clear_queue(&self) {
        if let Some(queue) = &self.queue {
            queue.clear();
        }
    }
    fn restart_music(&mut self) {
        self.music_error = None;
        self.music = if self.music_mode == MusicMode::Custom {
            self.music_path.clone().map(|path| {
                Stream::open(
                    path,
                    self.queue.as_ref().map_or(44100, |q| q.spec().freq as u32),
                )
            })
        } else {
            None
        };
    }
    pub fn selected_music(&self) -> Option<&std::path::Path> {
        self.music_path.as_deref()
    }
    pub fn pause_for_dialog(&self) {
        if let Some(queue) = &self.queue {
            queue.pause();
            queue.clear();
        }
    }
    pub fn select_music(&mut self, path: PathBuf, sdl: &Sdl, options: &Options) {
        if !path
            .extension()
            .is_some_and(|s| s.eq_ignore_ascii_case("mp3"))
        {
            self.music_error = Some("Choose an MP3 file".into());
            return;
        }
        self.music_path = Some(path);
        self.music_mode = MusicMode::Custom;
        self.music_generation = None;
        if self.enabled {
            self.start(sdl, options);
        } else {
            self.restart_music();
        }
    }
    fn sync_music(&mut self, batch: &SoundBatch, active: bool) {
        let wanted = active && batch.music_playing && self.music_mode == MusicMode::Custom;
        if self.music_generation.is_some_and(|g| g != batch.generation)
            || (self.music_active && !wanted)
        {
            self.music = None;
            self.music_active = false;
        }
        self.music_generation = Some(batch.generation);
        if wanted && self.music.is_none() && self.music_error.is_none() {
            self.restart_music();
        }
        self.music_active = wanted;
        if let Some(music) = &mut self.music {
            music.poll();
            if let Some(error) = &music.error {
                self.music_error = Some(error.clone());
                self.music_active = false;
            }
        }
    }
    pub fn service(&mut self, game: &mut LevelRun, visible: bool) {
        let batch = game.sound.drain();
        let active = self.enabled && visible && game.session.result().is_none();
        if let Err(error) = self.deliver(&batch, active) {
            self.failure(error);
        }
    }
    fn deliver(&mut self, batch: &SoundBatch, active: bool) -> Result<(), String> {
        self.sync_music(batch, active);
        let (Some(output), Some(queue)) = (&mut self.output, &self.queue) else {
            return Ok(());
        };
        output.set_mix(
            if self.music_mode == MusicMode::Original {
                self.music_volume
            } else {
                0
            },
            self.effects_volume,
        )?;
        let flush = output.synchronize(batch, active)?;
        let channels = usize::from(queue.spec().channels);
        let frame_bytes = 2 * channels;
        if flush || queue.size() as usize / frame_bytes > MAX_QUEUED_FRAMES {
            queue.clear();
        }
        if !active {
            queue.pause();
            return Ok(());
        }
        // Fixed work and queue ceilings: a slow device/CPU cannot trap the game
        // in an unbounded refill loop or admit seconds of stale sound.
        for _ in 0..4 {
            if queue.size() as usize / frame_bytes >= TARGET {
                break;
            }
            let mut pcm = [0; CHUNK];
            output.render(&mut pcm)?;
            if channels == 2 {
                let mut stereo = [0_i16; CHUNK * 2];
                let gain = f32::from(self.volume) * f32::from(self.music_volume) / 100.;
                for (sample, target) in pcm.iter().zip(stereo.chunks_exact_mut(2)) {
                    let music = if self.music_active {
                        self.music.as_mut().map_or([0.; 2], Stream::next)
                    } else {
                        [0.; 2]
                    };
                    target.copy_from_slice(&mix_frame(*sample, music, gain));
                }
                self.nonzero += stereo
                    .chunks_exact(2)
                    .filter(|v| v.iter().any(|&s| s != 0))
                    .count() as u64;
                queue.queue_audio(&stereo)?;
            } else {
                self.nonzero += pcm.iter().filter(|&&s| s != 0).count() as u64;
                queue.queue_audio(&pcm)?;
            }
            self.submitted += CHUNK as u64;
        }
        queue.resume();
        Ok(())
    }
}
/// Original-free device probe uses the production service, not a parallel sink.
pub fn probe() -> Result<String, Box<dyn Error>> {
    let options = Options::parse(
        ["--synthetic", "--volume", "100"]
            .map(str::to_string)
            .into_iter(),
    )?
    .unwrap();
    let sdl = sdl2::init()?;
    let audio = sdl.audio()?;
    let driver = audio.current_audio_driver();
    let mut service = AudioService::new(&sdl, &options);
    if service.failed {
        return Err("audio device unavailable".into());
    }
    let mut game = LevelRun::new(
        lemmings_game::Assets::synthetic_builder(),
        1,
        None,
        lemmings_game::pacing::Pacing::Reference,
    )?;
    service.service(&mut game, true);
    for _ in 0..55 {
        game.tick()?;
    }
    service.queue.as_ref().unwrap().clear();
    service.service(&mut game, true);
    let queue = service.queue.as_ref().unwrap();
    let before = queue.size();
    assert!(before > 0 && before as usize / 2 <= MAX_QUEUED_FRAMES && service.nonzero > 0);
    let mut after = before;
    for _ in 0..25 {
        std::thread::sleep(Duration::from_millis(10));
        after = queue.size();
        if after < before {
            break;
        }
    }
    assert!(after < before, "device did not consume queued PCM");
    service.enabled = false;
    service.service(&mut game, true);
    assert_eq!(service.queue.as_ref().unwrap().size(), 0);
    service.enabled = true;
    service.service(&mut game, true);
    service.service(&mut game, false);
    assert_eq!(service.queue.as_ref().unwrap().size(), 0);
    game.sound.fresh_generation_after(0)?;
    service.service(&mut game, true);
    assert!(service.queue.as_ref().unwrap().size() as usize / 2 <= MAX_QUEUED_FRAMES);
    service.silence();
    Ok(format!(
        "{{\"status\":\"PASS\",\"driver\":{driver:?},\"queued_bytes\":{before},\"consumed_bytes\":{},\"cleared_bytes\":0,\"queue_limit_frames\":{MAX_QUEUED_FRAMES},\"nonzero_frames\":{},\"submitted_frames\":{},\"mute_hide_generation\":true}}",
        before - after,
        service.nonzero,
        service.submitted
    ))
}

/// Exercise the production stereo mixer/device queue using our composed fixture.
pub fn local_probe(fixture: Option<PathBuf>) -> Result<String, Box<dyn Error>> {
    let sdl = sdl2::init()?;
    let options = Options::parse(
        ["--synthetic", "--volume", "100"]
            .into_iter()
            .map(str::to_owned),
    )?
    .unwrap();
    let mut game = LevelRun::new(
        lemmings_game::Assets::synthetic_builder(),
        1,
        None,
        lemmings_game::pacing::Pacing::Reference,
    )?;
    let mut service = AudioService::new(&sdl, &options);
    let path = fixture.unwrap_or_else(|| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/local-tone.mp3")
    });
    service.select_music(path, &sdl, &options);
    if service
        .queue
        .as_ref()
        .is_none_or(|q| q.spec().channels != 2)
    {
        return Err("stereo device unavailable".into());
    }
    for _ in 0..55 {
        game.tick()?;
    }
    let mut peak = 0i16;
    let mut stereo = false;
    let mut consumed = false;
    for _ in 0..80 {
        service.service(&mut game, true);
        if service.music_error.is_some() {
            return Err(service.music_error.clone().unwrap().into());
        }
        if let Some(music) = &mut service.music {
            for _ in 0..64 {
                let v = mix_frame(0, music.next(), 100.);
                peak = peak.max(v[0].saturating_abs());
                stereo |= v[0] != v[1];
            }
        }
        let before = service.queue.as_ref().unwrap().size();
        std::thread::sleep(Duration::from_millis(5));
        consumed |= service.queue.as_ref().unwrap().size() < before;
    }
    let selected = service.music_path.clone();
    let mode = service.music_mode;
    service.pause_for_dialog();
    if service.queue.as_ref().unwrap().status() != sdl2::audio::AudioStatus::Paused
        || service.queue.as_ref().unwrap().size() != 0
        || service.music_path != selected
        || service.music_mode != mode
        || service.music.is_none()
    {
        return Err("dialog pause changed music selection or failed to silence device".into());
    }
    service.service(&mut game, true);
    if service.queue.as_ref().unwrap().status() != sdl2::audio::AudioStatus::Playing {
        return Err("dialog return did not resume audio".into());
    }
    service.service(&mut game, false);
    if peak == 0 || !stereo || !consumed || service.queue.as_ref().unwrap().size() != 0 {
        return Err("local MP3 stereo/queue lifecycle failed".into());
    }
    service.service(&mut game, true);
    service.music_mode = MusicMode::Off;
    service.clear_queue();
    service.service(&mut game, true);
    service.music_mode = MusicMode::Original;
    service.service(&mut game, true);
    Ok(format!(
        "{{\"status\":\"PASS\",\"stereo\":true,\"peak\":{peak},\"device_consumed\":true,\"hide_clears_queue\":true,\"modes\":true,\"dialog_pause_preserves_selection\":true}}"
    ))
}
