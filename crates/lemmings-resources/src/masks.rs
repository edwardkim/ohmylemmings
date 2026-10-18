use crate::{DecompressError, MainDatFile, SpriteDirection};
use core::fmt;

pub const MAIN_MASK_SECTION: usize = 1;
pub const MAIN_MASK_RECORD_SIZE: usize = 388;

/// Stored MAIN section-1 families, including the complete ten-glyph digit bank.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum MainMaskKind {
    BashingRight,
    BashingLeft,
    MiningRight,
    MiningLeft,
    Explosion,
    Countdown,
}
pub const MAIN_MASK_KINDS: [MainMaskKind; 6] = [
    MainMaskKind::BashingRight,
    MainMaskKind::BashingLeft,
    MainMaskKind::MiningRight,
    MainMaskKind::MiningLeft,
    MainMaskKind::Explosion,
    MainMaskKind::Countdown,
];

/// Fixed disk layout. Placement depends on the sprite/action context and is not
/// encoded here; this describes neither terrain application nor countdown timing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MaskInfo {
    pub kind: MainMaskKind,
    pub offset: usize,
    pub width: u8,
    pub height: u8,
    pub frame_count: u8,
}
impl MaskInfo {
    pub fn frame_size(self) -> usize {
        usize::from(self.width) * usize::from(self.height) / 8
    }
}
impl MainMaskKind {
    pub const fn info(self) -> MaskInfo {
        let (offset, width, height, frame_count) = match self {
            Self::BashingRight => (0, 16, 10, 4),
            Self::BashingLeft => (80, 16, 10, 4),
            Self::MiningRight => (160, 16, 13, 2),
            Self::MiningLeft => (212, 16, 13, 2),
            Self::Explosion => (264, 16, 22, 1),
            Self::Countdown => (308, 8, 8, 10),
        };
        MaskInfo {
            kind: self,
            offset,
            width,
            height,
            frame_count,
        }
    }
}

impl MainDatFile<'_> {
    /// Checks the declared section size and exact caller output length before
    /// writing. A decompression error may partially change output: discard it.
    /// A successful view borrows only output, so the file may then be released.
    pub fn decode_masks_into(self, output: &mut [u8]) -> Result<MainMasks<'_>, MainMaskError> {
        let part = self.sections()[MAIN_MASK_SECTION];
        let actual = usize::from(part.header().decompressed_size);
        if actual != MAIN_MASK_RECORD_SIZE {
            return Err(MainMaskError::DeclaredSize { actual });
        }
        if output.len() != MAIN_MASK_RECORD_SIZE {
            return Err(MainMaskError::BufferSize {
                actual: output.len(),
            });
        }
        part.decompress_into(output)
            .map_err(MainMaskError::Decompress)?;
        MainMasks::parse(output)
    }
}

/// Borrowed original 388-byte mask record, without allocation, palette, or I/O.
/// All 23 stored frames are preserved. Bit 1 means selected/foreground, matching
/// Lemmix's source mask convention; it is the inverse of pinned TS `Mask.at`.
/// No terrain mutation, clipping, steel rules, anchor or timing policy is applied.
///
/// ```
/// use lemmings_resources::{MainMasks, SpriteDirection, MAIN_MASK_RECORD_SIZE};
/// let mut data = [0; MAIN_MASK_RECORD_SIZE];
/// data[0] = 0x80; // bashing right, frame 0, leftmost pixel
/// let masks = MainMasks::parse(&data)?;
/// assert_eq!(masks.bashing(SpriteDirection::Right).frame(0)?.at(0, 0), Some(true));
/// assert_eq!(masks.bashing(SpriteDirection::Right).frame(0)?.at(1, 0), Some(false));
/// assert_eq!(masks.countdown_digit(5)?.as_bytes().as_ptr(), data[340..].as_ptr());
/// # Ok::<(), lemmings_resources::MainMaskError>(())
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MainMasks<'a> {
    bytes: &'a [u8],
}
impl<'a> MainMasks<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, MainMaskError> {
        if bytes.len() != MAIN_MASK_RECORD_SIZE {
            return Err(MainMaskError::Size {
                actual: bytes.len(),
            });
        }
        Ok(Self { bytes })
    }
    pub const fn as_bytes(self) -> &'a [u8] {
        self.bytes
    }
    pub const fn frames(self, kind: MainMaskKind) -> MaskFrames<'a> {
        MaskFrames {
            bytes: self.bytes,
            info: kind.info(),
        }
    }
    pub fn bashing(self, direction: SpriteDirection) -> MaskFrames<'a> {
        self.frames(match direction {
            SpriteDirection::Right => MainMaskKind::BashingRight,
            SpriteDirection::Left => MainMaskKind::BashingLeft,
        })
    }
    pub fn mining(self, direction: SpriteDirection) -> MaskFrames<'a> {
        self.frames(match direction {
            SpriteDirection::Right => MainMaskKind::MiningRight,
            SpriteDirection::Left => MainMaskKind::MiningLeft,
        })
    }
    /// Value-based glyph lookup. Disk order is 9 down to 0; Lemmix's display
    /// reader uses only the 5..1 subset starting at 0x154. All ten are available.
    pub fn countdown_digit(self, digit: u8) -> Result<MaskBitmap<'a>, MainMaskError> {
        if digit > 9 {
            return Err(MainMaskError::Digit { value: digit });
        }
        self.frames(MainMaskKind::Countdown)
            .frame(usize::from(9 - digit))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MaskFrames<'a> {
    bytes: &'a [u8],
    info: MaskInfo,
}
impl<'a> MaskFrames<'a> {
    pub const fn info(self) -> MaskInfo {
        self.info
    }
    /// Exact stored frame index, without wrapping, mirroring or rotation.
    pub fn frame(self, index: usize) -> Result<MaskBitmap<'a>, MainMaskError> {
        if index >= usize::from(self.info.frame_count) {
            return Err(MainMaskError::FrameIndex {
                kind: self.info.kind,
                index,
                count: self.info.frame_count,
            });
        }
        let size = self.info.frame_size();
        let start = self.info.offset + index * size;
        // Fixed metadata partitions all 388 bytes; bounded u8 frame count means
        // arithmetic fits native/WASM usize before taking the borrowed window.
        Ok(MaskBitmap {
            bytes: &self.bytes[start..start + size],
            info: self.info,
        })
    }
}

/// Row-major one-bit bitmap, MSB left. `true` is a stored one, not an RGB/alpha
/// value or a terrain query. Out-of-range coordinates return None, not false.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MaskBitmap<'a> {
    bytes: &'a [u8],
    info: MaskInfo,
}
impl<'a> MaskBitmap<'a> {
    pub const fn as_bytes(self) -> &'a [u8] {
        self.bytes
    }
    pub const fn width(self) -> u8 {
        self.info.width
    }
    pub const fn height(self) -> u8 {
        self.info.height
    }
    pub fn at(self, x: usize, y: usize) -> Option<bool> {
        if x >= usize::from(self.width()) || y >= usize::from(self.height()) {
            return None;
        }
        Some(self.bit_at(y * usize::from(self.width()) + x))
    }
    /// Exact row-major caller output. A length error preserves every boolean.
    pub fn decode_into(self, output: &mut [bool]) -> Result<(), MainMaskError> {
        let expected = usize::from(self.width()) * usize::from(self.height());
        if output.len() != expected {
            return Err(MainMaskError::OutputSize {
                expected,
                actual: output.len(),
            });
        }
        for (index, bit) in output.iter_mut().enumerate() {
            *bit = self.bit_at(index);
        }
        Ok(())
    }
    fn bit_at(self, index: usize) -> bool {
        self.bytes[index / 8] & (128 >> (index % 8)) != 0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MainMaskError {
    Size {
        actual: usize,
    },
    DeclaredSize {
        actual: usize,
    },
    BufferSize {
        actual: usize,
    },
    FrameIndex {
        kind: MainMaskKind,
        index: usize,
        count: u8,
    },
    Digit {
        value: u8,
    },
    OutputSize {
        expected: usize,
        actual: usize,
    },
    Decompress(DecompressError),
}
impl fmt::Display for MainMaskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Size { actual } => write!(
                f,
                "MAIN mask record length {actual}; expected {MAIN_MASK_RECORD_SIZE}"
            ),
            Self::DeclaredSize { actual } => write!(
                f,
                "MAIN mask section 1 declared size {actual}; expected {MAIN_MASK_RECORD_SIZE}"
            ),
            Self::BufferSize { actual } => write!(
                f,
                "MAIN mask decode output length {actual}; expected {MAIN_MASK_RECORD_SIZE}"
            ),
            Self::FrameIndex { kind, index, count } => {
                write!(f, "MAIN mask {kind:?} frame {index}; expected 0..{count}")
            }
            Self::Digit { value } => write!(f, "MAIN countdown digit {value}; expected 0..=9"),
            Self::OutputSize { expected, actual } => write!(
                f,
                "MAIN mask bit output length {actual}; expected {expected}"
            ),
            Self::Decompress(error) => write!(f, "MAIN mask section 1: {error}"),
        }
    }
}
impl core::error::Error for MainMaskError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
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
    fn record() -> [u8; 388] {
        [0; 388]
    }
    fn container(data: &[u8]) -> Vec<u8> {
        let mut b = packed(&vec![0; 21104]);
        b.extend(packed(data));
        for _ in 2..7 {
            b.extend(packed(&[0]));
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
    fn every_short_prefix_and_extra_byte_is_rejected_and_exact_record_is_borrowed() {
        let b = [0; 389];
        for n in 0..388 {
            assert_eq!(
                MainMasks::parse(&b[..n]),
                Err(MainMaskError::Size { actual: n })
            );
        }
        assert_eq!(
            MainMasks::parse(&b),
            Err(MainMaskError::Size { actual: 389 })
        );
        let view = MainMasks::parse(&b[..388]).unwrap();
        assert_eq!(view.as_bytes().as_ptr(), b.as_ptr());
        assert_eq!(view.as_bytes(), &b[..388]);
    }
    #[test]
    fn all_six_families_partition_388_bytes_into_23_frames_and_3104_bits() {
        let b = record();
        let masks = MainMasks::parse(&b).unwrap();
        let mut offset = 0;
        let mut frames = 0;
        let mut bits = 0;
        for kind in MAIN_MASK_KINDS {
            let info = masks.frames(kind).info();
            assert_eq!(info, kind.info());
            assert_eq!(info.offset, offset);
            assert_eq!(
                info.frame_size(),
                usize::from(info.width) * usize::from(info.height) / 8
            );
            offset += info.frame_size() * usize::from(info.frame_count);
            frames += usize::from(info.frame_count);
            bits +=
                usize::from(info.width) * usize::from(info.height) * usize::from(info.frame_count);
        }
        assert_eq!((offset, frames, bits), (388, 23, 3104));
    }
    #[test]
    fn msb_left_row_order_and_true_for_stored_one_are_independent_of_ts_inversion() {
        let mut b = record();
        b[0] = 128;
        b[1] = 1;
        b[2] = 64;
        let masks = MainMasks::parse(&b).unwrap();
        let image = masks.frames(MainMaskKind::BashingRight).frame(0).unwrap();
        assert_eq!(image.at(0, 0), Some(true));
        assert_eq!(image.at(1, 0), Some(false));
        assert_eq!(image.at(15, 0), Some(true));
        assert_eq!(image.at(1, 1), Some(true));
        assert_eq!(image.at(0, 1), Some(false));
    }
    #[test]
    fn each_family_preserves_all_disk_frames_without_rotation_or_padding() {
        let mut b = record();
        let layouts = [
            (0, 20, 4),
            (80, 20, 4),
            (160, 26, 2),
            (212, 26, 2),
            (264, 44, 1),
            (308, 8, 10),
        ];
        for (offset, size, count) in layouts {
            for frame in 0..count {
                b[offset + size * frame] = 1 << (frame % 8);
            }
        }
        let view = MainMasks::parse(&b).unwrap();
        for kind in MAIN_MASK_KINDS {
            let group = view.frames(kind);
            for index in 0..usize::from(group.info().frame_count) {
                assert_eq!(group.frame(index).unwrap().at(7 - index % 8, 0), Some(true));
            }
        }
    }
    #[test]
    fn direction_helpers_select_independent_stored_images_without_mirroring() {
        let mut b = record();
        b[0] = 128;
        b[80] = 64;
        b[160] = 32;
        b[212] = 16;
        let view = MainMasks::parse(&b).unwrap();
        let r = view.bashing(SpriteDirection::Right);
        let l = view.bashing(SpriteDirection::Left);
        assert_eq!(r.info().kind, MainMaskKind::BashingRight);
        assert_eq!(l.info().kind, MainMaskKind::BashingLeft);
        assert_eq!(r.frame(0).unwrap().at(0, 0), Some(true));
        assert_eq!(l.frame(0).unwrap().at(1, 0), Some(true));
        assert_eq!(l.frame(0).unwrap().at(15, 0), Some(false));
        assert_eq!(
            view.mining(SpriteDirection::Right)
                .frame(0)
                .unwrap()
                .at(2, 0),
            Some(true)
        );
        assert_eq!(
            view.mining(SpriteDirection::Left)
                .frame(0)
                .unwrap()
                .at(3, 0),
            Some(true)
        );
    }
    #[test]
    fn countdown_digit_value_maps_to_stored_nine_down_to_zero_order() {
        let mut b = record();
        for index in 0..10 {
            b[308 + index * 8] = 1 << (index % 8);
        }
        let view = MainMasks::parse(&b).unwrap();
        for digit in 0..=9 {
            let image = view.countdown_digit(digit).unwrap();
            assert_eq!(
                image,
                view.frames(MainMaskKind::Countdown)
                    .frame(usize::from(9 - digit))
                    .unwrap()
            );
            assert_eq!((image.width(), image.height()), (8, 8));
            assert_eq!(image.at(7 - usize::from(9 - digit) % 8, 0), Some(true));
        }
        assert_eq!(
            view.countdown_digit(5).unwrap().as_bytes().as_ptr(),
            b[340..].as_ptr()
        );
        assert_eq!(
            view.countdown_digit(1).unwrap().as_bytes().as_ptr(),
            b[372..].as_ptr()
        );
    }
    #[test]
    fn countdown_values_above_nine_are_rejected_before_subtraction() {
        let b = record();
        let view = MainMasks::parse(&b).unwrap();
        for value in 10..=255 {
            assert_eq!(
                view.countdown_digit(value),
                Err(MainMaskError::Digit { value })
            );
        }
    }
    #[test]
    fn coordinates_do_not_wrap_rows_and_huge_values_are_checked_before_arithmetic() {
        let b = record();
        let view = MainMasks::parse(&b).unwrap();
        for kind in MAIN_MASK_KINDS {
            let image = view.frames(kind).frame(0).unwrap();
            for (x, y) in [
                (usize::from(image.width()), 0),
                (0, usize::from(image.height())),
                (usize::MAX, 0),
                (0, usize::MAX),
            ] {
                assert_eq!(image.at(x, y), None);
            }
        }
    }
    #[test]
    fn frame_indices_are_exact_and_never_wrap_or_overflow() {
        let b = record();
        let view = MainMasks::parse(&b).unwrap();
        for kind in MAIN_MASK_KINDS {
            let group = view.frames(kind);
            let count = group.info().frame_count;
            for index in [usize::from(count), usize::MAX] {
                assert_eq!(
                    group.frame(index),
                    Err(MainMaskError::FrameIndex { kind, index, count })
                );
            }
        }
    }
    #[test]
    fn bulk_decode_matches_all_bits_and_wrong_lengths_preserve_sentinels() {
        let mut b = record();
        for (i, v) in b.iter_mut().enumerate() {
            *v = (i * 73) as u8;
        }
        let view = MainMasks::parse(&b).unwrap();
        for kind in MAIN_MASK_KINDS {
            let group = view.frames(kind);
            for index in 0..usize::from(group.info().frame_count) {
                let image = group.frame(index).unwrap();
                let width = usize::from(image.width());
                let n = width * usize::from(image.height());
                let mut out = vec![true; n + 2];
                image.decode_into(&mut out[1..n + 1]).unwrap();
                assert!(out[0] && out[n + 1]);
                for (i, &bit) in out[1..n + 1].iter().enumerate() {
                    assert_eq!(image.at(i % width, i / width), Some(bit));
                }
                assert_eq!(image.as_bytes().len(), n / 8);
                for actual in [0, n - 1, n + 1] {
                    let mut output = vec![true; actual];
                    assert_eq!(
                        image.decode_into(&mut output),
                        Err(MainMaskError::OutputSize {
                            expected: n,
                            actual
                        })
                    );
                    assert!(output.iter().all(|&v| v));
                }
            }
        }
    }
    #[test]
    fn decoded_view_borrows_only_output_and_retains_the_last_original_bit() {
        let mut output = record();
        let ptr = output.as_ptr();
        let view = {
            let mut data = record();
            data[0] = 128;
            data[387] = 1;
            let bytes = container(&data);
            let file = MainDatFile::parse(&bytes).unwrap();
            file.decode_masks_into(&mut output).unwrap()
        };
        assert_eq!(view.as_bytes().as_ptr(), ptr);
        assert_eq!(
            view.bashing(SpriteDirection::Right)
                .frame(0)
                .unwrap()
                .at(0, 0),
            Some(true)
        );
        assert_eq!(view.countdown_digit(0).unwrap().at(7, 7), Some(true));
    }
    #[test]
    fn unsupported_declared_section_sizes_leave_output_untouched() {
        for actual in [1, 387, 389] {
            let b = container(&vec![0; actual]);
            let file = MainDatFile::parse(&b).unwrap();
            let mut output = [99; 388];
            assert_eq!(
                file.decode_masks_into(&mut output),
                Err(MainMaskError::DeclaredSize { actual })
            );
            assert_eq!(output, [99; 388]);
        }
    }
    #[test]
    fn caller_decode_buffer_must_have_exact_length_and_is_preserved_on_size_error() {
        let b = container(&record());
        let file = MainDatFile::parse(&b).unwrap();
        for actual in [0, 387, 389] {
            let mut output = vec![99; actual];
            assert_eq!(
                file.decode_masks_into(&mut output),
                Err(MainMaskError::BufferSize { actual })
            );
            assert!(output.iter().all(|&v| v == 99));
        }
    }
    #[test]
    fn invalid_commands_keep_their_error_source_and_other_sections_stay_readable() {
        let mut b = packed(&vec![0; 21104]);
        b.extend([8, 0, 0, 0, 1, 132, 0, 0, 0, 11, 0]);
        for _ in 2..7 {
            b.extend(packed(&[0]));
        }
        let file = MainDatFile::parse(&b).unwrap();
        let mut out = record();
        let e = file.decode_masks_into(&mut out).unwrap_err();
        assert!(matches!(e, MainMaskError::Decompress(_)));
        assert!(core::error::Error::source(&e).is_some());
        let mut other = [99];
        file.sections()[2].decompress_into(&mut other).unwrap();
        assert_eq!(other, [0]);
    }
}
