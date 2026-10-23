//! Embedded AA glyph coverage; offline rasterization/provenance in hd_ui_font.json.
#[path = "ui_font_metrics.rs"]
mod metrics;
pub(super) const WIDTH: u32 = 1024;
pub(super) const HEIGHT: u32 = 384;
pub(super) const ALPHA: &[u8; 1024 * 384] = include_bytes!("ui_font.alpha");
pub(super) const COLORS: [[u8; 3]; 3] = [[236, 245, 233], [157, 177, 167], [180, 225, 140]];

fn glyph(c: char) -> [i32; 7] {
    metrics::GLYPHS[if (' '..='~').contains(&c) {
        c as usize - 32
    } else {
        b'?' as usize - 32
    }]
}
fn scale(value: i64, size: i32) -> i32 {
    {
        let scaled = value * i64::from(size.clamp(0, 96));
        let rounded = if scaled >= 0 {
            (scaled + 1536) / 3072
        } else {
            -((-scaled + 1536) / 3072)
        };
        rounded.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
    }
}
pub(super) fn width(text: &str, size: i32) -> i32 {
    scale(text.chars().map(|c| i64::from(glyph(c)[6])).sum(), size)
}
pub(super) fn fit(text: &str, size: i32, limit: i32) -> String {
    if width(text, size) <= limit {
        return text.into();
    }
    if width("...", size) > limit {
        return String::new();
    }
    let mut result = String::new();
    for c in text.chars() {
        let mut next = result.clone();
        next.push(c);
        next.push_str("...");
        if width(&next, size) > limit {
            break;
        }
        result.push(c);
    }
    result.push_str("...");
    result
}

pub(super) fn draw(
    out: &mut Vec<super::scene::Draw>,
    texture: usize,
    x: i32,
    y: i32,
    text: &str,
    size: i32,
) {
    let mut pen = 0i64;
    for c in text.chars() {
        let [gx, gy, w, h, left, top, advance] = glyph(c);
        if w > 0 && h > 0 {
            out.push(super::scene::Draw::Blit {
                texture,
                source: [gx as u32, gy as u32, w as u32, h as u32],
                dest: [
                    x + scale(pen + i64::from(left) * 64, size),
                    y + scale(i64::from(40 - top) * 64, size),
                    scale(i64::from(w) * 64, size).max(1),
                    scale(i64::from(h) * 64, size).max(1),
                ],
            });
        }
        pen += i64::from(advance);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn proportional_advances_and_ascii_fallback_keep_layout_predictable() {
        assert!(width("WWW", 20) > width("III", 20));
        assert!(width("Fun 01", 20) > 0);
        assert_eq!(width("한", 20), width("?", 20));
        assert_eq!(width("", 20), 0);
    }
    #[test]
    fn long_titles_fit_without_splitting_utf8_or_exceeding_status_width() {
        let text = "A very long level title with a 한 character and punctuation!";
        let fitted = fit(text, 20, 360);
        assert!(fitted.ends_with("..."));
        assert!(width(&fitted, 20) <= 360);
        assert_eq!(fit("Fun 1", 20, 360), "Fun 1");
        assert_eq!(fit(text, 20, 1), "");
    }
}
