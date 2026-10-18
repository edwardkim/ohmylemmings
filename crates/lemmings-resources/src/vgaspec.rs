use crate::{DatError, DatSection, DatSections, DecompressError, IndexedPixel, Rgb6};
use core::fmt;

/// Width of the special-background image before level placement.
pub const VGASPEC_WIDTH: usize = 960;
/// Height of four 40-row strips, stacked from top to bottom.
pub const VGASPEC_HEIGHT: usize = 160;
/// Four strips, each with three 4800-byte color planes. There is no mask plane.
pub const VGASPEC_PLANAR_SIZE: usize = 57600;
const HEADER_SIZE: usize = 40;
const STRIPE_SIZE: usize = 14400;
const STRIPE_COUNT: usize = 4;
const STRIPE_HEIGHT: usize = 40;

/// One complete DAT section. Parsing validates framing/XOR and the whole tail;
/// compressed commands and the inner palette/RLE record are checked on decode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VgaSpecFile<'a> {
    section: DatSection<'a>,
}
impl<'a> VgaSpecFile<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, VgaSpecError> {
        let mut first = None;
        let mut count = 0;
        for (index, section) in DatSections::new(bytes).enumerate() {
            let section = section.map_err(|error| VgaSpecError::Dat {
                section: index,
                error,
            })?;
            if index == 0 {
                first = Some(section);
            }
            count += 1;
        }
        if count != 1 {
            return Err(VgaSpecError::SectionCount { actual: count });
        }
        Ok(Self {
            section: first.unwrap(),
        })
    }
    pub fn decoded_size(self) -> usize {
        usize::from(self.section.header().decompressed_size)
    }
    /// Output must match the declared DAT size. A size error leaves it unchanged;
    /// other decompression or inner-record errors may change it: discard on error.
    /// The returned record borrows only the caller's buffer, not the input DAT.
    pub fn decode_into<'b>(self, output: &'b mut [u8]) -> Result<VgaSpecRecord<'b>, VgaSpecError> {
        self.section
            .decompress_into(output)
            .map_err(VgaSpecError::Decompress)?;
        VgaSpecRecord::parse(output)
    }
}

/// Borrowed inner record: 24 VGA bytes, 8 EGA selectors, 8 unknown bytes, then
/// four exactly-sized RLE strips. Every packet and the complete tail is checked.
/// Unknown header bytes are preserved without interpreting or limiting them.
///
/// ```
/// use lemmings_resources::{VgaSpecRecord, VgaSpecError};
/// let truncated = [0; 39];
/// assert_eq!(VgaSpecRecord::parse(&truncated),
///     Err(VgaSpecError::HeaderSize { actual: 39 }));
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VgaSpecRecord<'a> {
    bytes: &'a [u8],
    starts: [usize; STRIPE_COUNT],
}
impl<'a> VgaSpecRecord<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, VgaSpecError> {
        if bytes.len() < HEADER_SIZE {
            return Err(VgaSpecError::HeaderSize {
                actual: bytes.len(),
            });
        }
        for (offset, &value) in bytes[..32].iter().enumerate() {
            if value > 63 {
                return Err(VgaSpecError::PaletteValue { offset, value });
            }
        }
        let mut starts = [0; STRIPE_COUNT];
        let mut offset = HEADER_SIZE;
        for (stripe, start) in starts.iter_mut().enumerate() {
            *start = offset;
            offset = unpack_stripe(bytes, offset, stripe, None)?;
        }
        if offset != bytes.len() {
            return Err(VgaSpecError::TrailingData {
                offset,
                remaining: bytes.len() - offset,
            });
        }
        Ok(Self { bytes, starts })
    }
    pub const fn as_bytes(self) -> &'a [u8] {
        self.bytes
    }
    /// Eight validated six-bit RGB colors; conversion policy remains explicit.
    pub fn vga_palette(self) -> [Rgb6; 8] {
        core::array::from_fn(|i| {
            Rgb6::from_components([
                self.bytes[i * 3],
                self.bytes[i * 3 + 1],
                self.bytes[i * 3 + 2],
            ])
            .expect("validated VGA palette")
        })
    }
    pub fn ega_palette(self) -> &'a [u8; 8] {
        self.bytes[24..32].try_into().unwrap()
    }
    pub fn unknown_palette_bytes(self) -> &'a [u8; 8] {
        self.bytes[32..40].try_into().unwrap()
    }
    /// Decode strip 0..3 into exactly 14400 bytes (three color planes).
    /// Invalid index/length leaves output unchanged; the record's packets have
    /// already been validated, so no packet error can occur for this immutable view.
    pub fn decode_stripe_into(self, index: usize, output: &mut [u8]) -> Result<(), VgaSpecError> {
        let start = *self
            .starts
            .get(index)
            .ok_or(VgaSpecError::StripeIndex { index })?;
        check_output(output.len(), STRIPE_SIZE)?;
        unpack_stripe(self.bytes, start, index, Some(output))?;
        Ok(())
    }
    /// Decode all four strips into an exact 57600-byte caller buffer. This view
    /// retains strip-major planar order; it does not rearrange into full-image planes.
    /// The returned image borrows only this output, independently of the record.
    pub fn decode_into<'b>(self, output: &'b mut [u8]) -> Result<VgaSpecImage<'b>, VgaSpecError> {
        check_output(output.len(), VGASPEC_PLANAR_SIZE)?;
        for (index, stripe) in output.chunks_exact_mut(STRIPE_SIZE).enumerate() {
            self.decode_stripe_into(index, stripe)?;
        }
        Ok(VgaSpecImage {
            planes: (&*output).try_into().expect("checked planar length"),
        })
    }
}

/// 960x160 image over a caller-owned strip-major planar buffer. Index zero is
/// transparent; indices 1..7 are opaque, independently of their RGB color values.
/// Coordinates start at (0,0) in this image; level placement belongs to composition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VgaSpecImage<'a> {
    planes: &'a [u8; VGASPEC_PLANAR_SIZE],
}
impl<'a> VgaSpecImage<'a> {
    pub const fn planar_bytes(self) -> &'a [u8; VGASPEC_PLANAR_SIZE] {
        self.planes
    }
    /// Rejects caller coordinates before multiplying them.
    pub fn pixel(self, x: usize, y: usize) -> Option<IndexedPixel> {
        if x >= VGASPEC_WIDTH || y >= VGASPEC_HEIGHT {
            return None;
        }
        Some(self.pixel_at(x, y))
    }
    /// Decode row-major indices and opacity into exactly 153600 pixels. A size
    /// error leaves every output pixel unchanged; valid output needs no allocation.
    pub fn decode_into(self, output: &mut [IndexedPixel]) -> Result<(), VgaSpecError> {
        check_output(output.len(), VGASPEC_WIDTH * VGASPEC_HEIGHT)?;
        for (index, pixel) in output.iter_mut().enumerate() {
            *pixel = self.pixel_at(index % VGASPEC_WIDTH, index / VGASPEC_WIDTH);
        }
        Ok(())
    }
    fn pixel_at(self, x: usize, y: usize) -> IndexedPixel {
        let stripe = y / STRIPE_HEIGHT;
        let row = y % STRIPE_HEIGHT;
        let offset = stripe * STRIPE_SIZE + row * (VGASPEC_WIDTH / 8) + x / 8;
        let bit = 0x80 >> (x % 8);
        let mut color = 0;
        for plane in 0..3 {
            if self.planes[offset + plane * (STRIPE_SIZE / 3)] & bit != 0 {
                color |= 1 << plane;
            }
        }
        IndexedPixel {
            color,
            opaque: color != 0,
        }
    }
}

/// Packet offsets are relative to the inner DAT-decompressed record (including
/// its 40-byte header); stripe indices are 0..3. Wrapped DAT/decompression errors
/// retain their original compressed-file offsets and are exposed through source().
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VgaSpecError {
    Dat {
        section: usize,
        error: DatError,
    },
    SectionCount {
        actual: usize,
    },
    Decompress(DecompressError),
    HeaderSize {
        actual: usize,
    },
    PaletteValue {
        offset: usize,
        value: u8,
    },
    StripeIndex {
        index: usize,
    },
    TruncatedPacket {
        stripe: usize,
        offset: usize,
        requested: usize,
        remaining: usize,
    },
    StripeLength {
        stripe: usize,
        offset: usize,
        actual: usize,
    },
    StripeOverrun {
        stripe: usize,
        offset: usize,
        length: usize,
        remaining: usize,
    },
    TrailingData {
        offset: usize,
        remaining: usize,
    },
    OutputSize {
        expected: usize,
        actual: usize,
    },
}
impl fmt::Display for VgaSpecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dat { section, error } => write!(f, "VGASPEC section {section}: {error}"),
            Self::SectionCount { actual } => {
                write!(f, "VGASPEC has {actual} DAT sections; expected 1")
            }
            Self::Decompress(error) => write!(f, "VGASPEC DAT decode: {error}"),
            Self::HeaderSize { actual } => write!(
                f,
                "VGASPEC record length {actual}; need a {HEADER_SIZE}-byte header"
            ),
            Self::PaletteValue { offset, value } => write!(
                f,
                "VGASPEC palette byte {offset} has value {value}; expected 0..=63"
            ),
            Self::StripeIndex { index } => {
                write!(f, "VGASPEC stripe {index} outside 0..{STRIPE_COUNT}")
            }
            Self::TruncatedPacket {
                stripe,
                offset,
                requested,
                remaining,
            } => write!(
                f,
                "VGASPEC stripe {stripe} at byte {offset}: need {requested} bytes, {remaining} remain"
            ),
            Self::StripeLength {
                stripe,
                offset,
                actual,
            } => write!(
                f,
                "VGASPEC stripe {stripe} ends at byte {offset} after {actual} decoded bytes; expected {STRIPE_SIZE}"
            ),
            Self::StripeOverrun {
                stripe,
                offset,
                length,
                remaining,
            } => write!(
                f,
                "VGASPEC stripe {stripe} at byte {offset}: packet length {length} exceeds {remaining} remaining output bytes"
            ),
            Self::TrailingData { offset, remaining } => {
                write!(f, "VGASPEC has {remaining} trailing bytes at byte {offset}")
            }
            Self::OutputSize { expected, actual } => {
                write!(f, "VGASPEC output length {actual}; expected {expected}")
            }
        }
    }
}
impl core::error::Error for VgaSpecError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Dat { error, .. } => Some(error),
            Self::Decompress(error) => Some(error),
            _ => None,
        }
    }
}
fn check_output(actual: usize, expected: usize) -> Result<(), VgaSpecError> {
    if actual != expected {
        return Err(VgaSpecError::OutputSize { expected, actual });
    }
    Ok(())
}

// A single packet walker validates without an output buffer and later decodes
// the same immutable bytes. Parsing needs no scratch array or pixel allocation.
fn unpack_stripe(
    bytes: &[u8],
    mut offset: usize,
    stripe: usize,
    mut output: Option<&mut [u8]>,
) -> Result<usize, VgaSpecError> {
    let mut produced = 0;
    loop {
        let command_offset = offset;
        let command = *bytes.get(offset).ok_or(VgaSpecError::TruncatedPacket {
            stripe,
            offset,
            requested: 1,
            remaining: 0,
        })?;
        offset += 1;
        if command == 128 {
            if produced != STRIPE_SIZE {
                return Err(VgaSpecError::StripeLength {
                    stripe,
                    offset: command_offset,
                    actual: produced,
                });
            }
            return Ok(offset);
        }
        let literal = command < 128;
        let length = if literal {
            usize::from(command) + 1
        } else {
            257 - usize::from(command)
        };
        if length > STRIPE_SIZE - produced {
            return Err(VgaSpecError::StripeOverrun {
                stripe,
                offset: command_offset,
                length,
                remaining: STRIPE_SIZE - produced,
            });
        }
        let requested = if literal { length } else { 1 };
        let remaining = bytes.len() - offset;
        if requested > remaining {
            return Err(VgaSpecError::TruncatedPacket {
                stripe,
                offset,
                requested,
                remaining,
            });
        }
        if let Some(out) = output.as_deref_mut() {
            if literal {
                out[produced..produced + length].copy_from_slice(&bytes[offset..offset + length]);
            } else {
                out[produced..produced + length].fill(bytes[offset]);
            }
        }
        offset += requested;
        produced += length;
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use std::{format, string::String, vec, vec::Vec};

    fn fill(output: &mut Vec<u8>, mut count: usize, value: u8) {
        while count != 0 {
            let length = count.min(128);
            if length == 1 {
                output.extend_from_slice(&[0, value]);
            } else {
                output.extend_from_slice(&[(257 - length) as u8, value]);
            }
            count -= length;
        }
    }
    fn stripe(value: u8) -> Vec<u8> {
        let mut b = Vec::new();
        fill(&mut b, STRIPE_SIZE, value);
        b.push(128);
        b
    }
    fn record() -> Vec<u8> {
        let mut b = vec![0; 40];
        for _ in 0..4 {
            b.extend(stripe(0));
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
    fn every_incomplete_prefix_is_rejected_and_exact_record_is_borrowed() {
        let b = record();
        for len in 0..b.len() {
            assert!(VgaSpecRecord::parse(&b[..len]).is_err(), "prefix {len}");
        }
        let parsed = VgaSpecRecord::parse(&b).unwrap();
        assert_eq!(parsed.as_bytes().as_ptr(), b.as_ptr());
        assert_eq!(parsed.as_bytes(), b);
        assert_eq!(
            VgaSpecRecord::parse(&[0; 39]),
            Err(VgaSpecError::HeaderSize { actual: 39 })
        );
    }

    #[test]
    fn palettes_preserve_eight_rgb_colors_ega_selectors_and_unknown_bytes() {
        let mut b = record();
        for (i, v) in b[..32].iter_mut().enumerate() {
            *v = (i * 2) as u8;
        }
        b[32..40].copy_from_slice(&[0, 64, 128, 255, 1, 2, 3, 4]);
        let r = VgaSpecRecord::parse(&b).unwrap();
        for (i, c) in r.vga_palette().into_iter().enumerate() {
            assert_eq!(c.components(), b[i * 3..i * 3 + 3]);
            assert_eq!(
                c.to_rgb8_shifted(),
                [
                    (i * 6) as u8 * 4,
                    (i * 6 + 2) as u8 * 4,
                    (i * 6 + 4) as u8 * 4
                ]
            );
        }
        assert_eq!(r.ega_palette(), &b[24..32]);
        assert_eq!(r.unknown_palette_bytes(), &b[32..40]);
        assert_eq!(r.unknown_palette_bytes().as_ptr(), b[32..].as_ptr());
        b[..32].fill(63);
        assert!(VgaSpecRecord::parse(&b).is_ok());
    }

    #[test]
    fn invalid_vga_or_ega_values_identify_each_byte_but_unknown_fields_are_raw() {
        for offset in 0..32 {
            for value in [64, 255] {
                let mut b = record();
                b[offset] = value;
                assert_eq!(
                    VgaSpecRecord::parse(&b),
                    Err(VgaSpecError::PaletteValue { offset, value })
                );
            }
        }
    }

    #[test]
    fn every_literal_and_repeat_code_decodes_its_full_count_including_marker_values() {
        for code in 0..=255u8 {
            if code == 128 {
                continue;
            }
            let count = if code < 128 {
                usize::from(code) + 1
            } else {
                257 - usize::from(code)
            };
            let mut b = vec![0; 40];
            b.push(code);
            if code < 128 {
                b.extend(vec![128; count]);
            } else {
                b.push(128);
            }
            fill(&mut b, STRIPE_SIZE - count, 0);
            b.push(128);
            for _ in 1..4 {
                b.extend(stripe(0));
            }
            let r = VgaSpecRecord::parse(&b).unwrap();
            let mut out = vec![99; STRIPE_SIZE];
            r.decode_stripe_into(0, &mut out).unwrap();
            assert_eq!(&out[..count], vec![128; count]);
            assert!(out[count..].iter().all(|&v| v == 0));
        }
    }

    #[test]
    fn stripe_planes_rows_and_transparency_produce_the_full_960_by_160_image() {
        let mut b = vec![0; 40];
        let mut expected = Vec::new();
        for values in [[0x55, 0x33, 0x0f], [0xff, 0, 0], [0, 0xff, 0], [0, 0, 0xff]] {
            for value in values {
                fill(&mut b, 4800, value);
                expected.extend(vec![value; 4800]);
            }
            b.push(128);
        }
        let mut planes = vec![99; VGASPEC_PLANAR_SIZE];
        let ptr = planes.as_ptr();
        let image = VgaSpecRecord::parse(&b)
            .unwrap()
            .decode_into(&mut planes)
            .unwrap();
        assert_eq!(image.planar_bytes().as_slice(), expected);
        assert_eq!(image.planar_bytes().as_ptr(), ptr);
        let mut pixels = vec![IndexedPixel::default(); VGASPEC_WIDTH * VGASPEC_HEIGHT];
        image.decode_into(&mut pixels).unwrap();
        for (i, pixel) in pixels.iter().enumerate() {
            let y = i / 960;
            let color = if y < 40 {
                (i % 960 % 8) as u8
            } else {
                1 << ((y / 40) - 1)
            };
            assert_eq!(
                *pixel,
                IndexedPixel {
                    color,
                    opaque: color != 0
                }
            );
        }
        for (x, y, color) in [
            (0, 0, 0),
            (7, 0, 7),
            (8, 0, 0),
            (959, 39, 7),
            (0, 40, 1),
            (959, 79, 1),
            (0, 80, 2),
            (0, 120, 4),
            (959, 159, 4),
        ] {
            assert_eq!(
                image.pixel(x, y),
                Some(IndexedPixel {
                    color,
                    opaque: color != 0
                })
            );
        }
    }

    #[test]
    fn short_literals_repeats_and_missing_opcodes_report_exact_packet_offsets() {
        for count in [0, 1, 127] {
            let mut b = vec![0; 40];
            b.push(127);
            b.extend(vec![0; count]);
            assert_eq!(
                VgaSpecRecord::parse(&b),
                Err(VgaSpecError::TruncatedPacket {
                    stripe: 0,
                    offset: 41,
                    requested: 128,
                    remaining: count
                })
            );
        }
        let mut b = vec![0; 40];
        b.push(129);
        assert_eq!(
            VgaSpecRecord::parse(&b),
            Err(VgaSpecError::TruncatedPacket {
                stripe: 0,
                offset: 41,
                requested: 1,
                remaining: 0
            })
        );
        assert_eq!(
            VgaSpecRecord::parse(&[0; 40]),
            Err(VgaSpecError::TruncatedPacket {
                stripe: 0,
                offset: 40,
                requested: 1,
                remaining: 0
            })
        );
    }

    #[test]
    fn each_early_stripe_terminator_is_rejected_including_in_the_last_stripe() {
        for stripe_index in 0..4 {
            for actual in [0, 1, 14399] {
                let mut b = vec![0; 40];
                for _ in 0..stripe_index {
                    b.extend(stripe(0));
                }
                fill(&mut b, actual, 0);
                let offset = b.len();
                b.push(128);
                assert_eq!(
                    VgaSpecRecord::parse(&b),
                    Err(VgaSpecError::StripeLength {
                        stripe: stripe_index,
                        offset,
                        actual
                    })
                );
            }
        }
    }

    #[test]
    fn packets_must_not_cross_a_stripe_boundary_even_when_more_input_follows() {
        for (produced, code, length) in [(14399, 255, 2), (14399, 1, 2), (14400, 129, 128)] {
            let mut b = vec![0; 40];
            fill(&mut b, produced, 0);
            let offset = b.len();
            b.extend([code, 0, 0]);
            assert_eq!(
                VgaSpecRecord::parse(&b),
                Err(VgaSpecError::StripeOverrun {
                    stripe: 0,
                    offset,
                    length,
                    remaining: 14400 - produced
                })
            );
        }
    }

    #[test]
    fn exactly_four_terminators_and_no_trailing_bytes_are_required() {
        let mut b = record();
        b.pop();
        assert_eq!(
            VgaSpecRecord::parse(&b),
            Err(VgaSpecError::TruncatedPacket {
                stripe: 3,
                offset: b.len(),
                requested: 1,
                remaining: 0
            })
        );
        for tail in [&[128][..], &[0, 0][..], &[255][..]] {
            let mut b = record();
            let offset = b.len();
            b.extend(tail);
            assert_eq!(
                VgaSpecRecord::parse(&b),
                Err(VgaSpecError::TrailingData {
                    offset,
                    remaining: tail.len()
                })
            );
        }
    }

    #[test]
    fn stripe_and_planar_output_size_errors_leave_caller_memory_unchanged() {
        let b = record();
        let r = VgaSpecRecord::parse(&b).unwrap();
        for index in [4, usize::MAX] {
            let mut out = vec![99; STRIPE_SIZE];
            assert_eq!(
                r.decode_stripe_into(index, &mut out),
                Err(VgaSpecError::StripeIndex { index })
            );
            assert!(out.iter().all(|&v| v == 99));
        }
        for actual in [0, STRIPE_SIZE - 1, STRIPE_SIZE + 1] {
            let mut out = vec![99; actual];
            assert_eq!(
                r.decode_stripe_into(0, &mut out),
                Err(VgaSpecError::OutputSize {
                    expected: STRIPE_SIZE,
                    actual
                })
            );
            assert!(out.iter().all(|&v| v == 99));
        }
        for actual in [0, VGASPEC_PLANAR_SIZE - 1, VGASPEC_PLANAR_SIZE + 1] {
            let mut out = vec![99; actual];
            assert_eq!(
                r.decode_into(&mut out),
                Err(VgaSpecError::OutputSize {
                    expected: VGASPEC_PLANAR_SIZE,
                    actual
                })
            );
            assert!(out.iter().all(|&v| v == 99));
        }
    }

    #[test]
    fn pixels_check_large_coordinates_and_output_lengths_without_modifying_on_error() {
        let b = record();
        let mut planes = vec![0; VGASPEC_PLANAR_SIZE];
        let image = VgaSpecRecord::parse(&b)
            .unwrap()
            .decode_into(&mut planes)
            .unwrap();
        for (x, y) in [(960, 0), (0, 160), (usize::MAX, 0), (0, usize::MAX)] {
            assert_eq!(image.pixel(x, y), None);
        }
        for actual in [0, 153599, 153601] {
            let mut out = vec![
                IndexedPixel {
                    color: 99,
                    opaque: true
                };
                actual
            ];
            assert_eq!(
                image.decode_into(&mut out),
                Err(VgaSpecError::OutputSize {
                    expected: 153600,
                    actual
                })
            );
            assert!(out.iter().all(|p| p.color == 99 && p.opaque));
        }
    }

    #[test]
    fn outer_dat_requires_one_complete_section_and_validates_the_entire_tail() {
        for actual in [0, 2, 3] {
            let b = packed(&record()).repeat(actual);
            assert_eq!(
                VgaSpecFile::parse(&b),
                Err(VgaSpecError::SectionCount { actual })
            );
        }
        let b = packed(&record());
        assert_eq!(
            VgaSpecFile::parse(&b).unwrap().decoded_size(),
            record().len()
        );
        let mut bad = b.clone();
        bad.push(0);
        assert!(matches!(
            VgaSpecFile::parse(&bad),
            Err(VgaSpecError::Dat {
                section: 1,
                error: DatError::TruncatedHeader { .. }
            })
        ));
        let mut bad = b.clone();
        bad[1] ^= 1;
        let error = VgaSpecFile::parse(&bad).unwrap_err();
        assert!(matches!(
            error,
            VgaSpecError::Dat {
                section: 0,
                error: DatError::ChecksumMismatch { .. }
            }
        ));
        assert!(core::error::Error::source(&error).is_some());
    }

    #[test]
    fn outer_decompression_uses_exact_caller_memory_and_returns_inner_validation_errors() {
        let data = record();
        let b = packed(&data);
        let file = VgaSpecFile::parse(&b).unwrap();
        let mut out = vec![99; data.len() - 1];
        assert!(matches!(
            file.decode_into(&mut out),
            Err(VgaSpecError::Decompress(DecompressError::OutputSize { .. }))
        ));
        assert!(out.iter().all(|&v| v == 99));
        let mut out = vec![0; data.len()];
        let r = file.decode_into(&mut out).unwrap();
        assert_eq!(r.as_bytes(), data);
        assert_eq!(r.as_bytes().as_ptr(), out.as_ptr());
        let bad = packed(&[0; 39]);
        let file = VgaSpecFile::parse(&bad).unwrap();
        assert_eq!(
            file.decode_into(&mut [0; 39]),
            Err(VgaSpecError::HeaderSize { actual: 39 })
        );
        let bad = [8, 255, 0, 0, 0, 1, 0, 0, 0, 11, 255];
        let file = VgaSpecFile::parse(&bad).unwrap();
        let error = file.decode_into(&mut [0; 1]).unwrap_err();
        assert!(matches!(error, VgaSpecError::Decompress(_)));
        assert!(core::error::Error::source(&error).is_some());
    }

    #[test]
    fn planar_image_lifetime_depends_only_on_its_output_buffer() {
        let mut out = vec![0; VGASPEC_PLANAR_SIZE];
        let image = {
            let b = record();
            VgaSpecRecord::parse(&b)
                .unwrap()
                .decode_into(&mut out)
                .unwrap()
        };
        assert_eq!(
            image.pixel(959, 159),
            Some(IndexedPixel {
                color: 0,
                opaque: false
            })
        );
    }
}
