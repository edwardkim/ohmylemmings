use core::fmt;

/// Raw GROUND file size; unlike VGAGR, this is not a compressed DAT container.
pub const GROUND_RECORD_SIZE: usize = 1056;
pub const GROUND_OBJECT_COUNT: usize = 16;
pub const GROUND_TERRAIN_COUNT: usize = 64;

/// Three independent on-disk banks. Custom is terrain; Standard and Preview
/// form the lower and upper eight object colors in the pinned TS reference.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GroundPalette {
    Custom,
    Standard,
    Preview,
}
impl GroundPalette {
    const fn index(self) -> usize {
        match self {
            Self::Custom => 0,
            Self::Standard => 1,
            Self::Preview => 2,
        }
    }
}

/// Validated VGA DAC components, each in 0..=63. No alpha or transparency policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rgb6 {
    components: [u8; 3],
}
impl Rgb6 {
    pub const fn components(self) -> [u8; 3] {
        self.components
    }
    /// Lemmings.ts conversion: 0..=63 becomes 0..=252.
    pub fn to_rgb8_shifted(self) -> [u8; 3] {
        self.components.map(|v| v << 2)
    }
    /// Lemmix conversion: floor(component * 255 / 63), giving 0..=255.
    pub fn to_rgb8_full_range(self) -> [u8; 3] {
        self.components.map(|v| (u16::from(v) * 255 / 63) as u8)
    }
}

/// All fields of one 28-byte slot, including uninterpreted flags and words.
/// Trigger fields remain in disk units; use `trigger_rect` for local pixels.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GroundObject {
    pub slot: usize,
    pub animation_flags: u16,
    pub first_frame: u8,
    pub frame_count: u8,
    pub width: u8,
    pub height: u8,
    pub frame_size: u16,
    /// Relative to each object animation frame image.
    pub mask_offset: u16,
    pub unknown: [u16; 3],
    pub trigger_left: u16,
    pub trigger_top: u16,
    pub trigger_width: u8,
    pub trigger_height: u8,
    pub trigger_effect: u8,
    pub image_offset: u16,
    /// Byte address of the preview image, not an animation frame index.
    pub preview_offset: u16,
    pub sound_effect: u8,
}
/// Trigger rectangle in pixels relative to the object image.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObjectTrigger {
    pub left: i32,
    pub top: i32,
    pub width: u16,
    pub height: u16,
}
impl GroundObject {
    /// True only when every disk field is zero; slot numbering is retained.
    pub fn is_empty(self) -> bool {
        self == Self {
            slot: self.slot,
            ..Self::default()
        }
    }
    pub fn animation_loops(self) -> bool {
        self.animation_flags & 1 == 0
    }
    /// Pinned TS convention: left/top in units of four, with top biased by -4.
    /// i32 accommodates all u16 disk coordinates, including a zero top (-4).
    pub fn trigger_rect(self) -> ObjectTrigger {
        ObjectTrigger {
            left: i32::from(self.trigger_left) * 4,
            top: i32::from(self.trigger_top) * 4 - 4,
            width: u16::from(self.trigger_width) * 4,
            height: u16::from(self.trigger_height) * 4,
        }
    }
}
/// One eight-byte terrain slot; image and mask addresses are both absolute
/// within the decompressed VGAGR terrain section.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GroundTerrain {
    pub slot: usize,
    pub width: u8,
    pub height: u8,
    pub image_offset: u16,
    pub mask_offset: u16,
    pub unknown: u16,
}
impl GroundTerrain {
    /// True only for an all-zero disk slot, including its unknown word.
    pub fn is_empty(self) -> bool {
        self == Self {
            slot: self.slot,
            ..Self::default()
        }
    }
}

/// Borrowed 1056-byte GROUND metadata with checked six-bit EGA/VGA values.
///
/// Parsing checks length and palettes. It preserves every object/terrain slot,
/// including empty or unusual metadata. Frame dimensions, animation ranges,
/// trigger effects and image addresses are not certified without VGAGR data.
/// No graphics-set number is inferred or authenticated from these bytes.
///
/// ```
/// use lemmings_resources::{GroundPalette, GroundRecord, GROUND_RECORD_SIZE};
/// let bytes = [0; GROUND_RECORD_SIZE];
/// let ground = GroundRecord::parse(&bytes)?;
/// assert_eq!(ground.objects().len(), 16);
/// assert!(ground.terrain(63).unwrap().is_empty());
/// assert_eq!(ground.vga_palette(GroundPalette::Custom)[0].components(), [0; 3]);
/// # Ok::<(), lemmings_resources::GroundError>(())
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GroundRecord<'a> {
    bytes: &'a [u8; GROUND_RECORD_SIZE],
}
impl<'a> GroundRecord<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, GroundError> {
        let bytes: &[u8; GROUND_RECORD_SIZE] =
            bytes.try_into().map_err(|_| GroundError::Length {
                actual: bytes.len(),
            })?;
        for (i, &value) in bytes[960..].iter().enumerate() {
            if value > 63 {
                return Err(GroundError::PaletteValue {
                    offset: 960 + i,
                    value,
                });
            }
        }
        Ok(Self { bytes })
    }
    pub const fn as_bytes(self) -> &'a [u8; GROUND_RECORD_SIZE] {
        self.bytes
    }
    /// Lookup by original slot, with `None` for any out-of-range index.
    pub fn object(self, index: usize) -> Option<GroundObject> {
        if index >= GROUND_OBJECT_COUNT {
            return None;
        }
        let b = &self.bytes[index * 28..index * 28 + 28];
        Some(GroundObject {
            slot: index,
            animation_flags: le(b, 0),
            first_frame: b[2],
            frame_count: b[3],
            width: b[4],
            height: b[5],
            frame_size: le(b, 6),
            mask_offset: le(b, 8),
            unknown: [le(b, 10), le(b, 12), le(b, 25)],
            trigger_left: le(b, 14),
            trigger_top: le(b, 16),
            trigger_width: b[18],
            trigger_height: b[19],
            trigger_effect: b[20],
            image_offset: le(b, 21),
            preview_offset: le(b, 23),
            sound_effect: b[27],
        })
    }
    /// Lookup by original slot, preserving absolute mask addresses.
    pub fn terrain(self, index: usize) -> Option<GroundTerrain> {
        if index >= GROUND_TERRAIN_COUNT {
            return None;
        }
        let b = &self.bytes[448 + index * 8..456 + index * 8];
        Some(GroundTerrain {
            slot: index,
            width: b[0],
            height: b[1],
            image_offset: le(b, 2),
            mask_offset: le(b, 4),
            unknown: le(b, 6),
        })
    }
    pub fn objects(
        self,
    ) -> impl ExactSizeIterator<Item = GroundObject> + core::iter::FusedIterator + 'a {
        (0..GROUND_OBJECT_COUNT).map(move |i| self.object(i).unwrap())
    }
    pub fn terrains(
        self,
    ) -> impl ExactSizeIterator<Item = GroundTerrain> + core::iter::FusedIterator + 'a {
        (0..GROUND_TERRAIN_COUNT).map(move |i| self.terrain(i).unwrap())
    }
    /// Raw EGA hardware selectors; not VGA indices or RGB triples.
    pub fn ega_palette(self, bank: GroundPalette) -> &'a [u8; 8] {
        let start = 960 + bank.index() * 8;
        self.bytes[start..start + 8].try_into().unwrap()
    }
    /// Raw six-bit RGB, in disk order. Select an explicit scale when displaying.
    pub fn vga_palette(self, bank: GroundPalette) -> [Rgb6; 8] {
        core::array::from_fn(|i| {
            let start = 984 + bank.index() * 24 + i * 3;
            Rgb6 {
                components: [
                    self.bytes[start],
                    self.bytes[start + 1],
                    self.bytes[start + 2],
                ],
            }
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GroundError {
    Length { actual: usize },
    PaletteValue { offset: usize, value: u8 },
}
impl fmt::Display for GroundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Length { actual } => write!(
                f,
                "GROUND length {actual}; expected {GROUND_RECORD_SIZE} bytes"
            ),
            Self::PaletteValue { offset, value } => write!(
                f,
                "GROUND palette byte {offset} has value {value}; expected 0..=63"
            ),
        }
    }
}
impl core::error::Error for GroundError {}

fn le(b: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([b[offset], b[offset + 1]])
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use std::vec;

    fn word(b: &mut [u8], offset: usize, value: u16) {
        b[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }

    #[test]
    fn exact_length_is_required_for_every_truncated_prefix_and_extra_bytes() {
        for actual in 0..GROUND_RECORD_SIZE {
            assert_eq!(
                GroundRecord::parse(&vec![0; actual]),
                Err(GroundError::Length { actual })
            );
        }
        for actual in [1057, 2048] {
            assert_eq!(
                GroundRecord::parse(&vec![0; actual]),
                Err(GroundError::Length { actual })
            );
        }
    }

    #[test]
    fn records_borrow_all_bytes_and_preserve_empty_slot_numbers() {
        let b = [0; 1056];
        let ground = GroundRecord::parse(&b).unwrap();
        assert_eq!(ground.as_bytes().as_ptr(), b.as_ptr());
        let mut objects = ground.objects();
        assert_eq!(objects.len(), 16);
        for slot in 0..16 {
            let o = objects.next().unwrap();
            assert_eq!(o.slot, slot);
            assert!(o.is_empty());
        }
        assert_eq!(objects.len(), 0);
        assert_eq!(objects.next(), None);
        assert_eq!(objects.next(), None);
        let mut terrains = ground.terrains();
        assert_eq!(terrains.len(), 64);
        for slot in 0..64 {
            let t = terrains.next().unwrap();
            assert_eq!(t.slot, slot);
            assert!(t.is_empty());
        }
        assert_eq!(terrains.next(), None);
        assert_eq!(terrains.next(), None);
    }

    #[test]
    fn object_fields_include_unaligned_little_endian_words_and_unknowns() {
        let mut b = [0; 1056];
        let start = 15 * 28;
        for (offset, value) in [
            (0, 0x8003),
            (6, 0x1234),
            (8, 0x2345),
            (10, 0x3456),
            (12, 0x4567),
            (14, 0x5678),
            (16, 0x6789),
            (21, 0x789a),
            (23, 0x89ab),
            (25, 0x9abc),
        ] {
            word(&mut b, start + offset, value);
        }
        for (offset, value) in [
            (2, 7),
            (3, 9),
            (4, 48),
            (5, 25),
            (18, 13),
            (19, 17),
            (20, 0xfe),
            (27, 0xfd),
        ] {
            b[start + offset] = value;
        }
        let g = GroundRecord::parse(&b).unwrap();
        let object = g.object(15).unwrap();
        assert_eq!(
            object,
            GroundObject {
                slot: 15,
                animation_flags: 0x8003,
                first_frame: 7,
                frame_count: 9,
                width: 48,
                height: 25,
                frame_size: 0x1234,
                mask_offset: 0x2345,
                unknown: [0x3456, 0x4567, 0x9abc],
                trigger_left: 0x5678,
                trigger_top: 0x6789,
                trigger_width: 13,
                trigger_height: 17,
                trigger_effect: 0xfe,
                image_offset: 0x789a,
                preview_offset: 0x89ab,
                sound_effect: 0xfd
            }
        );
        assert!(!object.is_empty());
        assert!(g.object(14).unwrap().is_empty());
        assert_eq!(g.as_bytes(), &b);
    }

    #[test]
    fn terrain_mask_is_absolute_and_unknown_words_are_preserved() {
        let mut b = [0; 1056];
        let start = 448 + 63 * 8;
        b[start] = 255;
        b[start + 1] = 127;
        word(&mut b, start + 2, 0xabcd);
        word(&mut b, start + 4, 0x0123);
        word(&mut b, start + 6, 0xfedc);
        let g = GroundRecord::parse(&b).unwrap();
        assert_eq!(
            g.terrain(63).unwrap(),
            GroundTerrain {
                slot: 63,
                width: 255,
                height: 127,
                image_offset: 0xabcd,
                mask_offset: 0x0123,
                unknown: 0xfedc
            }
        );
        assert!(!g.terrain(63).unwrap().is_empty());
        assert!(g.terrain(62).unwrap().is_empty());
    }

    #[test]
    fn out_of_range_slot_access_never_multiplies_untrusted_indexes() {
        let b = [0; 1056];
        let g = GroundRecord::parse(&b).unwrap();
        for index in [16, 64, usize::MAX] {
            assert_eq!(g.object(index), None);
        }
        for index in [64, 65, usize::MAX] {
            assert_eq!(g.terrain(index), None);
        }
    }

    #[test]
    fn all_palette_banks_preserve_order_and_six_bit_values() {
        let mut b = [0; 1056];
        for (i, v) in b[960..].iter_mut().enumerate() {
            *v = (i % 64) as u8;
        }
        let g = GroundRecord::parse(&b).unwrap();
        for (i, bank) in [
            GroundPalette::Custom,
            GroundPalette::Standard,
            GroundPalette::Preview,
        ]
        .into_iter()
        .enumerate()
        {
            assert_eq!(g.ega_palette(bank), &b[960 + 8 * i..968 + 8 * i]);
            assert_eq!(g.ega_palette(bank).as_ptr(), b[960 + 8 * i..].as_ptr());
            for (j, c) in g.vga_palette(bank).into_iter().enumerate() {
                assert_eq!(
                    c.components(),
                    b[984 + 24 * i + 3 * j..987 + 24 * i + 3 * j]
                );
            }
        }
    }

    #[test]
    fn invalid_ega_and_vga_components_report_the_exact_byte_offset() {
        for offset in 960..1056 {
            for value in [64, 255] {
                let mut b = [0; 1056];
                b[offset] = value;
                assert_eq!(
                    GroundRecord::parse(&b),
                    Err(GroundError::PaletteValue { offset, value })
                );
            }
        }
    }

    #[test]
    fn rgb_scaling_explicitly_distinguishes_both_reference_conventions() {
        let mut b = [0; 1056];
        for value in 0..64u8 {
            b[984..987].fill(value);
            let c = GroundRecord::parse(&b)
                .unwrap()
                .vga_palette(GroundPalette::Custom)[0];
            assert_eq!(c.to_rgb8_shifted(), [value * 4; 3]);
            assert_eq!(
                c.to_rgb8_full_range(),
                [(u16::from(value) * 255 / 63) as u8; 3]
            );
        }
    }

    #[test]
    fn trigger_coordinates_scale_without_unsigned_underflow_or_truncation() {
        let mut b = [0; 1056];
        word(&mut b, 14, u16::MAX);
        b[18] = 255;
        b[19] = 255;
        let o = GroundRecord::parse(&b).unwrap().object(0).unwrap();
        assert_eq!(
            o.trigger_rect(),
            ObjectTrigger {
                left: 262140,
                top: -4,
                width: 1020,
                height: 1020
            }
        );
        word(&mut b, 16, u16::MAX);
        assert_eq!(
            GroundRecord::parse(&b)
                .unwrap()
                .object(0)
                .unwrap()
                .trigger_rect()
                .top,
            262136
        );
    }

    #[test]
    fn only_loop_bit_is_interpreted_and_other_bits_remain_available() {
        for flags in [0, 1, 2, 3, 0xfffe, 0xffff] {
            let mut b = [0; 1056];
            word(&mut b, 0, flags);
            let o = GroundRecord::parse(&b).unwrap().object(0).unwrap();
            assert_eq!(o.animation_flags, flags);
            assert_eq!(o.animation_loops(), flags & 1 == 0);
        }
    }

    #[test]
    fn zero_dimensions_do_not_hide_nonzero_unknown_metadata() {
        let mut b = [0; 1056];
        word(&mut b, 25, 0xbeef);
        word(&mut b, 448 + 6, 0xcafe);
        let g = GroundRecord::parse(&b).unwrap();
        assert!(!g.object(0).unwrap().is_empty());
        assert!(!g.terrain(0).unwrap().is_empty());
        assert_eq!(g.object(0).unwrap().frame_count, 0);
    }

    #[test]
    fn errors_describe_length_or_palette_location() {
        assert_eq!(
            std::format!("{}", GroundError::Length { actual: 1 }),
            "GROUND length 1; expected 1056 bytes"
        );
        assert_eq!(
            std::format!(
                "{}",
                GroundError::PaletteValue {
                    offset: 1055,
                    value: 64
                }
            ),
            "GROUND palette byte 1055 has value 64; expected 0..=63"
        );
    }
}
