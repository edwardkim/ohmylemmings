//! Opt-in monotonic observations of the production SDL loop. No per-sample I/O.
use crate::{LevelRun, Pacing};
use std::{
    error::Error,
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
pub struct Recorder {
    path: PathBuf,
    started: Instant,
    limit: Duration,
    period_ns: u128,
    ticks: Vec<[u128; 5]>,
    frames: Vec<[u128; 9]>,
    audio: Vec<[u128; 6]>,
    last_frame: Option<u128>,
    underruns: u64,
    pub upload_samples: Vec<u128>,
}
impl Recorder {
    pub fn new(path: &Path, seconds: f64, pacing: Pacing) -> Result<Self, Box<dyn Error>> {
        if path.exists() || !seconds.is_finite() || !(0.1..=600.0).contains(&seconds) {
            return Err("new performance path and 0.1..600 seconds required".into());
        }
        let period_ns = match pacing {
            Pacing::Dos => 1_000_000_000u128 * 156250 / 2190197,
            Pacing::Reference => 1_000_000_000 / 17,
        };
        Ok(Self {
            path: path.into(),
            started: Instant::now(),
            limit: Duration::from_secs_f64(seconds),
            period_ns,
            ticks: Vec::with_capacity((seconds * 20.0).ceil() as usize),
            frames: Vec::with_capacity((seconds * 120.0).ceil() as usize),
            audio: Vec::with_capacity((seconds * 1200.0).ceil() as usize),
            last_frame: None,
            underruns: 0,
            upload_samples: Vec::new(),
        })
    }
    pub fn done(&self) -> bool {
        self.started.elapsed() >= self.limit
    }
    pub fn tick(&mut self, begin: Instant, step: u64, alive: usize) {
        let at = begin.duration_since(self.started).as_nanos();
        self.ticks.push([
            at,
            u128::from(step),
            begin.elapsed().as_nanos(),
            at.saturating_sub(u128::from(step) * self.period_ns),
            alive as u128,
        ]);
    }
    pub fn frame(
        &mut self,
        begin: Instant,
        cpu: u128,
        upload: u128,
        present: u128,
        budget: Option<crate::frame_clock::FrameBudget>,
        skipped: u128,
    ) {
        let at = begin.duration_since(self.started).as_nanos();
        let gap = self.last_frame.map_or(0, |p| at - p);
        self.last_frame = Some(at);
        self.frames.push([
            at,
            gap,
            cpu,
            upload,
            present,
            budget.map_or(0, |b| {
                begin.saturating_duration_since(b.scheduled).as_nanos()
            }),
            budget.map_or(0, |b| {
                Instant::now()
                    .saturating_duration_since(b.deadline)
                    .as_nanos()
            }),
            skipped,
            u128::from(budget.is_some()),
        ]);
    }
    pub fn audio(&mut self, sample: Option<(i32, u16, u32, bool)>, cost: u128) {
        if let Some((rate, quantum, queued, playing)) = sample {
            if playing && queued == 0 {
                self.underruns += 1;
            }
            self.audio.push([
                self.started.elapsed().as_nanos(),
                queued as u128,
                u128::from(playing),
                rate as u128,
                u128::from(quantum),
                cost,
            ]);
        }
    }
    pub fn finish(
        &self,
        game: &LevelRun,
        driver: &str,
        renderer: &str,
    ) -> Result<(), Box<dyn Error>> {
        let elapsed = self.started.elapsed().as_secs_f64();
        let presentation_base_fps = crate::frame_clock::TARGET_FPS;
        let memory = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
        let peak = memory
            .lines()
            .find(|l| l.starts_with("VmHWM:"))
            .and_then(|l| l.split_whitespace().nth(1))
            .unwrap_or("0");
        let observer_bytes =
            (self.ticks.capacity() * 5 + self.frames.capacity() * 9 + self.audio.capacity() * 6)
                * std::mem::size_of::<u128>();
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&self.path)?;
        writeln!(
            file,
            "{{\"process_peak_rss_kib\":{peak},\"presentation_base_fps\":{presentation_base_fps},\"observer_capacity_bytes\":{observer_bytes},\"upload_operations\":32,\"upload_samples_ns\":{:?},\"seconds\":{},\"required_seconds\":{},\"video_driver\":{driver:?},\"renderer\":{renderer:?},\"tick_period_ns\":{},\"ticks\":{:?},\"frames\":{:?},\"audio\":{:?},\"poll_detected_underruns\":{},\"state\":{}}}",
            self.upload_samples,
            elapsed,
            self.limit.as_secs_f64(),
            self.period_ns,
            self.ticks,
            self.frames,
            self.audio,
            self.underruns,
            lemmings_game::diagnostics::state(&game.session)
        )?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_duration_and_existing_output_are_rejected() {
        let path = std::env::temp_dir().join("lemmings-perf-unwritten");
        for seconds in [0.0, -1.0, f64::NAN, 601.0] {
            assert!(Recorder::new(&path, seconds, Pacing::Dos).is_err());
        }
        assert!(
            Recorder::new(
                &Path::new(env!("CARGO_MANIFEST_DIR")).join("src/performance.rs"),
                1.0,
                Pacing::Dos
            )
            .is_err()
        );
    }
}
