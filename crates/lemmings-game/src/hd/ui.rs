//! Code-native UI skin: antialiased corners, a restrained gradient and nine-slice borders.
use super::scene::Draw;
pub(super) const SKIN_WIDTH: u32 = 96;
pub(super) const SKIN_HEIGHT: u32 = 32;
pub(super) const BACKGROUND: [u8; 4] = [20, 34, 38, 255];
#[derive(Clone, Copy)]
pub(super) enum Ink {
    Main,
    Subtle,
    Accent,
}
fn inside(x: i32, y: i32, inset: i32) -> bool {
    let r = (8 - inset) * 8;
    let lo = inset * 8;
    let hi = (32 - inset) * 8;
    if x < lo || y < lo || x >= hi || y >= hi {
        return false;
    }
    let cx = x.clamp(lo + r, hi - r);
    let cy = y.clamp(lo + r, hi - r);
    (x - cx) * (x - cx) + (y - cy) * (y - cy) <= r * r
}
pub(super) fn skin() -> Vec<u8> {
    let styles = [
        ([28, 46, 48], [65, 87, 80]),
        ([43, 73, 56], [159, 211, 111]),
        ([25, 40, 44], [102, 136, 114]),
    ];
    let mut bytes = vec![0; 96 * 32 * 4];
    for (style, (fill, border)) in styles.into_iter().enumerate() {
        for y in 0..32 {
            for x in 0..32 {
                let (mut outer, mut inner) = (0, 0);
                for sy in 0..4 {
                    for sx in 0..4 {
                        let (px, py) = (x * 8 + sx * 2 + 1, y * 8 + sy * 2 + 1);
                        outer += i32::from(inside(px, py, 0));
                        inner += i32::from(inside(px, py, 2));
                    }
                }
                if outer == 0 {
                    continue;
                }
                let at = (y as usize * 96 + style * 32 + x as usize) * 4;
                for c in 0..3 {
                    bytes[at + c] = ((inner * (fill[c] + 8 * (31 - y) / 31)
                        + (outer - inner) * border[c])
                        / outer) as u8;
                }
                bytes[at + 3] = (outer * 255 / 16) as u8;
            }
        }
    }
    bytes
}
pub(super) fn card(out: &mut Vec<Draw>, texture: usize, style: u32, [x, y, w, h]: [i32; 4]) {
    let source = [0, 8, 24, 32];
    let dx = [x, x + 8, x + w - 8, x + w];
    let dy = [y, y + 8, y + h - 8, y + h];
    for row in 0..3 {
        for col in 0..3 {
            out.push(Draw::Blit {
                texture,
                source: [
                    style * 32 + source[col],
                    source[row],
                    source[col + 1] - source[col],
                    source[row + 1] - source[row],
                ],
                dest: [
                    dx[col],
                    dy[row],
                    dx[col + 1] - dx[col],
                    dy[row + 1] - dy[row],
                ],
            });
        }
    }
}
