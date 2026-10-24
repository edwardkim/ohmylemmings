//! Bounded, world-aligned material cache and read-only terrain edge lighting.

use super::{assets::AssetFrame, scene::Texture};

pub(super) struct Material {
    pub rgba: Vec<u8>,
    width: usize,
    height: usize,
    period: [usize; 2],
}
impl Material {
    pub fn new(source: &Texture, frame: AssetFrame) -> Self {
        // Only the selected material is cached. A hostile display footprint
        // cannot allocate more than4MiB; the world period still uses metadata.
        let width = usize::from(frame.display[0]).min(1024);
        let height = usize::from(frame.display[1]).min(1024);
        let mut rgba = vec![0; width * height * 4];
        let axis = |at: usize, size: usize, extent: u32| {
            let position = (((at * 2 + 1) * extent as usize * 128 / size).saturating_sub(128))
                .min((extent as usize - 1) * 256);
            let apron = (size / 16).max(1);
            let edge = at.min(size - 1 - at);
            let blend = apron.saturating_sub(edge) * 128 / apron;
            (position, blend)
        };
        for y in 0..height {
            let (sy, by) = axis(y, height, frame.rect[3]);
            for x in 0..width {
                let (sx, bx) = axis(x, width, frame.rect[2]);
                let fetch = |x, y| sample(source, frame.rect, x, y);
                let mut color = fetch(sx, sy);
                let mx = (frame.rect[2] as usize - 1) * 256 - sx;
                let my = (frame.rect[3] as usize - 1) * 256 - sy;
                if bx > 0 {
                    color = mix(color, fetch(mx, sy), bx);
                }
                if by > 0 {
                    let mut opposite = fetch(sx, my);
                    if bx > 0 {
                        opposite = mix(opposite, fetch(mx, my), bx);
                    }
                    color = mix(color, opposite, by);
                }
                rgba[(y * width + x) * 4..(y * width + x + 1) * 4].copy_from_slice(&color);
            }
        }
        Self {
            rgba,
            width,
            height,
            period: frame.display.map(usize::from),
        }
    }
    pub fn pixel(&self, x: usize, y: usize) -> [u8; 4] {
        let x = x % self.period[0] * self.width / self.period[0];
        let y = y % self.period[1] * self.height / self.period[1];
        let at = (y * self.width + x) * 4;
        [self.rgba[at], self.rgba[at + 1], self.rgba[at + 2], 255]
    }
}
fn mix(a: [u8; 4], b: [u8; 4], weight: usize) -> [u8; 4] {
    let mut c = [255; 4];
    for i in 0..3 {
        c[i] =
            ((usize::from(a[i]) * (256 - weight) + usize::from(b[i]) * weight + 128) / 256) as u8;
    }
    c
}
fn sample(source: &Texture, rect: [u32; 4], x: usize, y: usize) -> [u8; 4] {
    let (x0, y0) = (x / 256, y / 256);
    let (x1, y1) = (
        (x0 + 1).min(rect[2] as usize - 1),
        (y0 + 1).min(rect[3] as usize - 1),
    );
    let pixel = |x: usize, y: usize| {
        let at = ((rect[1] as usize + y) * source.width as usize + rect[0] as usize + x) * 4;
        [
            source.rgba[at],
            source.rgba[at + 1],
            source.rgba[at + 2],
            255,
        ]
    };
    mix(
        mix(pixel(x0, y0), pixel(x1, y0), x % 256),
        mix(pixel(x0, y1), pixel(x1, y1), x % 256),
        y % 256,
    )
}

pub(super) fn shade(mut color: [u8; 4], mask: &[u8], width: usize, x: usize, y: usize) -> [u8; 4] {
    let (wx, wy) = ((x / 6) as i32, (y / 6) as i32);
    let height = mask.len() / width;
    let empty = |x: i32, y: i32| {
        x < 0
            || y < 0
            || x >= width as i32
            || y >= height as i32
            || mask[y as usize * width + x as usize] == 0
    };
    // Preserve tiny single-cell fragments instead of covering them in a bevel.
    if empty(wx - 1, wy) && empty(wx + 1, wy) && empty(wx, wy - 1) && empty(wx, wy + 1) {
        return color;
    }
    let ray = |dx, dy, offset: usize| {
        for distance in 1..=2 {
            if empty(wx + dx * distance, wy + dy * distance) {
                return (12 - offset - (distance as usize - 1) * 6) as i32;
            }
        }
        0
    };
    let gain = 256 + ray(0, -1, y % 6 + 1) * 4 + ray(-1, 0, x % 6 + 1) * 2
        - ray(0, 1, 6 - y % 6) * 6
        - ray(1, 0, 6 - x % 6) * 3;
    for value in &mut color[..3] {
        *value = (i32::from(*value) * gain / 256).clamp(0, 255) as u8;
    }
    color
}

/// Weather only the inside of substantial vertical faces. Logical collision is
/// untouched; the maximum inset is less than one logical pixel (6 HD pixels).
/// Walkable caps, narrow walls, steel and builder bricks retain their silhouette.
pub(super) fn contour(
    mut color: [u8; 4],
    mask: &[u8],
    width: usize,
    x: usize,
    y: usize,
    theme: u8,
) -> [u8; 4] {
    let (wx, wy) = ((x / 6) as i32, (y / 6) as i32);
    let height = mask.len() / width;
    let solid = |cx: i32, cy: i32| {
        cx >= 0
            && cy >= 0
            && cx < width as i32
            && cy < height as i32
            && mask[cy as usize * width + cx as usize] != 0
    };
    if !(-2..=2).all(|dy| solid(wx, wy + dy)) {
        return color;
    }
    for side in [-1, 1] {
        for distance in 1..=2 {
            let boundary = wx + side * distance;
            if solid(boundary, wy) {
                continue;
            }
            // Restrict to a locally vertical face with solid backing. Preserve
            // corners, diagonal slopes, single-cell debris and thin partitions.
            if (-1..=1).any(|dy| solid(boundary, wy + dy)) || !solid(wx - side * 2, wy) {
                break;
            }
            let depth = (distance as usize - 1) * 6 + if side < 0 { x % 6 } else { 5 - x % 6 };
            let maximum = match theme {
                2 => 2,
                3 => 4,
                _ => 5,
            };
            // World-anchored facets, independent of camera, chunk or frame.
            let node = y / 24;
            let value = |n: usize| {
                let mut h = (boundary as u32).wrapping_mul(0x9e3779b9)
                    ^ (n as u32).wrapping_mul(0x85ebca6b)
                    ^ u32::from(theme);
                h ^= h >> 16;
                (h as usize % (maximum + 1)) * 256
            };
            let inset = (value(node) * (24 - y % 24) + value(node + 1) * (y % 24)) / 24;
            if depth * 256 < inset {
                return [0; 4];
            }
            // An irregular inner bevel makes the small silhouette change read
            // as a fractured edge, rather than a dark straight border.
            let remaining = (depth * 256 - inset) / 256;
            if remaining < 7 {
                let gain = if side < 0 {
                    220 + remaining * 5
                } else {
                    158 + remaining * 14
                };
                for c in &mut color[..3] {
                    *c = (usize::from(*c) * gain / 256) as u8;
                }
            }
            break;
        }
    }
    color
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hd::{
        assets::{AssetFrame, AssetKey},
        scene::Texture,
    };
    #[test]
    fn vertical_contours_vary_inside_one_cell_and_preserve_walkable_caps_and_thin_walls() {
        let mut mask = vec![0; 40 * 80];
        for y in 5..75 {
            for x in 10..30 {
                mask[y * 40 + x] = 1;
            }
        }
        let original = mask.clone();
        let color = [120, 100, 80, 255];
        let mut widths = std::collections::BTreeSet::new();
        for y in 8 * 6..72 * 6 {
            let cut = (0..6)
                .filter(|x| contour(color, &mask, 40, 60 + x, y, 3)[3] == 0)
                .count();
            assert!(cut <= 4);
            widths.insert(cut);
            assert_eq!(contour(color, &mask, 40, 66, y, 3)[3], 255);
        }
        assert!(widths.len() >= 3);
        for theme in 0..5 {
            let limit = match theme {
                2 => 2,
                3 => 4,
                _ => 5,
            };
            for y in 8 * 6..72 * 6 {
                for edge in [60, 174] {
                    let removed = (0..6)
                        .filter(|dx| contour(color, &mask, 40, edge + dx, y, theme)[3] == 0)
                        .count();
                    assert!(removed <= limit);
                }
            }
        }
        for x in 60..180 {
            assert_eq!(contour(color, &mask, 40, x, 30, 3), color);
        }
        assert_eq!(mask, original);
        mask.fill(0);
        for y in 5..75 {
            mask[y * 40 + 10] = 1;
        }
        for y in 30..450 {
            assert_eq!(contour(color, &mask, 40, 60, y, 3), color);
        }
    }
    #[test]
    fn material_display_size_controls_world_period_and_wrap_is_continuous() {
        let source = Texture {
            width: 4,
            height: 2,
            rgba: [0, 64, 128, 255, 32, 96, 160, 255]
                .into_iter()
                .flat_map(|v| [v, v, v, 255])
                .collect(),
            revision: 1,
        };
        let frame = AssetFrame {
            key: AssetKey::Rock,
            atlas: 0,
            rect: [0, 0, 4, 2],
            display: [96, 48],
            anchor: [0, 0],
        };
        let tile = Material::new(&source, frame);
        assert_eq!(tile.pixel(0, 12), tile.pixel(96, 12));
        assert_eq!(tile.pixel(0, 12), tile.pixel(95, 12));
        assert_eq!(tile.pixel(20, 0), tile.pixel(20, 47));
        assert_ne!(tile.pixel(20, 12), tile.pixel(60, 12));
        assert_eq!(tile.pixel(96 + 20, 48 + 12), tile.pixel(20, 12));
        assert!(tile.pixel(20, 12)[0] > 0 && tile.pixel(20, 12)[0] < 64);
        let huge = Material::new(
            &source,
            AssetFrame {
                display: [4096, 4096],
                ..frame
            },
        );
        assert!(huge.rgba.len() <= 1024 * 1024 * 4);
        assert_eq!(huge.pixel(42, 42), huge.pixel(4096 + 42, 4096 + 42));
    }
    #[test]
    fn surface_lighting_is_inside_the_mask_and_distinguishes_top_bottom_and_cut_face() {
        let mut mask = vec![0; 32 * 32];
        for y in 5..25 {
            for x in 5..25 {
                mask[y * 32 + x] = 1;
            }
        }
        let color = [120, 100, 80, 255];
        assert_eq!(shade(color, &mask, 32, 15 * 6 + 3, 15 * 6 + 3), color);
        assert!(shade(color, &mask, 32, 15 * 6 + 3, 5 * 6)[0] > color[0]);
        assert!(shade(color, &mask, 32, 15 * 6 + 3, 25 * 6 - 1)[0] < color[0]);
        mask[15 * 32 + 16] = 0;
        assert!(shade(color, &mask, 32, 16 * 6 - 1, 15 * 6 + 3)[0] < color[0]);
        assert_eq!(shade(color, &mask, 32, 16 * 6 - 1, 15 * 6 + 3)[3], 255);
    }
}
