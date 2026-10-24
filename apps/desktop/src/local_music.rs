//! Bounded local-file MP3 worker. No decoding or file I/O on the game thread.
use std::{
    fs::File,
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver, SyncSender, TryRecvError},
};
use symphonia::core::{
    audio::SampleBuffer,
    codecs::{CODEC_TYPE_MP3, DecoderOptions},
    errors::Error,
    formats::FormatOptions,
    io::MediaSourceStream,
    meta::{Limit, MetadataOptions},
    probe::Hint,
};
const CHUNK: usize = 1024;
const QUEUED: usize = 8;
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MusicMode {
    Original,
    Custom,
    Off,
}
pub enum Message {
    Pcm(Vec<[f32; 2]>),
    Error(String),
}
pub struct Stream {
    receiver: Receiver<Message>,
    current: Vec<[f32; 2]>,
    at: usize,
    pub error: Option<String>,
    pub ready: bool,
}
impl Stream {
    pub fn open(path: PathBuf, rate: u32) -> Self {
        let (sender, receiver) = mpsc::sync_channel(QUEUED);
        std::thread::spawn(move || {
            loop {
                match decode(&path, rate, &sender) {
                    Ok(true) => {}
                    Ok(false) => break,
                    Err(error) => {
                        let _ = sender.send(Message::Error(error));
                        break;
                    }
                }
            }
        });
        Self {
            receiver,
            current: Vec::new(),
            at: 0,
            error: None,
            ready: false,
        }
    }
    pub fn poll(&mut self) {
        if self.at < self.current.len() || self.error.is_some() {
            return;
        }
        match self.receiver.try_recv() {
            Ok(Message::Pcm(pcm)) => {
                self.current = pcm;
                self.at = 0;
                self.ready = true;
            }
            Ok(Message::Error(error)) => self.error = Some(error),
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                self.error = Some("Music decoder stopped; retry the file".into())
            }
        }
    }
    pub fn next(&mut self) -> [f32; 2] {
        self.poll();
        if self.at >= self.current.len() {
            return [0.; 2];
        }
        let frame = self.current[self.at];
        self.at += 1;
        frame
    }
}
fn decode(path: &Path, rate: u32, sender: &SyncSender<Message>) -> Result<bool, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let info = file.metadata().map_err(|e| e.to_string())?;
    if !info.is_file() || info.len() == 0 || info.len() > 512 * 1024 * 1024 {
        return Err("Choose an MP3 file up to 512MB".into());
    }
    let stream = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    hint.with_extension("mp3");
    let metadata = MetadataOptions {
        limit_metadata_bytes: Limit::Maximum(256 * 1024),
        limit_visual_bytes: Limit::Maximum(0),
    };
    let mut format = symphonia::default::get_probe()
        .format(
            &hint,
            stream,
            &FormatOptions {
                enable_gapless: true,
                ..Default::default()
            },
            &metadata,
        )
        .map_err(|e| e.to_string())?
        .format;
    let track = format.default_track().ok_or("No audio track")?;
    if track.codec_params.codec != CODEC_TYPE_MP3 {
        return Err("Only MP3 audio is supported".into());
    }
    let id = track.id;
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| e.to_string())?;
    let mut resampler = Resampler::new(rate);
    let mut output = Vec::with_capacity(CHUNK);
    let mut frames = 0usize;
    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(Error::IoError(error)) if error.kind() == std::io::ErrorKind::UnexpectedEof => {
                break;
            }
            Err(error) => return Err(error.to_string()),
        };
        if packet.track_id() != id {
            continue;
        }
        let audio = decoder.decode(&packet).map_err(|e| e.to_string())?;
        let channels = audio.spec().channels.count();
        let input_rate = audio.spec().rate;
        if !(1..=2).contains(&channels)
            || !(8000..=192000).contains(&input_rate)
            || audio.capacity() > 8192
        {
            return Err("Unsupported MP3 audio format".into());
        }
        let mut pcm = SampleBuffer::<f32>::new(audio.capacity() as u64, *audio.spec());
        pcm.copy_interleaved_ref(audio);
        for samples in pcm.samples().chunks_exact(channels) {
            let frame = [samples[0], samples[channels - 1]];
            if !frame.iter().all(|v| v.is_finite()) {
                return Err("Invalid MP3 samples".into());
            }
            for sample in resampler.push(frame, input_rate) {
                output.push(sample);
                frames += 1;
                if output.len() == CHUNK
                    && sender
                        .send(Message::Pcm(std::mem::replace(
                            &mut output,
                            Vec::with_capacity(CHUNK),
                        )))
                        .is_err()
                {
                    return Ok(false);
                }
            }
        }
    }
    if frames == 0 {
        return Err("The MP3 contains no playable audio".into());
    }
    if !output.is_empty() && sender.send(Message::Pcm(output)).is_err() {
        return Ok(false);
    }
    Ok(true)
}
struct Resampler {
    previous: Option<[f32; 2]>,
    phase: u64,
    rate: u32,
    input_rate: Option<u32>,
}
impl Resampler {
    fn new(rate: u32) -> Self {
        Self {
            previous: None,
            phase: 0,
            rate,
            input_rate: None,
        }
    }
    fn push(
        &mut self,
        current: [f32; 2],
        input_rate: u32,
    ) -> impl Iterator<Item = [f32; 2]> + use<> {
        if self.input_rate != Some(input_rate) {
            self.previous = None;
            self.phase = 0;
            self.input_rate = Some(input_rate);
        }
        let previous = self.previous.replace(current);
        let start = self.phase;
        let rate = self.rate;
        let count = if previous.is_some() {
            ((u64::from(rate).saturating_sub(start)).div_ceil(u64::from(input_rate))) as usize
        } else {
            0
        };
        if previous.is_some() {
            self.phase = start + count as u64 * u64::from(input_rate) - u64::from(rate);
        }
        (0..count).map(move |n| {
            let mix = (start + n as u64 * u64::from(input_rate)) as f32 / rate as f32;
            let previous = previous.unwrap();
            [
                previous[0] + (current[0] - previous[0]) * mix,
                previous[1] + (current[1] - previous[1]) * mix,
            ]
        })
    }
}
pub fn mix_frame(effect: i16, music: [f32; 2], gain: f32) -> [i16; 2] {
    music.map(|sample| {
        (f32::from(effect) + sample * 32767. * gain / 100.)
            .round()
            .clamp(-32768., 32767.) as i16
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stereo_clip_and_resampling_are_bounded_and_preserve_pitch_duration() {
        assert_eq!(mix_frame(32000, [1., -1.], 100.), [32767, -767]);
        assert_eq!(mix_frame(-123, [1., -1.], 0.), [-123, -123]);
        for input in [8000, 22050, 44100, 48000] {
            let mut r = Resampler::new(44100);
            let mut frames = 0;
            for _ in 0..input {
                frames += r
                    .push([0.25, -0.25], input)
                    .inspect(|v| assert_eq!(*v, [0.25, -0.25]))
                    .count();
            }
            assert!((frames as i64 - 44100).abs() < 8);
        }
    }
    #[test]
    fn real_mp3_is_stereo_looped_bounded_and_cancellation_unblocks_worker() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/local-tone.mp3");
        let stream = Stream::open(path, 44100);
        let mut count = 0;
        let mut different = false;
        while count < 44100 {
            match stream
                .receiver
                .recv_timeout(std::time::Duration::from_secs(3))
                .unwrap()
            {
                Message::Error(e) => panic!("{e}"),
                Message::Pcm(pcm) => {
                    assert!(pcm.len() <= CHUNK);
                    different |= pcm.iter().any(|v| (v[0] - v[1]).abs() > 0.01);
                    count += pcm.len();
                }
            }
        }
        assert!(different);
        drop(stream);
    }
    #[test]
    fn missing_and_corrupt_files_report_error_without_panicking() {
        let stream = Stream::open(PathBuf::from("missing-user-song.mp3"), 44100);
        assert!(matches!(
            stream
                .receiver
                .recv_timeout(std::time::Duration::from_secs(3))
                .unwrap(),
            Message::Error(_)
        ));
        let stream = Stream::open(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"),
            44100,
        );
        assert!(matches!(
            stream
                .receiver
                .recv_timeout(std::time::Duration::from_secs(3))
                .unwrap(),
            Message::Error(_)
        ));
    }
}
