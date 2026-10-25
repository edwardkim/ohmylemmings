//! Optional player feedback. Measures SDL present completion cadence, not scanout.

use sdl2::{
    pixels::PixelFormatEnum,
    rect::Rect,
    render::{Canvas, Texture},
    video::Window,
};
use std::time::{Duration, Instant};

const WIDTH: usize = 160;
const HEIGHT: usize = 28;

struct Samples {
    gaps: [f64; 240],
    count: usize,
    cursor: usize,
    last: Option<Instant>,
    intervals: u64,
    elapsed: f64,
    slow: u64,
}
impl Default for Samples {
    fn default() -> Self {
        Self {
            gaps: [0.; 240],
            count: 0,
            cursor: 0,
            last: None,
            intervals: 0,
            elapsed: 0.,
            slow: 0,
        }
    }
}
struct Summary {
    fps: Option<f64>,
    p95_ms: Option<f64>,
    max_ms: Option<f64>,
}
impl Samples {
    fn hold(&mut self) {
        self.last = None;
    }
    fn frame(&mut self, now: Instant) {
        let previous = self.last.replace(now);
        let Some(previous) = previous else {
            return;
        };
        let gap = now.saturating_duration_since(previous).as_secs_f64() * 1000.;
        if gap == 0. {
            return;
        }
        self.gaps[self.cursor] = gap;
        self.cursor = (self.cursor + 1) % 240;
        self.count = (self.count + 1).min(240);
        self.intervals += 1;
        self.elapsed += gap;
        if gap > 50. {
            self.slow += 1;
        }
    }
    fn summary(&self) -> Summary {
        if self.count == 0 {
            return Summary {
                fps: None,
                p95_ms: None,
                max_ms: None,
            };
        }
        let mut gaps = self.gaps;
        gaps[..self.count].sort_unstable_by(f64::total_cmp);
        Summary {
            fps: Some(self.count as f64 * 1000. / gaps[..self.count].iter().sum::<f64>()),
            p95_ms: Some(gaps[(self.count * 95).div_ceil(100) - 1]),
            max_ms: Some(gaps[self.count - 1]),
        }
    }
}

pub struct Hud {
    pub enabled: bool,
    active: bool,
    samples: Samples,
    updated: Option<Instant>,
    dirty: bool,
    pixels: Vec<u8>,
    message: &'static str,
}
impl Default for Hud {
    fn default() -> Self {
        Self {
            enabled: false,
            active: false,
            samples: Samples::default(),
            updated: None,
            dirty: true,
            pixels: vec![0; WIDTH * HEIGHT * 4],
            message: "F3 HIDE F2 COPY",
        }
    }
}
fn number(value: Option<f64>) -> String {
    value.map_or_else(|| "--".into(), |v| format!("{v:.1}"))
}
impl Hud {
    pub fn reset(&mut self) {
        self.samples = Samples::default();
        self.updated = None;
        self.dirty = true;
        self.message = "F3 HIDE F2 COPY";
    }
    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
        self.reset();
    }
    pub fn set_active(&mut self, active: bool) {
        if self.active != active {
            self.active = active;
            self.samples.hold();
            self.dirty = true;
        }
    }
    pub fn frame(&mut self, now: Instant) {
        if self.enabled && self.active {
            self.samples.frame(now);
        }
    }
    pub fn copied(&mut self, success: bool) {
        self.message = if success {
            "COPIED - ADD YOUR FEEDBACK"
        } else {
            "COPY FAILED - TRY F2 AGAIN"
        };
        self.dirty = true;
    }
    pub fn feedback(&self, level: &str, hd: bool, size: (u32, u32), runtime: &str) -> String {
        let s = self.samples.summary();
        let presentation_base_fps = crate::frame_clock::TARGET_FPS;
        format!(
            "Oh My Lemmings performance feedback\nversion: {}\nplatform: Native {}/{}\nlevel: {level}\npresentation: {}\nviewport: {}x{}\nruntime: {runtime}\npresentation_base_fps: {presentation_base_fps}\ncadence: SDL present completion (not physical display FPS)\nwindow: last 240 active intervals\nfps: {}\np95_ms: {}\nmax_ms: {}\nrecent_intervals: {}\nintervals: {}\nactive_seconds: {:.3}\nslow_intervals (>50ms): {}\n체감: 부드러움 / 끊김이 느껴진 상황을 적어주세요.\n",
            env!("CARGO_PKG_VERSION"),
            std::env::consts::OS,
            std::env::consts::ARCH,
            if hd { "HD" } else { "original" },
            size.0,
            size.1,
            number(s.fps),
            number(s.p95_ms),
            number(s.max_ms),
            self.samples.count,
            self.samples.intervals,
            self.samples.elapsed / 1000.,
            self.samples.slow
        )
    }
    pub fn draw(
        &mut self,
        canvas: &mut Canvas<Window>,
        texture: &mut Texture<'_>,
        hd: bool,
    ) -> Result<(), String> {
        if !self.enabled {
            return Ok(());
        }
        let now = Instant::now();
        if self.dirty
            || self
                .updated
                .is_none_or(|last| now.duration_since(last) >= Duration::from_millis(500))
        {
            let s = self.samples.summary();
            let lines = [
                format!(
                    "PRESENT {} FPS{}",
                    number(s.fps),
                    if self.active { "" } else { " [HELD]" }
                ),
                format!("P95 {} MAX {} MS", number(s.p95_ms), number(s.max_ms)),
                format!(">50MS {}  {}", self.samples.slow, self.message),
            ];
            for pixel in self.pixels.chunks_exact_mut(4) {
                pixel.copy_from_slice(&[8, 18, 22, 235]);
            }
            for (row, line) in lines.iter().enumerate() {
                for (col, c) in line.chars().take(39).enumerate() {
                    for (y, bits) in glyph(c).iter().enumerate() {
                        for x in 0..3 {
                            if bits & (1 << (2 - x)) != 0 {
                                let at = ((row * 8 + y + 3) * WIDTH + col * 4 + x + 2) * 4;
                                self.pixels[at..at + 4].copy_from_slice(&[209, 245, 182, 255]);
                            }
                        }
                    }
                }
            }
            debug_assert_eq!(texture.query().format, PixelFormatEnum::RGBA32);
            texture
                .update(None, &self.pixels, WIDTH * 4)
                .map_err(|e| e.to_string())?;
            self.updated = Some(now);
            self.dirty = false;
        }
        let scale = if hd { 6 } else { 1 };
        canvas.copy(
            texture,
            None,
            Rect::new(
                4 * scale,
                4 * scale,
                WIDTH as u32 * scale as u32,
                HEIGHT as u32 * scale as u32,
            ),
        )
    }
}

// Own tiny ASCII HUD glyphs; cached texture, one SDL copy per visible frame.
fn glyph(c: char) -> [u8; 5] {
    match c {
        '0' => [7, 5, 5, 5, 7],
        '1' => [2, 6, 2, 2, 7],
        '2' => [7, 1, 7, 4, 7],
        '3' => [7, 1, 7, 1, 7],
        '4' => [5, 5, 7, 1, 1],
        '5' => [7, 4, 7, 1, 7],
        '6' => [7, 4, 7, 5, 7],
        '7' => [7, 1, 1, 1, 1],
        '8' => [7, 5, 7, 5, 7],
        '9' => [7, 5, 7, 1, 7],
        'A' => [2, 5, 7, 5, 5],
        'B' => [6, 5, 6, 5, 6],
        'C' => [7, 4, 4, 4, 7],
        'D' => [6, 5, 5, 5, 6],
        'E' => [7, 4, 6, 4, 7],
        'F' => [7, 4, 6, 4, 4],
        'G' => [7, 4, 5, 5, 7],
        'H' => [5, 5, 7, 5, 5],
        'I' => [7, 2, 2, 2, 7],
        'J' => [1, 1, 1, 5, 7],
        'K' => [5, 5, 6, 5, 5],
        'L' => [4, 4, 4, 4, 7],
        'M' => [5, 7, 7, 5, 5],
        'N' => [5, 7, 7, 7, 5],
        'O' => [7, 5, 5, 5, 7],
        'P' => [7, 5, 7, 4, 4],
        'Q' => [7, 5, 5, 7, 1],
        'R' => [6, 5, 6, 5, 5],
        'S' => [7, 4, 7, 1, 7],
        'T' => [7, 2, 2, 2, 2],
        'U' => [5, 5, 5, 5, 7],
        'V' => [5, 5, 5, 5, 2],
        'W' => [5, 5, 7, 7, 5],
        'X' => [5, 5, 2, 5, 5],
        'Y' => [5, 5, 2, 2, 2],
        'Z' => [7, 1, 2, 4, 7],
        '.' => [0, 0, 0, 0, 2],
        '-' => [0, 0, 7, 0, 0],
        '>' => [4, 2, 1, 2, 4],
        '[' => [6, 4, 4, 4, 6],
        ']' => [3, 1, 1, 1, 3],
        _ => [0; 5],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn holds_do_not_turn_hidden_time_into_stutter_and_buffer_is_bounded() {
        let base = Instant::now();
        let mut samples = Samples::default();
        samples.frame(base);
        for i in 1..=60 {
            samples.frame(base + Duration::from_micros(i * 16_667));
        }
        assert!((samples.summary().fps.unwrap() - 60.).abs() < 0.01);
        samples.hold();
        samples.frame(base + Duration::from_secs(100));
        samples.frame(base + Duration::from_millis(100_050));
        assert_eq!(samples.slow, 0);
        samples.frame(base + Duration::from_millis(100_101));
        assert_eq!(samples.slow, 1);
        for i in 1..=1000 {
            samples.frame(base + Duration::from_millis(100_101 + i * 17));
        }
        let result = samples.summary();
        assert_eq!(samples.count, 240);
        assert_eq!(samples.intervals, 1062);
        assert!((result.max_ms.unwrap() - 17.).abs() < 1e-9);
        assert_eq!(samples.slow, 1);
    }

    #[test]
    fn disabled_hud_collects_nothing_and_toggle_resets_measurement() {
        let base = Instant::now();
        let mut hud = Hud::default();
        hud.set_active(true);
        hud.frame(base);
        hud.frame(base + Duration::from_secs(1));
        assert_eq!(hud.samples.intervals, 0);
        hud.toggle();
        hud.frame(base);
        hud.frame(base + Duration::from_millis(10));
        assert_eq!(hud.samples.summary().fps, Some(100.));
        hud.set_active(false);
        hud.frame(base + Duration::from_secs(100));
        assert_eq!(hud.samples.intervals, 1);
        hud.toggle();
        hud.toggle();
        assert_eq!(hud.samples.intervals, 0);
        assert!(hud.samples.summary().fps.is_none());
    }
}
