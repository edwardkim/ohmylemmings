use crate::{DatError, DatSection, DatSections, DecompressError, IndexedPixel};
use core::fmt;

pub const MAIN_DAT_SECTION_COUNT: usize = 7;
pub const LEMMING_ANIMATION_COUNT: usize = 28;
pub const LEMMING_SPRITE_DATA_SIZE: usize = 21104;

/// Stored image direction, independent of simulation movement rules.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpriteDirection {
    Right,
    Left,
}

/// Resource animation names, not simulation states or playback behavior.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LemmingSpriteKind {
    Walking,
    Jumping,
    Digging,
    Climbing,
    Drowning,
    Hoisting,
    Building,
    Bashing,
    Mining,
    Falling,
    Umbrella,
    Splatting,
    Exiting,
    Vaporizing,
    Blocking,
    Shrugging,
    OhNo,
    Explosion,
}
impl LemmingSpriteKind {
    const fn slots(self) -> [usize; 2] {
        match self {
            Self::Walking => [0, 2],
            Self::Jumping => [1, 3],
            Self::Digging => [4, 4],
            Self::Climbing => [5, 6],
            Self::Drowning => [7, 7],
            Self::Hoisting => [8, 9],
            Self::Building => [10, 11],
            Self::Bashing => [12, 13],
            Self::Mining => [14, 15],
            Self::Falling => [16, 17],
            Self::Umbrella => [18, 19],
            Self::Splatting => [20, 20],
            Self::Exiting => [21, 21],
            Self::Vaporizing => [22, 22],
            Self::Blocking => [23, 23],
            Self::Shrugging => [24, 25],
            Self::OhNo => [26, 26],
            Self::Explosion => [27, 27],
        }
    }
}

/// Fixed DOS/Lemmix metadata. `direction=None` means both directions share the
/// same stored animation. Foot coordinates anchor the top-left to a lemming;
/// they do not specify collision, frame timing, looping, or initial frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LemmingAnimationInfo {
    pub slot: u8,
    pub kind: LemmingSpriteKind,
    pub direction: Option<SpriteDirection>,
    pub offset: usize,
    pub frame_count: u8,
    pub width: u8,
    pub height: u8,
    pub planes: u8,
    pub foot_x: u8,
    pub foot_y: u8,
}
impl LemmingAnimationInfo {
    pub fn origin_offset(self) -> (i16, i16) {
        (-i16::from(self.foot_x), -i16::from(self.foot_y))
    }
    pub fn frame_size(self) -> usize {
        usize::from(self.width) * usize::from(self.height) / 8 * usize::from(self.planes)
    }
}

// Styles.Base.pas InitMetadata at Lemmix 40e9bc3. Provenance and TS direction/
// anchor differences: mydocs/tech_insights/issue-16_lemming_sprites.md.
const ANIMATIONS: [LemmingAnimationInfo; LEMMING_ANIMATION_COUNT] = [
    LemmingAnimationInfo {
        slot: 0,
        kind: LemmingSpriteKind::Walking,
        direction: Some(SpriteDirection::Right),
        offset: 0x0000,
        frame_count: 8,
        width: 16,
        height: 10,
        planes: 2,
        foot_x: 8,
        foot_y: 10,
    },
    LemmingAnimationInfo {
        slot: 1,
        kind: LemmingSpriteKind::Jumping,
        direction: Some(SpriteDirection::Right),
        offset: 0x0140,
        frame_count: 1,
        width: 16,
        height: 10,
        planes: 2,
        foot_x: 8,
        foot_y: 10,
    },
    LemmingAnimationInfo {
        slot: 2,
        kind: LemmingSpriteKind::Walking,
        direction: Some(SpriteDirection::Left),
        offset: 0x0168,
        frame_count: 8,
        width: 16,
        height: 10,
        planes: 2,
        foot_x: 8,
        foot_y: 10,
    },
    LemmingAnimationInfo {
        slot: 3,
        kind: LemmingSpriteKind::Jumping,
        direction: Some(SpriteDirection::Left),
        offset: 0x02A8,
        frame_count: 1,
        width: 16,
        height: 10,
        planes: 2,
        foot_x: 8,
        foot_y: 10,
    },
    LemmingAnimationInfo {
        slot: 4,
        kind: LemmingSpriteKind::Digging,
        direction: None,
        offset: 0x02D0,
        frame_count: 16,
        width: 16,
        height: 14,
        planes: 3,
        foot_x: 8,
        foot_y: 12,
    },
    LemmingAnimationInfo {
        slot: 5,
        kind: LemmingSpriteKind::Climbing,
        direction: Some(SpriteDirection::Right),
        offset: 0x0810,
        frame_count: 8,
        width: 16,
        height: 12,
        planes: 2,
        foot_x: 8,
        foot_y: 12,
    },
    LemmingAnimationInfo {
        slot: 6,
        kind: LemmingSpriteKind::Climbing,
        direction: Some(SpriteDirection::Left),
        offset: 0x0990,
        frame_count: 8,
        width: 16,
        height: 12,
        planes: 2,
        foot_x: 8,
        foot_y: 12,
    },
    LemmingAnimationInfo {
        slot: 7,
        kind: LemmingSpriteKind::Drowning,
        direction: None,
        offset: 0x0B10,
        frame_count: 16,
        width: 16,
        height: 10,
        planes: 2,
        foot_x: 8,
        foot_y: 10,
    },
    LemmingAnimationInfo {
        slot: 8,
        kind: LemmingSpriteKind::Hoisting,
        direction: Some(SpriteDirection::Right),
        offset: 0x0D90,
        frame_count: 8,
        width: 16,
        height: 12,
        planes: 2,
        foot_x: 8,
        foot_y: 12,
    },
    LemmingAnimationInfo {
        slot: 9,
        kind: LemmingSpriteKind::Hoisting,
        direction: Some(SpriteDirection::Left),
        offset: 0x0F10,
        frame_count: 8,
        width: 16,
        height: 12,
        planes: 2,
        foot_x: 8,
        foot_y: 12,
    },
    LemmingAnimationInfo {
        slot: 10,
        kind: LemmingSpriteKind::Building,
        direction: Some(SpriteDirection::Right),
        offset: 0x1090,
        frame_count: 16,
        width: 16,
        height: 13,
        planes: 3,
        foot_x: 8,
        foot_y: 13,
    },
    LemmingAnimationInfo {
        slot: 11,
        kind: LemmingSpriteKind::Building,
        direction: Some(SpriteDirection::Left),
        offset: 0x1570,
        frame_count: 16,
        width: 16,
        height: 13,
        planes: 3,
        foot_x: 8,
        foot_y: 13,
    },
    LemmingAnimationInfo {
        slot: 12,
        kind: LemmingSpriteKind::Bashing,
        direction: Some(SpriteDirection::Right),
        offset: 0x1A50,
        frame_count: 32,
        width: 16,
        height: 10,
        planes: 3,
        foot_x: 8,
        foot_y: 10,
    },
    LemmingAnimationInfo {
        slot: 13,
        kind: LemmingSpriteKind::Bashing,
        direction: Some(SpriteDirection::Left),
        offset: 0x21D0,
        frame_count: 32,
        width: 16,
        height: 10,
        planes: 3,
        foot_x: 8,
        foot_y: 10,
    },
    LemmingAnimationInfo {
        slot: 14,
        kind: LemmingSpriteKind::Mining,
        direction: Some(SpriteDirection::Right),
        offset: 0x2950,
        frame_count: 24,
        width: 16,
        height: 13,
        planes: 3,
        foot_x: 8,
        foot_y: 13,
    },
    LemmingAnimationInfo {
        slot: 15,
        kind: LemmingSpriteKind::Mining,
        direction: Some(SpriteDirection::Left),
        offset: 0x30A0,
        frame_count: 24,
        width: 16,
        height: 13,
        planes: 3,
        foot_x: 8,
        foot_y: 13,
    },
    LemmingAnimationInfo {
        slot: 16,
        kind: LemmingSpriteKind::Falling,
        direction: Some(SpriteDirection::Right),
        offset: 0x37F0,
        frame_count: 4,
        width: 16,
        height: 10,
        planes: 2,
        foot_x: 8,
        foot_y: 10,
    },
    LemmingAnimationInfo {
        slot: 17,
        kind: LemmingSpriteKind::Falling,
        direction: Some(SpriteDirection::Left),
        offset: 0x3890,
        frame_count: 4,
        width: 16,
        height: 10,
        planes: 2,
        foot_x: 8,
        foot_y: 10,
    },
    LemmingAnimationInfo {
        slot: 18,
        kind: LemmingSpriteKind::Umbrella,
        direction: Some(SpriteDirection::Right),
        offset: 0x3930,
        frame_count: 8,
        width: 16,
        height: 16,
        planes: 3,
        foot_x: 8,
        foot_y: 16,
    },
    LemmingAnimationInfo {
        slot: 19,
        kind: LemmingSpriteKind::Umbrella,
        direction: Some(SpriteDirection::Left),
        offset: 0x3C30,
        frame_count: 8,
        width: 16,
        height: 16,
        planes: 3,
        foot_x: 8,
        foot_y: 16,
    },
    LemmingAnimationInfo {
        slot: 20,
        kind: LemmingSpriteKind::Splatting,
        direction: None,
        offset: 0x3F30,
        frame_count: 16,
        width: 16,
        height: 10,
        planes: 2,
        foot_x: 8,
        foot_y: 10,
    },
    LemmingAnimationInfo {
        slot: 21,
        kind: LemmingSpriteKind::Exiting,
        direction: None,
        offset: 0x41B0,
        frame_count: 8,
        width: 16,
        height: 13,
        planes: 2,
        foot_x: 8,
        foot_y: 13,
    },
    LemmingAnimationInfo {
        slot: 22,
        kind: LemmingSpriteKind::Vaporizing,
        direction: None,
        offset: 0x4350,
        frame_count: 14,
        width: 16,
        height: 14,
        planes: 4,
        foot_x: 8,
        foot_y: 14,
    },
    LemmingAnimationInfo {
        slot: 23,
        kind: LemmingSpriteKind::Blocking,
        direction: None,
        offset: 0x4970,
        frame_count: 16,
        width: 16,
        height: 10,
        planes: 2,
        foot_x: 8,
        foot_y: 10,
    },
    LemmingAnimationInfo {
        slot: 24,
        kind: LemmingSpriteKind::Shrugging,
        direction: Some(SpriteDirection::Right),
        offset: 0x4BF0,
        frame_count: 8,
        width: 16,
        height: 10,
        planes: 2,
        foot_x: 8,
        foot_y: 10,
    },
    LemmingAnimationInfo {
        slot: 25,
        kind: LemmingSpriteKind::Shrugging,
        direction: Some(SpriteDirection::Left),
        offset: 0x4D30,
        frame_count: 8,
        width: 16,
        height: 10,
        planes: 2,
        foot_x: 8,
        foot_y: 10,
    },
    LemmingAnimationInfo {
        slot: 26,
        kind: LemmingSpriteKind::OhNo,
        direction: None,
        offset: 0x4E70,
        frame_count: 16,
        width: 16,
        height: 10,
        planes: 2,
        foot_x: 8,
        foot_y: 10,
    },
    LemmingAnimationInfo {
        slot: 27,
        kind: LemmingSpriteKind::Explosion,
        direction: None,
        offset: 0x50F0,
        frame_count: 1,
        width: 32,
        height: 32,
        planes: 3,
        foot_x: 16,
        foot_y: 25,
    },
];

/// Original VGA MAIN.DAT: all seven section headers/payload checksums are
/// validated, but only sprite section 0 is decompressed here. Other sections
/// remain borrowed for later UI/mask readers; their command streams are unverified.
#[derive(Clone, Copy, Debug)]
pub struct MainDatFile<'a> {
    bytes: &'a [u8],
    sections: [DatSection<'a>; MAIN_DAT_SECTION_COUNT],
}
impl<'a> MainDatFile<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, LemmingSpriteError> {
        let mut iter = DatSections::new(bytes);
        let first = iter
            .next()
            .ok_or(LemmingSpriteError::SectionCount { actual: 0 })?
            .map_err(|error| LemmingSpriteError::Dat { section: 0, error })?;
        let mut sections = [first; MAIN_DAT_SECTION_COUNT];
        let mut count = 1;
        for result in iter {
            let section = result.map_err(|error| LemmingSpriteError::Dat {
                section: count,
                error,
            })?;
            if count < MAIN_DAT_SECTION_COUNT {
                sections[count] = section;
            }
            count += 1;
        }
        if count != MAIN_DAT_SECTION_COUNT {
            return Err(LemmingSpriteError::SectionCount { actual: count });
        }
        let actual = usize::from(first.header().decompressed_size);
        if actual != LEMMING_SPRITE_DATA_SIZE {
            return Err(LemmingSpriteError::SpriteSize { actual });
        }
        Ok(Self { bytes, sections })
    }
    pub const fn as_bytes(self) -> &'a [u8] {
        self.bytes
    }
    pub const fn sections(self) -> [DatSection<'a>; MAIN_DAT_SECTION_COUNT] {
        self.sections
    }
    /// A buffer-length error preserves output. A command-stream error may
    /// partially write output: discard it on failure. The returned view borrows
    /// only output, so the compressed file can be released after success.
    pub fn decode_sprites_into(
        self,
        output: &mut [u8],
    ) -> Result<LemmingSprites<'_>, LemmingSpriteError> {
        if output.len() != LEMMING_SPRITE_DATA_SIZE {
            return Err(LemmingSpriteError::BufferSize {
                actual: output.len(),
            });
        }
        self.sections[0]
            .decompress_into(output)
            .map_err(LemmingSpriteError::Decompress)?;
        LemmingSprites::parse(output)
    }
}

/// Borrowed 21104-byte sprite record with no palette, allocation, or I/O.
/// All frames are planar, MSB left, plane 0 contributes bit 0. Index 0 is
/// transparent; indices 1..15 address a caller-selected sixteen-color palette.
///
/// ```
/// use lemmings_resources::{LemmingSprites, LemmingSpriteKind, SpriteDirection,
///     IndexedPixel, LEMMING_SPRITE_DATA_SIZE};
/// let mut data = [0; LEMMING_SPRITE_DATA_SIZE];
/// data[0] = 0x80; // walking right, frame 0, plane 0, top-left
/// let sprites = LemmingSprites::parse(&data)?;
/// let animation = sprites.for_sprite(LemmingSpriteKind::Walking, SpriteDirection::Right);
/// assert_eq!(animation.info().origin_offset(), (-8, -10));
/// assert_eq!(animation.frame(0)?.pixel(0, 0), Some(IndexedPixel { color: 1, opaque: true }));
/// # Ok::<(), lemmings_resources::LemmingSpriteError>(())
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LemmingSprites<'a> {
    bytes: &'a [u8],
}
impl<'a> LemmingSprites<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, LemmingSpriteError> {
        if bytes.len() != LEMMING_SPRITE_DATA_SIZE {
            return Err(LemmingSpriteError::SpriteSize {
                actual: bytes.len(),
            });
        }
        Ok(Self { bytes })
    }
    pub const fn as_bytes(self) -> &'a [u8] {
        self.bytes
    }
    pub fn animation(self, slot: usize) -> Result<LemmingFrames<'a>, LemmingSpriteError> {
        let info = *ANIMATIONS
            .get(slot)
            .ok_or(LemmingSpriteError::AnimationIndex { index: slot })?;
        Ok(LemmingFrames {
            bytes: self.bytes,
            info,
        })
    }
    pub fn for_sprite(
        self,
        kind: LemmingSpriteKind,
        direction: SpriteDirection,
    ) -> LemmingFrames<'a> {
        let slots = kind.slots();
        let index = match direction {
            SpriteDirection::Right => slots[0],
            SpriteDirection::Left => slots[1],
        };
        self.animation(index).expect("fixed sprite mapping")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LemmingFrames<'a> {
    bytes: &'a [u8],
    info: LemmingAnimationInfo,
}
impl<'a> LemmingFrames<'a> {
    pub const fn info(self) -> LemmingAnimationInfo {
        self.info
    }
    /// Disk frame order: no modulo, first-frame rotation, mirroring or replay.
    pub fn frame(self, index: usize) -> Result<SpriteFrame<'a>, LemmingSpriteError> {
        if index >= usize::from(self.info.frame_count) {
            return Err(LemmingSpriteError::FrameIndex {
                slot: self.info.slot,
                index,
                count: self.info.frame_count,
            });
        }
        // Fixed metadata partitions exactly 21104 bytes. Index is bounded by a
        // u8 count before multiplication, including on wasm32.
        let size = self.info.frame_size();
        let start = self.info.offset + index * size;
        Ok(SpriteFrame {
            bytes: &self.bytes[start..start + size],
            info: self.info,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpriteFrame<'a> {
    bytes: &'a [u8],
    info: LemmingAnimationInfo,
}
impl SpriteFrame<'_> {
    pub const fn width(self) -> u8 {
        self.info.width
    }
    pub const fn height(self) -> u8 {
        self.info.height
    }
    pub fn pixel(self, x: usize, y: usize) -> Option<IndexedPixel> {
        if x >= usize::from(self.width()) || y >= usize::from(self.height()) {
            return None;
        }
        Some(self.pixel_at(y * usize::from(self.width()) + x))
    }
    /// Exact row-major output. A length error leaves every pixel unchanged.
    pub fn decode_into(self, output: &mut [IndexedPixel]) -> Result<(), LemmingSpriteError> {
        let expected = usize::from(self.width()) * usize::from(self.height());
        if output.len() != expected {
            return Err(LemmingSpriteError::OutputSize {
                expected,
                actual: output.len(),
            });
        }
        for (i, pixel) in output.iter_mut().enumerate() {
            *pixel = self.pixel_at(i);
        }
        Ok(())
    }
    fn pixel_at(self, index: usize) -> IndexedPixel {
        let size = usize::from(self.width()) * usize::from(self.height()) / 8;
        let mut color = 0;
        for plane in 0..usize::from(self.info.planes) {
            if self.bytes[plane * size + index / 8] & (128 >> (index % 8)) != 0 {
                color |= 1 << plane;
            }
        }
        IndexedPixel {
            color,
            opaque: color != 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LemmingSpriteError {
    Dat { section: usize, error: DatError },
    SectionCount { actual: usize },
    SpriteSize { actual: usize },
    BufferSize { actual: usize },
    Decompress(DecompressError),
    AnimationIndex { index: usize },
    FrameIndex { slot: u8, index: usize, count: u8 },
    OutputSize { expected: usize, actual: usize },
}
impl fmt::Display for LemmingSpriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dat { section, error } => write!(f, "MAIN section {section}: {error}"),
            Self::SectionCount { actual } => write!(
                f,
                "MAIN has {actual} sections; expected {MAIN_DAT_SECTION_COUNT}"
            ),
            Self::SpriteSize { actual } => write!(
                f,
                "MAIN sprite record length {actual}; expected {LEMMING_SPRITE_DATA_SIZE}"
            ),
            Self::BufferSize { actual } => write!(
                f,
                "MAIN sprite output length {actual}; expected {LEMMING_SPRITE_DATA_SIZE}"
            ),
            Self::Decompress(error) => write!(f, "MAIN sprite section 0: {error}"),
            Self::AnimationIndex { index } => write!(
                f,
                "sprite animation {index}; expected 0..{LEMMING_ANIMATION_COUNT}"
            ),
            Self::FrameIndex { slot, index, count } => write!(
                f,
                "sprite animation {slot} frame {index}; expected 0..{count}"
            ),
            Self::OutputSize { expected, actual } => write!(
                f,
                "sprite pixel output length {actual}; expected {expected}"
            ),
        }
    }
}
impl core::error::Error for LemmingSpriteError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Dat { error, .. } => Some(error),
            Self::Decompress(error) => Some(error),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use std::{format, string::String, vec, vec::Vec};
    fn data() -> Vec<u8> {
        vec![0; LEMMING_SPRITE_DATA_SIZE]
    }
    fn pixel(color: u8) -> IndexedPixel {
        IndexedPixel {
            color,
            opaque: color != 0,
        }
    }
    fn container(first: &[u8], count: usize) -> Vec<u8> {
        let mut b = Vec::new();
        if count > 0 {
            b.extend(packed(first));
        }
        for i in 1..count {
            b.extend(packed(&[i as u8]));
        }
        b
    }
    fn packed(data: &[u8]) -> Vec<u8> {
        let mut bits = String::new();
        for chunk in data.rchunks(264) {
            bits += &if chunk.len() <= 8 {
                format!("00{:03b}", chunk.len() - 1)
            } else {
                format!("111{:08b}", chunk.len() - 9)
            };
            for byte in chunk.iter().rev() {
                bits += &format!("{byte:08b}");
            }
        }
        let payload: Vec<u8> = bits
            .as_bytes()
            .rchunks(8)
            .map(|c| {
                c.iter()
                    .enumerate()
                    .fold(0, |a, (i, b)| a | ((b - b'0') << i))
            })
            .collect();
        let mut b = vec![
            ((bits.len() - 1) % 8 + 1) as u8,
            payload.iter().fold(0, |a, b| a ^ b),
            0,
            0,
        ];
        b.extend((data.len() as u16).to_be_bytes());
        b.extend([0, 0]);
        b.extend(((10 + payload.len()) as u16).to_be_bytes());
        b.extend(payload);
        b
    }

    #[test]
    fn every_short_sprite_record_and_extra_byte_is_rejected_and_exact_input_is_borrowed() {
        let b = data();
        for length in 0..b.len() {
            assert_eq!(
                LemmingSprites::parse(&b[..length]),
                Err(LemmingSpriteError::SpriteSize { actual: length })
            );
        }
        let sprites = LemmingSprites::parse(&b).unwrap();
        assert_eq!(sprites.as_bytes().as_ptr(), b.as_ptr());
        assert_eq!(sprites.as_bytes(), b);
        let b = vec![0; LEMMING_SPRITE_DATA_SIZE + 1];
        assert_eq!(
            LemmingSprites::parse(&b),
            Err(LemmingSpriteError::SpriteSize { actual: b.len() })
        );
    }
    #[test]
    fn metadata_partitions_the_entire_record_into_28_animations_and_337_frames() {
        let b = data();
        let sprites = LemmingSprites::parse(&b).unwrap();
        let mut offset = 0;
        let mut frames = 0;
        let mut pixels = 0;
        for slot in 0..28 {
            let info = sprites.animation(slot).unwrap().info();
            assert_eq!(usize::from(info.slot), slot);
            assert_eq!(info.offset, offset);
            for direction in [SpriteDirection::Right, SpriteDirection::Left] {
                if info.direction.is_none() || info.direction == Some(direction) {
                    assert_eq!(sprites.for_sprite(info.kind, direction).info(), info);
                }
            }
            assert_eq!(
                info.frame_size(),
                usize::from(info.width) * usize::from(info.height) * usize::from(info.planes) / 8
            );
            offset += info.frame_size() * usize::from(info.frame_count);
            frames += usize::from(info.frame_count);
            pixels +=
                usize::from(info.width) * usize::from(info.height) * usize::from(info.frame_count);
        }
        assert_eq!((offset, frames, pixels), (21104, 337, 63488));
    }
    #[test]
    fn planes_are_msb_first_and_frames_keep_their_disk_order_with_zero_transparent() {
        let mut b = data();
        b[0] = 128;
        b[20] = 64;
        b[19] = 1;
        b[39] = 1;
        b[40 + 20] = 128;
        let sprites = LemmingSprites::parse(&b).unwrap();
        let frames = sprites.animation(0).unwrap();
        let f = frames.frame(0).unwrap();
        assert_eq!(f.pixel(0, 0), Some(pixel(1)));
        assert_eq!(f.pixel(1, 0), Some(pixel(2)));
        assert_eq!(f.pixel(2, 0), Some(pixel(0)));
        assert_eq!(f.pixel(15, 9), Some(pixel(3)));
        assert_eq!(frames.frame(1).unwrap().pixel(0, 0), Some(pixel(2)));
        assert_eq!(frames.frame(7).unwrap().pixel(0, 0), Some(pixel(0)));
    }
    #[test]
    fn three_and_four_plane_frames_keep_high_color_indices_without_a_palette() {
        let mut b = data();
        b[0x2d0 + 56] = 128;
        b[0x4350 + 84] = 128;
        for plane in 0..4 {
            b[0x4350 + plane * 28 + 27] = 1;
        }
        let sprites = LemmingSprites::parse(&b).unwrap();
        assert_eq!(
            sprites.animation(4).unwrap().frame(0).unwrap().pixel(0, 0),
            Some(pixel(4))
        );
        let f = sprites.animation(22).unwrap().frame(0).unwrap();
        assert_eq!(f.pixel(0, 0), Some(pixel(8)));
        assert_eq!(f.pixel(15, 13), Some(pixel(15)));
    }
    #[test]
    fn explosion_width_32_crosses_all_byte_boundaries_and_reaches_last_record_byte() {
        let mut b = data();
        for x in [7, 8, 15, 16, 31] {
            b[0x50f0 + x / 8] |= 128 >> (x % 8);
        }
        *b.last_mut().unwrap() = 1;
        let sprites = LemmingSprites::parse(&b).unwrap();
        let f = sprites.animation(27).unwrap().frame(0).unwrap();
        assert_eq!((f.width(), f.height()), (32, 32));
        for x in 0..32 {
            assert_eq!(
                f.pixel(x, 0),
                Some(pixel(if [7, 8, 15, 16, 31].contains(&x) {
                    1
                } else {
                    0
                }))
            );
        }
        assert_eq!(f.pixel(31, 31), Some(pixel(4)));
    }
    #[test]
    fn left_and_right_are_independent_stored_images_without_automatic_mirroring() {
        let mut b = data();
        b[0] = 128;
        b[0x168] = 64;
        let sprites = LemmingSprites::parse(&b).unwrap();
        let right = sprites.for_sprite(LemmingSpriteKind::Walking, SpriteDirection::Right);
        let left = sprites.for_sprite(LemmingSpriteKind::Walking, SpriteDirection::Left);
        assert_eq!(right.info().direction, Some(SpriteDirection::Right));
        assert_eq!(left.info().direction, Some(SpriteDirection::Left));
        assert_eq!(right.frame(0).unwrap().pixel(0, 0), Some(pixel(1)));
        assert_eq!(left.frame(0).unwrap().pixel(1, 0), Some(pixel(1)));
        assert_eq!(left.frame(0).unwrap().pixel(15, 0), Some(pixel(0)));
    }
    #[test]
    fn shared_directions_shrugging_and_anchors_follow_lemmix_metadata() {
        use LemmingSpriteKind::*;
        let b = data();
        let s = LemmingSprites::parse(&b).unwrap();
        for kind in [
            Digging, Drowning, Splatting, Exiting, Vaporizing, Blocking, OhNo, Explosion,
        ] {
            let r = s.for_sprite(kind, SpriteDirection::Right);
            let l = s.for_sprite(kind, SpriteDirection::Left);
            assert_eq!(r, l);
            assert_eq!(r.info().direction, None);
        }
        let r = s.for_sprite(Shrugging, SpriteDirection::Right).info();
        let l = s.for_sprite(Shrugging, SpriteDirection::Left).info();
        assert_eq!((r.slot, l.slot), (24, 25));
        for direction in [SpriteDirection::Right, SpriteDirection::Left] {
            assert_eq!(
                s.for_sprite(Mining, direction).info().origin_offset(),
                (-8, -13)
            );
        }
        assert_eq!(
            s.for_sprite(Vaporizing, SpriteDirection::Left)
                .info()
                .origin_offset(),
            (-8, -14)
        );
        assert_eq!(
            s.for_sprite(Explosion, SpriteDirection::Left)
                .info()
                .origin_offset(),
            (-16, -25)
        );
    }
    #[test]
    fn invalid_animation_frame_and_large_coordinates_are_checked_before_arithmetic() {
        let b = data();
        let s = LemmingSprites::parse(&b).unwrap();
        for index in [28, usize::MAX] {
            assert_eq!(
                s.animation(index),
                Err(LemmingSpriteError::AnimationIndex { index })
            );
        }
        for slot in 0..28 {
            let a = s.animation(slot).unwrap();
            let count = a.info().frame_count;
            for index in [usize::from(count), usize::MAX] {
                assert_eq!(
                    a.frame(index),
                    Err(LemmingSpriteError::FrameIndex {
                        slot: slot as u8,
                        index,
                        count
                    })
                );
            }
            let f = a.frame(0).unwrap();
            for (x, y) in [
                (usize::from(f.width()), 0),
                (0, usize::from(f.height())),
                (usize::MAX, 0),
                (0, usize::MAX),
            ] {
                assert_eq!(f.pixel(x, y), None);
            }
        }
    }
    #[test]
    fn whole_frame_decode_matches_pixel_lookup_and_wrong_lengths_preserve_canaries() {
        let mut b = data();
        for (i, v) in b.iter_mut().enumerate() {
            *v = (i.wrapping_mul(73)) as u8;
        }
        let s = LemmingSprites::parse(&b).unwrap();
        let sentinel = IndexedPixel {
            color: 99,
            opaque: true,
        };
        for slot in 0..28 {
            let a = s.animation(slot).unwrap();
            for index in 0..usize::from(a.info().frame_count) {
                let f = a.frame(index).unwrap();
                let n = usize::from(f.width()) * usize::from(f.height());
                let mut out = vec![sentinel; n + 2];
                f.decode_into(&mut out[1..n + 1]).unwrap();
                assert_eq!(out[0], sentinel);
                assert_eq!(out[n + 1], sentinel);
                for (i, p) in out[1..n + 1].iter().enumerate() {
                    assert_eq!(
                        Some(*p),
                        f.pixel(i % usize::from(f.width()), i / usize::from(f.width()))
                    );
                }
            }
        }
        let f = s.animation(0).unwrap().frame(0).unwrap();
        for actual in [0, 159, 161] {
            let mut out = vec![sentinel; actual];
            assert_eq!(
                f.decode_into(&mut out),
                Err(LemmingSpriteError::OutputSize {
                    expected: 160,
                    actual
                })
            );
            assert!(out.iter().all(|p| *p == sentinel));
        }
    }
    #[test]
    fn main_requires_all_seven_sections_and_validates_appended_sections() {
        let d = data();
        for count in [0, 1, 6, 8] {
            assert!(
                matches!(MainDatFile::parse(&container(&d,count)),Err(LemmingSpriteError::SectionCount { actual }) if actual==count)
            );
        }
        let b = container(&d, 7);
        let main = MainDatFile::parse(&b).unwrap();
        assert_eq!(main.as_bytes().as_ptr(), b.as_ptr());
        assert_eq!(main.as_bytes(), b);
        let sections = main.sections();
        assert_eq!(sections[0].header().decompressed_size, 21104);
        for section in &sections[1..] {
            assert_eq!(section.header().decompressed_size, 1);
        }
    }
    #[test]
    fn late_checksum_damage_and_short_trailing_header_return_dat_context() {
        let mut b = container(&data(), 7);
        *b.last_mut().unwrap() ^= 1;
        let e = MainDatFile::parse(&b).unwrap_err();
        assert!(matches!(e, LemmingSpriteError::Dat { section: 6, .. }));
        assert!(core::error::Error::source(&e).is_some());
        let mut b = container(&data(), 7);
        b.push(0);
        assert!(matches!(
            MainDatFile::parse(&b),
            Err(LemmingSpriteError::Dat { section: 7, .. })
        ));
    }
    #[test]
    fn main_rejects_the_wrong_declared_sprite_size_even_with_valid_framing() {
        for actual in [
            1,
            LEMMING_SPRITE_DATA_SIZE - 1,
            LEMMING_SPRITE_DATA_SIZE + 1,
        ] {
            let b = container(&vec![0; actual], 7);
            assert!(
                matches!(MainDatFile::parse(&b),Err(LemmingSpriteError::SpriteSize { actual:n }) if n==actual)
            );
        }
    }
    #[test]
    fn main_decode_borrows_only_caller_output_and_other_sections_remain_available() {
        let mut output = data();
        let ptr = output.as_ptr();
        let sprites = {
            let mut d = data();
            d[0] = 128;
            *d.last_mut().unwrap() = 1;
            let b = container(&d, 7);
            let file = MainDatFile::parse(&b).unwrap();
            let mut last = [0];
            file.sections()[6].decompress_into(&mut last).unwrap();
            assert_eq!(last, [6]);
            file.decode_sprites_into(&mut output).unwrap()
        };
        assert_eq!(sprites.as_bytes().as_ptr(), ptr);
        assert_eq!(
            sprites.animation(0).unwrap().frame(0).unwrap().pixel(0, 0),
            Some(pixel(1))
        );
        assert_eq!(
            sprites
                .animation(27)
                .unwrap()
                .frame(0)
                .unwrap()
                .pixel(31, 31),
            Some(pixel(4))
        );
    }
    #[test]
    fn wrong_decode_buffer_is_untouched_and_malformed_commands_keep_their_error_source() {
        let b = container(&data(), 7);
        let file = MainDatFile::parse(&b).unwrap();
        for actual in [
            0,
            LEMMING_SPRITE_DATA_SIZE - 1,
            LEMMING_SPRITE_DATA_SIZE + 1,
        ] {
            let mut out = vec![99; actual];
            assert_eq!(
                file.decode_sprites_into(&mut out),
                Err(LemmingSpriteError::BufferSize { actual })
            );
            assert!(out.iter().all(|&p| p == 99));
        }
        let mut b = vec![8, 0, 0, 0, 0x52, 0x70, 0, 0, 0, 11, 0];
        for _ in 1..7 {
            b.extend(packed(&[0]));
        }
        let file = MainDatFile::parse(&b).unwrap();
        let mut out = data();
        let e = file.decode_sprites_into(&mut out).unwrap_err();
        assert!(matches!(e, LemmingSpriteError::Decompress(_)));
        assert!(core::error::Error::source(&e).is_some());
    }
}
