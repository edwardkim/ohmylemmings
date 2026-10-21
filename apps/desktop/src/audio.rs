//! SDL device adapter; simulation and PCM synthesis never consult device time.
use super::Options;
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
use std::{error::Error, fs, time::Duration};
const CHUNK: usize = 512;
const TARGET: usize = 2048;
pub const MAX_QUEUED_FRAMES: usize = 3072;
pub struct AudioService {
    output: Option<AudioOutput>,
    queue: Option<AudioQueue<i16>>,
    pub enabled: bool,
    pub volume: u8,
    failed: bool,
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
            submitted: 0,
            nonzero: 0,
        };
        if result.enabled {
            result.start(sdl, options);
        }
        result
    }
    fn start(&mut self, sdl: &Sdl, options: &Options) {
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
                    channels: Some(1),
                    samples: Some(512),
                },
            )?;
            if queue.spec().channels != 1 {
                return Err("mono audio device required".into());
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
            }
            Err(error) => self.failure(error.to_string()),
        }
    }
    fn silence(&mut self) {
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
            "Sound unavailable (S retries)".into()
        } else if self.enabled {
            format!("Sound {}%", self.volume)
        } else {
            "Sound off (S enables)".into()
        }
    }
    pub fn input(&mut self, event: &Event, sdl: &Sdl, options: &Options) -> bool {
        if options.fast {
            return false;
        }
        if let Event::KeyDown {
            keycode: Some(key),
            repeat: false,
            ..
        } = event
        {
            match *key {
                Keycode::S => {
                    self.enabled = !self.enabled;
                    if !self.enabled {
                        self.silence();
                    } else if self.queue.is_none() || self.failed {
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
                q.size() / 2,
                q.status() == sdl2::audio::AudioStatus::Playing,
            )
        })
    }
    pub fn service(&mut self, game: &mut LevelRun, visible: bool) {
        let batch = game.sound.drain();
        let active = self.enabled && visible && game.session.result().is_none();
        if let Err(error) = self.deliver(&batch, active) {
            self.failure(error);
        }
    }
    fn deliver(&mut self, batch: &SoundBatch, active: bool) -> Result<(), String> {
        let (Some(output), Some(queue)) = (&mut self.output, &self.queue) else {
            return Ok(());
        };
        let flush = output.synchronize(batch, active)?;
        if flush || queue.size() as usize / 2 > MAX_QUEUED_FRAMES {
            queue.clear();
        }
        if !active {
            queue.pause();
            return Ok(());
        }
        // Fixed work and queue ceilings: a slow device/CPU cannot trap the game
        // in an unbounded refill loop or admit seconds of stale sound.
        for _ in 0..4 {
            if queue.size() as usize / 2 >= TARGET {
                break;
            }
            let mut pcm = [0; CHUNK];
            output.render(&mut pcm)?;
            queue.queue_audio(&pcm)?;
            self.submitted += CHUNK as u64;
            self.nonzero += pcm.iter().filter(|&&s| s != 0).count() as u64;
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
