//! HD-03 little-endian RGBA pack v1. PNG decoding and I/O belong to tools/hosts.
//! Pixels are straight-alpha, sRGB-encoded RGBA8. CRC detects accidental damage;
//! provenance/authenticity is supplied by the external SHA-256 manifest.
use super::Control;
use crate::panel;
use lemmings_resources::LemmingAnimationInfo;
use std::fmt;

const MAX_BYTES: usize = 64 * 1024 * 1024;
const MAX_DIM: u32 = 4096;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssetKey {
    Actor {
        slot: u8,
        frame: u16,
    },
    Rock,
    /// Original graphics set, object ID and absolute disk frame (not level slot).
    Object {
        set: u8,
        id: u8,
        frame: u16,
    },
    Icon(Control),
}

/// Stable atlas IDs. Skill order reuses the original panel contract; numeric
/// labels, selection/disabled states and minimap content are drawn by hosts.
pub const ICON_CONTROLS: [Control; 13] = [
    Control::Skill(panel::SKILLS[0]),
    Control::Skill(panel::SKILLS[1]),
    Control::Skill(panel::SKILLS[2]),
    Control::Skill(panel::SKILLS[3]),
    Control::Skill(panel::SKILLS[4]),
    Control::Skill(panel::SKILLS[5]),
    Control::Skill(panel::SKILLS[6]),
    Control::Skill(panel::SKILLS[7]),
    Control::Slower,
    Control::Faster,
    Control::Pause,
    Control::Nuke,
    Control::Restart,
];

/// HD-03b supports only the first theme's exit and entrance. These bounds are
/// pinned GROUND0O.DAT metadata, recorded in remaster_pack.json production.source_objects (#147).
pub const fn object_frame_count(set: u8, id: u8) -> Option<u16> {
    match (set, id) {
        (0, 0) => Some(1),
        (0, 1) => Some(10),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AssetFrame {
    pub key: AssetKey,
    pub atlas: u16,
    pub rect: [u32; 4],
    /// HD canvas dimensions, independent of the source image's pixel density.
    pub display: [u16; 2],
    /// Foot in the displayed canvas; may lie outside a trimmed source image.
    pub anchor: [i16; 2],
}

#[derive(Debug)]
pub struct Atlas<'a> {
    width: u32,
    height: u32,
    rgba: &'a [u8],
    alpha_cutoff: u8,
}

impl<'a> Atlas<'a> {
    pub const fn width(&self) -> u32 {
        self.width
    }
    pub const fn height(&self) -> u32 {
        self.height
    }
    pub const fn rgba(&self) -> &'a [u8] {
        self.rgba
    }

    /// Hosts discard samples with alpha <= this value before compositing.
    /// Other samples keep straight alpha; the borrowed pixels are never edited.
    pub const fn alpha_cutoff(&self) -> u8 {
        self.alpha_cutoff
    }

    pub fn row(&self, frame: &AssetFrame, row: u32) -> Option<&'a [u8]> {
        let [x, y, w, h] = frame.rect;
        if w == 0
            || h == 0
            || row >= h
            || x.checked_add(w)? > self.width
            || y.checked_add(h)? > self.height
        {
            return None;
        }
        let start = (usize::try_from(y.checked_add(row)?).ok()? * self.width as usize + x as usize)
            .checked_mul(4)?;
        self.rgba.get(start..start.checked_add(w as usize * 4)?)
    }
}

#[derive(Debug)]
pub struct AssetPack<'a> {
    atlases: Vec<Atlas<'a>>,
    frames: Vec<AssetFrame>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssetError {
    Magic,
    Truncated,
    Limit,
    Dimensions,
    Size,
    Checksum,
    Key,
    Frame,
    DuplicateKey,
    TrailingBytes,
}
impl fmt::Display for AssetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid HD asset pack: {self:?}")
    }
}
impl std::error::Error for AssetError {}

struct Reader<'a> {
    bytes: &'a [u8],
    position: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], AssetError> {
        let end = self.position.checked_add(n).ok_or(AssetError::Limit)?;
        let b = self
            .bytes
            .get(self.position..end)
            .ok_or(AssetError::Truncated)?;
        self.position = end;
        Ok(b)
    }
    fn u8(&mut self) -> Result<u8, AssetError> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, AssetError> {
        Ok(u16::from_le_bytes(
            self.take(2)?.try_into().expect("two bytes"),
        ))
    }
    fn i16(&mut self) -> Result<i16, AssetError> {
        Ok(i16::from_le_bytes(
            self.take(2)?.try_into().expect("two bytes"),
        ))
    }
    fn u32(&mut self) -> Result<u32, AssetError> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().expect("four bytes"),
        ))
    }
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = !0u32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb88320 & 0u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}

impl<'a> AssetPack<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, AssetError> {
        if bytes.len() > MAX_BYTES {
            return Err(AssetError::Limit);
        }
        let mut r = Reader { bytes, position: 0 };
        if r.take(8)? != b"LHDPAK01" {
            return Err(AssetError::Magic);
        }
        let atlas_count = r.u16()? as usize;
        let frame_count = r.u16()? as usize;
        if !(1..=16).contains(&atlas_count) || !(1..=4096).contains(&frame_count) {
            return Err(AssetError::Limit);
        }
        let mut atlases = Vec::with_capacity(atlas_count);
        for _ in 0..atlas_count {
            let width = r.u32()?;
            let height = r.u32()?;
            let size = r.u32()? as usize;
            let checksum = r.u32()?;
            let alpha_cutoff = r.u32()?;
            if alpha_cutoff > 254 {
                return Err(AssetError::Frame);
            }
            if width == 0 || height == 0 || width > MAX_DIM || height > MAX_DIM {
                return Err(AssetError::Dimensions);
            }
            let expected = (width as usize)
                .checked_mul(height as usize)
                .and_then(|n| n.checked_mul(4))
                .ok_or(AssetError::Size)?;
            if size != expected {
                return Err(AssetError::Size);
            }
            let rgba = r.take(size)?;
            if crc32(rgba) != checksum {
                return Err(AssetError::Checksum);
            }
            atlases.push(Atlas {
                width,
                height,
                rgba,
                alpha_cutoff: alpha_cutoff as u8,
            });
        }
        let mut frames = Vec::with_capacity(frame_count);
        for _ in 0..frame_count {
            let kind = r.u8()?;
            let slot = r.u8()?;
            let frame = r.u16()?;
            let key = match kind {
                0 => {
                    let info =
                        LemmingAnimationInfo::for_slot(slot as usize).ok_or(AssetError::Key)?;
                    if frame >= u16::from(info.frame_count) {
                        return Err(AssetError::Key);
                    }
                    AssetKey::Actor { slot, frame }
                }
                1 if slot == 0 && frame == 0 => AssetKey::Rock,
                2 => {
                    let (set, id) = (slot / 16, slot % 16);
                    if frame >= object_frame_count(set, id).ok_or(AssetError::Key)? {
                        return Err(AssetError::Key);
                    }
                    AssetKey::Object { set, id, frame }
                }
                3 if frame == 0 => {
                    AssetKey::Icon(*ICON_CONTROLS.get(slot as usize).ok_or(AssetError::Key)?)
                }
                _ => return Err(AssetError::Key),
            };
            let atlas = r.u16()?;
            if r.u16()? != 0 {
                return Err(AssetError::Frame);
            }
            let rect = [r.u32()?, r.u32()?, r.u32()?, r.u32()?];
            let display = [r.u16()?, r.u16()?];
            let anchor = [r.i16()?, r.i16()?];
            let image = atlases.get(atlas as usize).ok_or(AssetError::Frame)?;
            let [x, y, w, h] = rect;
            if w == 0
                || h == 0
                || x.checked_add(w).is_none_or(|n| n > image.width)
                || y.checked_add(h).is_none_or(|n| n > image.height)
                || display.iter().any(|&n| n == 0 || n > 4096)
                || anchor.iter().any(|&n| !(-4096..=4096).contains(&n))
            {
                return Err(AssetError::Frame);
            }
            if frames.iter().any(|f: &AssetFrame| f.key == key) {
                return Err(AssetError::DuplicateKey);
            }
            frames.push(AssetFrame {
                key,
                atlas,
                rect,
                display,
                anchor,
            });
        }
        if r.position != bytes.len() {
            return Err(AssetError::TrailingBytes);
        }
        Ok(Self { atlases, frames })
    }
    pub fn atlases(&self) -> &[Atlas<'a>] {
        &self.atlases
    }
    pub fn frames(&self) -> &[AssetFrame] {
        &self.frames
    }
    /// Missing HD states are explicit: no fabricated frame or silent fallback.
    pub fn frame(&self, key: AssetKey) -> Option<&AssetFrame> {
        self.frames.iter().find(|f| f.key == key)
    }
}
