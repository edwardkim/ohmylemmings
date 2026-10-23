//! HD-03 little-endian RGBA pack v1. PNG decoding and I/O belong to tools/hosts.
//! Pixels are straight-alpha, sRGB-encoded RGBA8. CRC detects accidental damage;
//! provenance/authenticity is supplied by the external SHA-256 manifest.
use super::Control;
use crate::panel;
use lemmings_resources::LemmingAnimationInfo;
use std::fmt;

const MAX_BYTES: usize = 128 * 1024 * 1024;
const MAX_DIM: u32 = 4096;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssetKey {
    Actor {
        slot: u8,
        frame: u16,
    },
    Rock,
    /// General terrain sets1..4. Set0 keeps the legacy Rock key/encoding.
    Material {
        set: u8,
    },
    /// Whole VGASPEC panorama, positioned at logical x=304, y=0.
    Special {
        set: u8,
    },
    /// Original graphics set, object ID and absolute disk frame (not level slot).
    Object {
        set: u8,
        id: u8,
        frame: u16,
    },
    Icon(Control),
}
impl AssetKey {
    pub const fn material(set: u8) -> Option<Self> {
        match set {
            0 => Some(Self::Rock),
            1..=4 => Some(Self::Material { set }),
            _ => None,
        }
    }
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

/// All54 nonempty GROUND0O..4O.DAT objects. Exact disk records are pinned
/// in hd_portals.json, hd_hazards.json, hd_traps.json and hd_decor.json.
pub const fn object_frame_count(set: u8, id: u8) -> Option<u16> {
    match (set, id) {
        (0..=4, 0) => Some(1),
        (0..=4, 1) => Some(10),
        _ => match exit_effect_frame_count(set, id) {
            Some(count) => Some(count),
            None => match hazard_frame_count(set, id) {
                Some(count) => Some(count),
                None => match trap_frame_count(set, id) {
                    Some(count) => Some(count),
                    None => decor_frame_count(set, id),
                },
            },
        },
    }
}

/// Exit ornament effect0 profiles; other object IDs include arrows and hazards.
pub const fn exit_effect_frame_count(set: u8, id: u8) -> Option<u16> {
    match (set, id) {
        (0, 7) | (1..=3, 6) | (4, 8) => Some(6),
        _ => None,
    }
}

/// GROUND effect5/6 profiles; disk records and triggers are pinned in hd_hazards.json.
/// These keys only select artwork and never alter the simulation's hazard rules.
pub const fn hazard_frame_count(set: u8, id: u8) -> Option<u16> {
    match (set, id) {
        (0..=3, 5) | (4, 6) | (1, 7) => Some(8),
        (1, 8 | 10) => Some(10),
        (2, 9) => Some(16),
        _ => None,
    }
}

/// Original effect4 object counts, pinned with triggers in hd_traps.json.
pub const fn trap_frame_count(set: u8, id: u8) -> Option<u16> {
    match (set, id) {
        (0, 6) | (2, 8) => Some(15),
        (0, 8) => Some(17),
        (0, 10) => Some(12),
        (3, 8) => Some(37),
        (3, 9 | 10) => Some(7),
        (4, 7) => Some(25),
        (4, 9) => Some(8),
        (4, 10) => Some(16),
        _ => None,
    }
}

/// Rotating pennants and effect7/8 arrows; exact records in hd_decor.json.
/// Artwork selection never changes the original one-way trigger masks.
pub const fn decor_frame_count(set: u8, id: u8) -> Option<u16> {
    match (set, id) {
        (0..=1, 2 | 9) | (2..=3, 2 | 7) | (4, 2 | 3) => Some(14),
        (1, 3 | 4) => Some(8),
        (0 | 2 | 3, 3 | 4) | (4, 4 | 5) => Some(7),
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

pub(super) fn crc32(bytes: &[u8]) -> u32 {
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
        if bytes.starts_with(b"LHDSET01") {
            let mut r = Reader { bytes, position: 8 };
            let count = r.u16()?;
            if !(1..=32).contains(&count) || r.u16()? != 0 {
                return Err(AssetError::Limit);
            }
            let mut result = Self {
                atlases: Vec::new(),
                frames: Vec::new(),
            };
            for _ in 0..count {
                let size = r.u32()? as usize;
                let part = r.take(size)?;
                if !part.starts_with(b"LHDPAK01") {
                    return Err(AssetError::Magic);
                }
                let pack = Self::parse(part)?;
                if result.atlases.len() + pack.atlases.len() > 32
                    || result.frames.len() + pack.frames.len() > 4096
                {
                    return Err(AssetError::Limit);
                }
                let base = result.atlases.len() as u16;
                for mut frame in pack.frames {
                    if result.frames.iter().any(|old| old.key == frame.key) {
                        return Err(AssetError::DuplicateKey);
                    }
                    frame.atlas += base;
                    result.frames.push(frame);
                }
                result.atlases.extend(pack.atlases);
            }
            if r.position != bytes.len() {
                return Err(AssetError::TrailingBytes);
            }
            return Ok(result);
        }
        let mut r = Reader { bytes, position: 0 };
        if r.take(8)? != b"LHDPAK01" {
            return Err(AssetError::Magic);
        }
        let atlas_count = r.u16()? as usize;
        let frame_count = r.u16()? as usize;
        if !(1..=32).contains(&atlas_count) || !(1..=4096).contains(&frame_count) {
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
                1 if frame == 0 => AssetKey::material(slot).ok_or(AssetError::Key)?,
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
                4 if slot < 4 && frame == 0 => AssetKey::Special { set: slot },
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
    /// Original slot/frame keys still absent from this pack. Partial v1 packs
    /// remain supported; producers can require complete coverage explicitly.
    pub fn missing_actor_frames(&self) -> Vec<(u8, u16)> {
        let mut missing = Vec::new();
        for slot in 0..28 {
            let info = LemmingAnimationInfo::for_slot(slot).expect("original slot");
            for frame in 0..u16::from(info.frame_count) {
                if self
                    .frame(AssetKey::Actor {
                        slot: slot as u8,
                        frame,
                    })
                    .is_none()
                {
                    missing.push((slot as u8, frame));
                }
            }
        }
        missing
    }
    /// Producers can require all55 portals; partial legacy v1 packs still load.
    pub fn missing_portal_frames(&self) -> Vec<(u8, u8, u16)> {
        let mut missing = Vec::new();
        for set in 0..5 {
            for id in 0..2 {
                for frame in 0..object_frame_count(set, id).expect("original portal") {
                    if self.frame(AssetKey::Object { set, id, frame }).is_none() {
                        missing.push((set, id, frame));
                    }
                }
            }
        }
        missing
    }
    pub fn missing_exit_effect_frames(&self) -> Vec<(u8, u8, u16)> {
        let mut missing = Vec::new();
        for set in 0..5 {
            for id in 0..16 {
                if let Some(count) = exit_effect_frame_count(set, id) {
                    for frame in 0..count {
                        if self.frame(AssetKey::Object { set, id, frame }).is_none() {
                            missing.push((set, id, frame));
                        }
                    }
                }
            }
        }
        missing
    }
    /// Missing HD states are explicit: no fabricated frame or silent fallback.
    pub fn frame(&self, key: AssetKey) -> Option<&AssetFrame> {
        self.frames.iter().find(|f| f.key == key)
    }
}
