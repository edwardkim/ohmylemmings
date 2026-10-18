use core::{fmt, iter::FusedIterator, str::Utf8Error};

// DOS TLVLRec / TLevelLoader.TranslateLevel, pinned source and differences:
// mydocs/tech_insights/issue-10_level_records.md.
/// Size of one decompressed DOS base-level record.
pub const LEVEL_RECORD_SIZE: usize = 2048;

/// DOS level metadata. Skill order: climber, floater, bomber, blocker, builder,
/// basher, miner, digger. Times are minutes; graphic selectors retain DOS values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LevelHeader {
    pub release_rate: u16,
    pub total: u16,
    pub required: u16,
    pub minutes: u16,
    pub skills: [u16; 8],
    pub screen_x: u16,
    pub graphics: u16,
    pub special: u16,
    pub superlemming: bool,
}

impl LevelHeader {
    /// Maps DOS selector 1..=4 to VGASPEC0..3; zero means no special image.
    pub const fn special_graphic_set(&self) -> Option<u8> {
        if self.special == 0 {
            None
        } else {
            Some((self.special - 1) as u8)
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObjectFlags {
    pub upside_down: bool,
    pub no_overwrite: bool,
    pub only_on_terrain: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObjectPlacement {
    pub slot: usize,
    pub x: i32,
    pub y: u16,
    pub id: u8,
    pub flags: ObjectFlags,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TerrainFlags {
    pub upside_down: bool,
    pub no_overwrite: bool,
    pub erase: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TerrainPlacement {
    pub slot: usize,
    pub x: i16,
    pub y: i16,
    pub id: u8,
    pub flags: TerrainFlags,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SteelArea {
    pub slot: usize,
    pub x: i16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

/// Borrowed, validated base record. Catalog mapping and ODDTABLE overlays are
/// separate operations; this view does not represent a displayed level yet.
///
/// ```
/// use lemmings_resources::{LEVEL_RECORD_SIZE, LevelRecord};
///
/// let mut bytes = [0; LEVEL_RECORD_SIZE];
/// for offset in [0, 2, 4, 6] { bytes[offset + 1] = 1; }
/// bytes[288..1888].fill(0xff); // Empty terrain slots.
/// bytes[2016..].fill(b' ');
/// let record = LevelRecord::parse(&bytes)?;
/// assert_eq!(record.header().total, 1);
/// assert_eq!(record.objects().count(), 0);
/// assert_eq!(record.title_bytes().len(), 32);
/// # Ok::<(), lemmings_resources::LevelError>(())
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LevelRecord<'a> {
    bytes: &'a [u8; LEVEL_RECORD_SIZE],
    header: LevelHeader,
}

impl<'a> LevelRecord<'a> {
    /// Validates the entire record before exposing infallible placement views.
    ///
    /// Supports DOS original graphic sets 0..=4 and special selectors 0..=4.
    /// Counts follow the supported DOS ranges; invalid values are never clamped.
    /// Unused placement bits and raw title bytes are retained by `as_bytes()`.
    pub fn parse(bytes: &'a [u8]) -> Result<Self, LevelError> {
        let bytes: &'a [u8; LEVEL_RECORD_SIZE] =
            bytes.try_into().map_err(|_| LevelError::Size {
                expected: LEVEL_RECORD_SIZE,
                actual: bytes.len(),
            })?;
        let words: [u16; 16] = core::array::from_fn(|index| word(bytes, index * 2));
        check(LevelField::ReleaseRate, 0, words[0], 1, 99)?;
        check(LevelField::Total, 2, words[1], 1, 255)?;
        check(LevelField::Required, 4, words[2], 1, words[1])?;
        check(LevelField::Minutes, 6, words[3], 1, 99)?;
        for index in 0..8 {
            check(
                LevelField::Skill(index),
                8 + index * 2,
                words[4 + index],
                0,
                99,
            )?;
        }
        check(LevelField::Graphics, 26, words[13], 0, 4)?;
        check(LevelField::Special, 28, words[14], 0, 4)?;
        if words[15] != 0 && words[15] != 0xffff {
            return Err(LevelError::UnsupportedSuperlemming {
                offset: 30,
                value: words[15],
            });
        }
        for (slot, entry) in bytes[1888..2016].chunks_exact(4).enumerate() {
            if entry[3] != 0 {
                return Err(LevelError::UnsupportedSteelByte {
                    offset: 1888 + slot * 4 + 3,
                    value: entry[3],
                });
            }
        }
        Ok(Self {
            bytes,
            header: LevelHeader {
                release_rate: words[0],
                total: words[1],
                required: words[2],
                minutes: words[3],
                skills: core::array::from_fn(|index| words[4 + index]),
                screen_x: words[12],
                graphics: words[13],
                special: words[14],
                superlemming: words[15] == 0xffff,
            },
        })
    }

    pub const fn header(&self) -> LevelHeader {
        self.header
    }

    pub const fn as_bytes(&self) -> &'a [u8; LEVEL_RECORD_SIZE] {
        self.bytes
    }

    /// Preserves all 32 title bytes, including spaces, NULs and non-UTF-8 bytes.
    pub fn title_bytes(&self) -> &'a [u8] {
        &self.bytes[2016..]
    }

    /// Optional UTF-8 interpretation, with no trimming or DOS code-page guess.
    pub fn title(&self) -> Result<&'a str, Utf8Error> {
        core::str::from_utf8(self.title_bytes())
    }

    /// Active objects in file order, retaining their zero-based original slots.
    /// Only an all-zero eight-byte record is empty. Coordinates follow Lemmix's
    /// unsigned words: X minus 16, Y unchanged; no world-boundary clipping.
    pub fn objects(&self) -> impl FusedIterator<Item = ObjectPlacement> + '_ {
        self.bytes[32..288]
            .chunks_exact(8)
            .enumerate()
            .filter_map(|(slot, b)| {
                if b.iter().all(|byte| *byte == 0) {
                    return None;
                }
                Some(ObjectPlacement {
                    slot,
                    x: i32::from(word(b, 0)) - 16,
                    y: word(b, 2),
                    id: b[5] & 0x0f,
                    flags: ObjectFlags {
                        upside_down: b[7] == 0x8f,
                        no_overwrite: b[6] & 0x80 != 0,
                        only_on_terrain: b[6] & 0x40 != 0,
                    },
                })
            })
    }

    /// Active terrain in file order. Only 0xffffffff is empty. X uses twelve
    /// unsigned bits minus 16; Y uses nine signed bits minus 4. Unused bits are
    /// ignored for placement decoding, including nonzero bits in original data.
    pub fn terrains(&self) -> impl FusedIterator<Item = TerrainPlacement> + '_ {
        self.bytes[288..1888]
            .chunks_exact(4)
            .enumerate()
            .filter_map(|(slot, b)| {
                if b == [0xff; 4] {
                    return None;
                }
                let mut y = ((u16::from(b[2]) << 1) | u16::from(b[3] >> 7)) as i16;
                if y >= 256 {
                    y -= 512;
                }
                Some(TerrainPlacement {
                    slot,
                    x: (((u16::from(b[0]) & 0x0f) << 8) | u16::from(b[1])) as i16 - 16,
                    y: y - 4,
                    id: b[3] & 0x3f,
                    flags: TerrainFlags {
                        upside_down: b[0] & 0x40 != 0,
                        no_overwrite: b[0] & 0x80 != 0,
                        erase: b[0] & 0x20 != 0,
                    },
                })
            })
    }

    /// Active steel areas, with four-pixel units and original slot numbers.
    /// X is byte 0 plus the high bit of byte 1; Y is byte 1's low seven bits.
    /// Size's high nibble encodes width and its low nibble encodes height.
    pub fn steel(&self) -> impl FusedIterator<Item = SteelArea> + '_ {
        self.bytes[1888..2016]
            .chunks_exact(4)
            .enumerate()
            .filter_map(|(slot, b)| {
                if b.iter().all(|byte| *byte == 0) {
                    return None;
                }
                Some(SteelArea {
                    slot,
                    x: ((u16::from(b[0]) << 1) | u16::from(b[1] >> 7)) as i16 * 4 - 16,
                    y: u16::from(b[1] & 0x7f) * 4,
                    width: u16::from(b[2] >> 4) * 4 + 4,
                    height: u16::from(b[2] & 0x0f) * 4 + 4,
                })
            })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LevelField {
    ReleaseRate,
    Total,
    Required,
    Minutes,
    Skill(usize),
    Graphics,
    Special,
}

// Called only for the fixed-size record header or an eight-byte object entry.
fn word(bytes: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes([bytes[offset], bytes[offset + 1]])
}

fn check(
    field: LevelField,
    offset: usize,
    value: u16,
    min: u16,
    max: u16,
) -> Result<(), LevelError> {
    if value < min || value > max {
        Err(LevelError::InvalidValue {
            offset,
            field,
            value,
            min,
            max,
        })
    } else {
        Ok(())
    }
}

/// Field offsets are relative to the decompressed record, not the DAT file.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LevelError {
    Size {
        expected: usize,
        actual: usize,
    },
    InvalidValue {
        offset: usize,
        field: LevelField,
        value: u16,
        min: u16,
        max: u16,
    },
    UnsupportedSuperlemming {
        offset: usize,
        value: u16,
    },
    UnsupportedSteelByte {
        offset: usize,
        value: u8,
    },
}

impl fmt::Display for LevelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Size { expected, actual } => {
                write!(f, "DOS level needs exactly {expected} bytes, got {actual}")
            }
            Self::InvalidValue {
                offset,
                field,
                value,
                min,
                max,
            } => write!(
                f,
                "DOS level {field:?} at offset {offset} is {value}, expected {min}..={max}"
            ),
            Self::UnsupportedSuperlemming { offset, value } => write!(
                f,
                "DOS level flag at offset {offset} is {value:#06x}, expected 0x0000 or 0xffff"
            ),
            Self::UnsupportedSteelByte { offset, value } => write!(
                f,
                "DOS steel reserved byte at offset {offset} is {value:#04x}, expected zero"
            ),
        }
    }
}

impl core::error::Error for LevelError {}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use std::{vec, vec::Vec};

    fn fixture() -> [u8; LEVEL_RECORD_SIZE] {
        let mut bytes = [0; LEVEL_RECORD_SIZE];
        // Distinct big-endian header values; no original game bytes.
        let words = [
            50u16, 80, 40, 5, 1, 2, 3, 4, 5, 6, 7, 8, 0x1234, 4, 3, 0xffff,
        ];
        for (i, word) in words.iter().enumerate() {
            bytes[i * 2..i * 2 + 2].copy_from_slice(&word.to_be_bytes());
        }
        bytes[288..1888].fill(0xff);
        bytes[2016..].copy_from_slice(b"0123456789abcdefghijklmnopqrstuv");
        bytes
    }

    fn put_word(bytes: &mut [u8], offset: usize, value: u16) {
        bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
    }

    fn invalid(offset: usize, field: LevelField, value: u16, min: u16, max: u16) -> LevelError {
        LevelError::InvalidValue {
            offset,
            field,
            value,
            min,
            max,
        }
    }

    #[test]
    fn header_words_skill_order_and_title_are_borrowed_without_normalization() {
        let bytes = fixture();
        let record = LevelRecord::parse(&bytes).unwrap();
        assert_eq!(
            record.header(),
            LevelHeader {
                release_rate: 50,
                total: 80,
                required: 40,
                minutes: 5,
                skills: [1, 2, 3, 4, 5, 6, 7, 8],
                screen_x: 0x1234,
                graphics: 4,
                special: 3,
                superlemming: true,
            }
        );
        assert_eq!(record.header().special_graphic_set(), Some(2));
        assert_eq!(record.as_bytes().as_ptr(), bytes.as_ptr());
        assert_eq!(record.title_bytes().as_ptr(), bytes[2016..].as_ptr());
        assert_eq!(record.title().unwrap(), "0123456789abcdefghijklmnopqrstuv");
    }

    #[test]
    fn empty_slots_and_iterators_remain_empty_after_exhaustion() {
        fn fused<T: FusedIterator>(_: &T) {}
        let bytes = fixture();
        let record = LevelRecord::parse(&bytes).unwrap();
        let mut objects = record.objects();
        let mut terrains = record.terrains();
        let mut steel = record.steel();
        fused(&objects);
        fused(&terrains);
        fused(&steel);
        for _ in 0..3 {
            assert_eq!(objects.next(), None);
            assert_eq!(terrains.next(), None);
            assert_eq!(steel.next(), None);
        }
    }

    #[test]
    fn every_truncated_record_and_extra_input_is_rejected() {
        let bytes = fixture();
        for actual in 0..LEVEL_RECORD_SIZE {
            assert_eq!(
                LevelRecord::parse(&bytes[..actual]),
                Err(LevelError::Size {
                    expected: 2048,
                    actual
                })
            );
        }
        for actual in [2049, 4096] {
            assert_eq!(
                LevelRecord::parse(&vec![0; actual]),
                Err(LevelError::Size {
                    expected: 2048,
                    actual
                })
            );
        }
    }

    #[test]
    fn header_ranges_reject_values_instead_of_clamping() {
        for (offset, field, value, min, max) in [
            (0, LevelField::ReleaseRate, 0, 1, 99),
            (0, LevelField::ReleaseRate, 100, 1, 99),
            (2, LevelField::Total, 0, 1, 255),
            (2, LevelField::Total, 256, 1, 255),
            (4, LevelField::Required, 0, 1, 80),
            (4, LevelField::Required, 81, 1, 80),
            (6, LevelField::Minutes, 0, 1, 99),
            (6, LevelField::Minutes, 100, 1, 99),
            (26, LevelField::Graphics, 5, 0, 4),
            (28, LevelField::Special, 5, 0, 4),
            (28, LevelField::Special, u16::MAX, 0, 4),
        ] {
            let mut bytes = fixture();
            put_word(&mut bytes, offset, value);
            assert_eq!(
                LevelRecord::parse(&bytes),
                Err(invalid(offset, field, value, min, max))
            );
        }
        for index in 0..8 {
            let mut bytes = fixture();
            put_word(&mut bytes, 8 + index * 2, 100);
            assert_eq!(
                LevelRecord::parse(&bytes),
                Err(invalid(8 + index * 2, LevelField::Skill(index), 100, 0, 99))
            );
        }
    }

    #[test]
    fn valid_header_extremes_and_raw_viewport_are_preserved() {
        for (rate, total, minutes, skills, screen_x) in
            [(1, 1, 1, 0, 0), (99, 255, 99, 99, u16::MAX)]
        {
            let mut bytes = fixture();
            for (offset, value) in [
                (0, rate),
                (2, total),
                (4, total),
                (6, minutes),
                (24, screen_x),
            ] {
                put_word(&mut bytes, offset, value);
            }
            for index in 0..8 {
                put_word(&mut bytes, 8 + 2 * index, skills);
            }
            let header = LevelRecord::parse(&bytes).unwrap().header();
            assert_eq!(
                (
                    header.release_rate,
                    header.total,
                    header.required,
                    header.minutes,
                    header.screen_x
                ),
                (rate, total, total, minutes, screen_x)
            );
            assert_eq!(header.skills, [skills; 8]);
        }
    }

    #[test]
    fn graphic_selectors_and_superlemming_are_explicit() {
        for selector in 0..=4 {
            let mut bytes = fixture();
            put_word(&mut bytes, 26, selector);
            put_word(&mut bytes, 28, selector);
            put_word(&mut bytes, 30, 0);
            let header = LevelRecord::parse(&bytes).unwrap().header();
            assert_eq!(header.graphics, selector);
            assert_eq!(
                header.special_graphic_set(),
                if selector == 0 {
                    None
                } else {
                    Some(selector as u8 - 1)
                }
            );
            assert!(!header.superlemming);
        }
        for value in [1, 0x00ff, 0xff00, 0xfffe] {
            let mut bytes = fixture();
            put_word(&mut bytes, 30, value);
            assert_eq!(
                LevelRecord::parse(&bytes),
                Err(LevelError::UnsupportedSuperlemming { offset: 30, value })
            );
        }
    }

    #[test]
    fn titles_preserve_spaces_nuls_and_non_utf8_bytes() {
        let mut bytes = fixture();
        bytes[2016..].fill(b' ');
        bytes[2017] = b'A';
        bytes[2018] = 0;
        let record = LevelRecord::parse(&bytes).unwrap();
        assert_eq!(&record.title().unwrap()[..4], " A\0 ");
        assert_eq!(record.title_bytes().len(), 32);
        bytes[2047] = 0xff;
        let record = LevelRecord::parse(&bytes).unwrap();
        assert_eq!(record.title_bytes()[31], 0xff);
        assert!(record.title().is_err());
    }

    #[test]
    fn objects_keep_original_slots_and_zero_flags_do_not_mean_empty() {
        let mut bytes = fixture();
        bytes[32 + 2 * 8..40 + 2 * 8].copy_from_slice(&[0, 32, 0, 7, 0, 3, 0, 0]);
        bytes[32 + 31 * 8..40 + 31 * 8]
            .copy_from_slice(&[0, 0, 0xff, 0xff, 0xff, 0xfa, 0xc0, 0x8f]);
        let record = LevelRecord::parse(&bytes).unwrap();
        let objects: Vec<_> = record.objects().collect();
        assert_eq!(
            objects,
            vec![
                ObjectPlacement {
                    slot: 2,
                    x: 16,
                    y: 7,
                    id: 3,
                    flags: ObjectFlags {
                        upside_down: false,
                        no_overwrite: false,
                        only_on_terrain: false
                    }
                },
                ObjectPlacement {
                    slot: 31,
                    x: -16,
                    y: u16::MAX,
                    id: 10,
                    flags: ObjectFlags {
                        upside_down: true,
                        no_overwrite: true,
                        only_on_terrain: true
                    }
                },
            ]
        );
    }

    #[test]
    fn object_coordinates_use_unsigned_big_endian_words_and_minus_sixteen_x() {
        for (encoded_x, encoded_y) in [
            (0u16, 0u16),
            (15, 255),
            (16, 256),
            (0x8000, 0x8000),
            (u16::MAX, u16::MAX),
        ] {
            let mut bytes = fixture();
            put_word(&mut bytes, 32, encoded_x);
            put_word(&mut bytes, 34, encoded_y);
            bytes[39] = 0x0f;
            let placement = LevelRecord::parse(&bytes)
                .unwrap()
                .objects()
                .next()
                .unwrap();
            assert_eq!(placement.x, i32::from(encoded_x) - 16);
            assert_eq!(placement.y, encoded_y);
        }
    }

    #[test]
    fn object_flags_follow_modifier_bits_and_exact_display_mode() {
        for modifier in 0..=u8::MAX {
            let mut bytes = fixture();
            bytes[38] = modifier;
            bytes[39] = 0x0f;
            let object = LevelRecord::parse(&bytes)
                .unwrap()
                .objects()
                .next()
                .unwrap();
            assert_eq!(object.flags.no_overwrite, modifier & 0x80 != 0);
            assert_eq!(object.flags.only_on_terrain, modifier & 0x40 != 0);
            assert!(!object.flags.upside_down);
        }
        for display in [0, 0x0f, 0x80, 0x8f, 0xff] {
            let mut bytes = fixture();
            bytes[37] = 1;
            bytes[39] = display;
            let object = LevelRecord::parse(&bytes)
                .unwrap()
                .objects()
                .next()
                .unwrap();
            assert_eq!(object.flags.upside_down, display == 0x8f);
        }
    }

    #[test]
    fn zero_terrain_is_active_and_only_all_ones_is_empty() {
        let mut bytes = fixture();
        bytes[288 + 3 * 4..292 + 3 * 4].fill(0);
        bytes[288 + 399 * 4..292 + 399 * 4].copy_from_slice(&[0xff, 0xff, 0xff, 0xfe]);
        let record = LevelRecord::parse(&bytes).unwrap();
        let terrain: Vec<_> = record.terrains().collect();
        assert_eq!(
            terrain,
            vec![
                TerrainPlacement {
                    slot: 3,
                    x: -16,
                    y: -4,
                    id: 0,
                    flags: TerrainFlags {
                        upside_down: false,
                        no_overwrite: false,
                        erase: false
                    }
                },
                TerrainPlacement {
                    slot: 399,
                    x: 4079,
                    y: -5,
                    id: 62,
                    flags: TerrainFlags {
                        upside_down: true,
                        no_overwrite: true,
                        erase: true
                    }
                },
            ]
        );
    }

    #[test]
    fn every_nine_bit_terrain_y_uses_signed_boundary_at_256() {
        for encoded in 0u16..512 {
            let mut bytes = fixture();
            bytes[288..292].copy_from_slice(&[
                0,
                16,
                (encoded >> 1) as u8,
                ((encoded & 1) << 7) as u8,
            ]);
            let terrain = LevelRecord::parse(&bytes)
                .unwrap()
                .terrains()
                .next()
                .unwrap();
            let expected = if encoded < 256 {
                encoded as i16 - 4
            } else {
                encoded as i16 - 516
            };
            assert_eq!(terrain.y, expected);
        }
    }

    #[test]
    fn terrain_flags_ids_and_ignored_bits_do_not_change_coordinates() {
        for modifier in 0u8..8 {
            for ignored in [0, 0x10] {
                let mut bytes = fixture();
                bytes[288..292].copy_from_slice(&[
                    (modifier << 5) | ignored | 0x0a,
                    0xbc,
                    0x80,
                    0x7f,
                ]);
                let terrain = LevelRecord::parse(&bytes)
                    .unwrap()
                    .terrains()
                    .next()
                    .unwrap();
                assert_eq!((terrain.x, terrain.y, terrain.id), (0x0abc - 16, -260, 63));
                assert_eq!(
                    terrain.flags,
                    TerrainFlags {
                        erase: modifier & 1 != 0,
                        upside_down: modifier & 2 != 0,
                        no_overwrite: modifier & 4 != 0
                    }
                );
                assert_eq!(
                    LevelRecord::parse(&bytes).unwrap().as_bytes()[288..292],
                    bytes[288..292]
                );
            }
        }
    }

    #[test]
    fn steel_uses_split_x_bits_low_y_bits_and_high_nibble_width() {
        let mut bytes = fixture();
        bytes[1888 + 4 * 4..1892 + 4 * 4].copy_from_slice(&[0x12, 0x83, 0x25, 0]);
        bytes[1888 + 31 * 4..1892 + 31 * 4].copy_from_slice(&[0xff, 0xff, 0xff, 0]);
        let record = LevelRecord::parse(&bytes).unwrap();
        assert_eq!(
            record.steel().collect::<Vec<_>>(),
            vec![
                SteelArea {
                    slot: 4,
                    x: 132,
                    y: 12,
                    width: 12,
                    height: 24
                },
                SteelArea {
                    slot: 31,
                    x: 2028,
                    y: 508,
                    width: 64,
                    height: 64
                },
            ]
        );
    }

    #[test]
    fn steel_minimum_area_can_have_only_a_nonzero_position_or_size() {
        for (raw, expected) in [
            (
                [0, 0, 1, 0],
                SteelArea {
                    slot: 0,
                    x: -16,
                    y: 0,
                    width: 4,
                    height: 8,
                },
            ),
            (
                [0, 1, 0, 0],
                SteelArea {
                    slot: 0,
                    x: -16,
                    y: 4,
                    width: 4,
                    height: 4,
                },
            ),
            (
                [0, 0x80, 0, 0],
                SteelArea {
                    slot: 0,
                    x: -12,
                    y: 0,
                    width: 4,
                    height: 4,
                },
            ),
        ] {
            let mut bytes = fixture();
            bytes[1888..1892].copy_from_slice(&raw);
            assert_eq!(
                LevelRecord::parse(&bytes).unwrap().steel().next(),
                Some(expected)
            );
        }
    }

    #[test]
    fn nonzero_steel_reserved_bytes_are_rejected_before_any_iteration() {
        for slot in [0, 17, 31] {
            for value in [1, 0x80, 0xff] {
                let mut bytes = fixture();
                let offset = 1888 + slot * 4 + 3;
                bytes[offset] = value;
                assert_eq!(
                    LevelRecord::parse(&bytes),
                    Err(LevelError::UnsupportedSteelByte { offset, value })
                );
            }
        }
    }

    #[test]
    fn fully_populated_records_preserve_all_slot_counts_and_can_be_reiterated() {
        let mut bytes = fixture();
        for slot in 0..32 {
            bytes[32 + slot * 8 + 7] = 0x0f;
            bytes[1888 + slot * 4] = 1;
        }
        bytes[288..1888].fill(0);
        let record = LevelRecord::parse(&bytes).unwrap();
        for _ in 0..2 {
            assert_eq!(
                record.objects().map(|p| p.slot).collect::<Vec<_>>(),
                (0..32).collect::<Vec<_>>()
            );
            assert_eq!(
                record.terrains().map(|p| p.slot).collect::<Vec<_>>(),
                (0..400).collect::<Vec<_>>()
            );
            assert_eq!(
                record.steel().map(|p| p.slot).collect::<Vec<_>>(),
                (0..32).collect::<Vec<_>>()
            );
        }
    }
}
