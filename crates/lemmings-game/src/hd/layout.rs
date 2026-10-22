use crate::{panel, render::SelectedSkill};

pub const WIDTH: u32 = 1920;
pub const HEIGHT: u32 = 1080;
pub const GAME_HEIGHT: u32 = 960;
pub const SCALE: i64 = 6;

/// Half-open rectangle in HD design coordinates or in the host's output units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    fn contains(self, x: f64, y: f64) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Control {
    Slower,
    Faster,
    Skill(SelectedSkill),
    Pause,
    Nuke,
    Restart,
    Minimap,
}

impl Control {
    /// Reuse original toolbar semantics, including held rate and double-click Nuke.
    /// This conversion never queues input or advances the existing state machine.
    pub fn original_button(self) -> Option<panel::Button> {
        Some(match self {
            Self::Slower => panel::Button::Slower,
            Self::Faster => panel::Button::Faster,
            Self::Skill(skill) => panel::Button::Skill(skill),
            Self::Pause => panel::Button::Pause,
            Self::Nuke => panel::Button::Nuke,
            Self::Restart | Self::Minimap => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ControlRegion {
    pub rect: Rect,
    pub control: Control,
}

/// Shared draw/hit regions. Status text and the minimap frame are not interactive.
pub fn control_regions() -> [ControlRegion; 14] {
    std::array::from_fn(|index| {
        let (x, width, control) = match index {
            0 => (400.0, 80.0, Control::Slower),
            1 => (480.0, 80.0, Control::Faster),
            2..=9 => (
                560.0 + (index - 2) as f64 * 104.0,
                104.0,
                Control::Skill(panel::SKILLS[index - 2]),
            ),
            10 => (1392.0, 88.0, Control::Pause),
            11 => (1480.0, 88.0, Control::Nuke),
            12 => (1568.0, 88.0, Control::Restart),
            _ => (1664.0, 248.0, Control::Minimap),
        };
        let (y, height) = if control == Control::Minimap {
            (976.0, 88.0)
        } else {
            (960.0, 120.0)
        };
        ControlRegion {
            rect: Rect {
                x,
                y,
                width,
                height,
            },
            control,
        }
    })
}

/// A resolved presentation target. Hosts still apply gameplay/replay/result guards.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Hit {
    Game { x: i32, y: i32 },
    Control(Control),
    Minimap { camera: i32 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    rect: Rect,
    scale: f64,
}

impl Viewport {
    pub fn fit(width: f64, height: f64) -> Option<Self> {
        if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
            return None;
        }
        let scale = (width / f64::from(WIDTH)).min(height / f64::from(HEIGHT));
        if scale == 0.0 {
            return None;
        }
        let (w, h) = (f64::from(WIDTH) * scale, f64::from(HEIGHT) * scale);
        if !w.is_finite() || !h.is_finite() || w <= 0.0 || h <= 0.0 {
            return None;
        }
        Some(Self {
            rect: Rect {
                x: (width - w) / 2.0,
                y: (height - h) / 2.0,
                width: w,
                height: h,
            },
            scale,
        })
    }

    pub const fn rect(self) -> Rect {
        self.rect
    }

    pub fn design_point(self, x: f64, y: f64) -> Option<(f64, f64)> {
        if !x.is_finite() || !y.is_finite() || !self.rect.contains(x, y) {
            return None;
        }
        Some((
            ((x - self.rect.x) / self.scale).clamp(0.0, f64::from(WIDTH).next_down()),
            ((y - self.rect.y) / self.scale).clamp(0.0, f64::from(HEIGHT).next_down()),
        ))
    }

    fn contains_design_rect(self, rect: Rect, x: f64, y: f64) -> bool {
        // Inverting first can round an exact toolbar edge to the preceding
        // game pixel at fractional DPI. Compare edges in the pointer's space.
        x >= self.rect.x + rect.x * self.scale
            && x < self.rect.x + (rect.x + rect.width) * self.scale
            && y >= self.rect.y + rect.y * self.scale
            && y < self.rect.y + (rect.y + rect.height) * self.scale
    }

    pub fn hit(self, x: f64, y: f64, world_width: usize) -> Option<Hit> {
        if world_width == 0 {
            return None;
        }
        self.design_point(x, y)?;
        if self.contains_design_rect(
            Rect {
                x: 0.0,
                y: 0.0,
                width: f64::from(WIDTH),
                height: f64::from(GAME_HEIGHT),
            },
            x,
            y,
        ) {
            return Some(Hit::Game {
                x: ((x - self.rect.x) / (SCALE as f64 * self.scale))
                    .floor()
                    .clamp(0.0, 319.0) as i32,
                y: ((y - self.rect.y) / (SCALE as f64 * self.scale))
                    .floor()
                    .clamp(0.0, 159.0) as i32,
            });
        }
        for region in control_regions() {
            if !self.contains_design_rect(region.rect, x, y) {
                continue;
            }
            if region.control != Control::Minimap {
                return Some(Hit::Control(region.control));
            }
            // Camera coordinates remain signed logical pixels, as in View::scroll.
            // Bound the span before adding the viewport so huge widths cannot wrap.
            let max = world_width.saturating_sub(320).min(i32::MAX as usize) as i64;
            let span = max + 320;
            let left = self.rect.x + region.rect.x * self.scale;
            let world_x =
                ((x - left) / (region.rect.width * self.scale) * span as f64).floor() as i64;
            return Some(Hit::Minimap {
                camera: (world_x - 160).clamp(0, max) as i32,
            });
        }
        None
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpriteRect {
    pub x: i64,
    pub y: i64,
    pub width: u32,
    pub height: u32,
}

/// Anchors are geometric coordinates measured from the image's left/top edges.
/// They may lie outside a trimmed image. Mirroring transforms the anchor too.
pub fn sprite_rect(
    foot: (i64, i64),
    size: (u32, u32),
    anchor: (i32, i32),
    flipped: bool,
) -> Option<SpriteRect> {
    if size.0 == 0 || size.1 == 0 {
        return None;
    }
    let anchor_x = if flipped {
        i64::from(size.0) - i64::from(anchor.0)
    } else {
        i64::from(anchor.0)
    };
    Some(SpriteRect {
        x: foot.0.checked_sub(anchor_x)?,
        y: foot.1.checked_sub(i64::from(anchor.1))?,
        width: size.0,
        height: size.1,
    })
}
