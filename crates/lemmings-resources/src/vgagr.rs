use crate::{
    DatError, DatSection, DatSections, DecompressError, GROUND_OBJECT_COUNT, GROUND_TERRAIN_COUNT,
    GroundRecord,
};
use core::fmt;

/// Identifies the decompressed section: terrain is section 0, objects section 1.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImageKind {
    Terrain,
    Object,
}

/// Local palette index plus the independent mask bit. An opaque zero is valid.
/// Terrain colors address Custom 0..7; object colors address Standard/Preview 0..15.
/// VGASPEC uses its own eight-color palette, with index zero transparent.
/// MAIN lemming sprites use caller-selected colors 0..15, with zero transparent.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct IndexedPixel {
    pub color: u8,
    pub opaque: bool,
}

/// Validated borrowed color planes and a one-bit opacity mask, with no RGB policy.
/// Widths must be positive multiples of eight; rows are top to bottom, MSB left.
/// Plane 0 contributes color bit 0. Mask bit 1 means opaque, even for color zero.
///
/// ```
/// use lemmings_resources::{GroundRecord, IndexedPixel, VgagrImages};
/// let mut ground = [0; 1056];
/// ground[448] = 8; ground[449] = 1; // one 8x1 terrain image
/// ground[452] = 3; // absolute mask offset after three one-byte color planes
/// let bytes = [0x80, 0, 0, 0x80];
/// let images = VgagrImages::new(GroundRecord::parse(&ground)?, &bytes, &[])?;
/// let image = images.terrain(0)?.unwrap();
/// let mut pixels = [IndexedPixel::default(); 8];
/// image.decode_into(&mut pixels)?;
/// assert_eq!(pixels[0], IndexedPixel { color: 1, opaque: true });
/// assert!(!pixels[1].opaque);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlanarImage<'a> {
    colors: &'a [u8],
    mask: &'a [u8],
    width: u8,
    height: u8,
    planes: u8,
}
impl PlanarImage<'_> {
    pub const fn width(self) -> u8 {
        self.width
    }
    pub const fn height(self) -> u8 {
        self.height
    }
    pub fn pixel_count(self) -> usize {
        usize::from(self.width) * usize::from(self.height)
    }
    /// Bounds are checked before multiplying caller-provided coordinates.
    pub fn pixel(self, x: usize, y: usize) -> Option<IndexedPixel> {
        if x >= usize::from(self.width) || y >= usize::from(self.height) {
            return None;
        }
        Some(self.pixel_at(y * usize::from(self.width) + x))
    }
    /// Writes row-major pixels to an exact-sized caller buffer. A size error
    /// leaves every output pixel unchanged; a validated image has no other error.
    pub fn decode_into(self, output: &mut [IndexedPixel]) -> Result<(), VgagrError> {
        if output.len() != self.pixel_count() {
            return Err(VgagrError::OutputSize {
                expected: self.pixel_count(),
                actual: output.len(),
            });
        }
        for (index, pixel) in output.iter_mut().enumerate() {
            *pixel = self.pixel_at(index);
        }
        Ok(())
    }
    fn pixel_at(self, index: usize) -> IndexedPixel {
        let byte = index / 8;
        let bit = 0x80 >> (index % 8);
        let plane_size = self.mask.len();
        let mut color = 0;
        for plane in 0..self.planes {
            if self.colors[usize::from(plane) * plane_size + byte] & bit != 0 {
                color |= 1 << plane;
            }
        }
        IndexedPixel {
            color,
            opaque: self.mask[byte] & bit != 0,
        }
    }
}

/// Every disk animation frame, without rotation by the initial display frame.
/// Construction validates all frames and the separate preview image window.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObjectFrames<'a> {
    bytes: &'a [u8],
    meta: crate::GroundObject,
}
impl<'a> ObjectFrames<'a> {
    pub const fn frame_count(self) -> u8 {
        self.meta.frame_count
    }
    pub const fn first_frame(self) -> u8 {
        self.meta.first_frame
    }
    pub fn frame(self, index: usize) -> Result<PlanarImage<'a>, VgagrError> {
        if index >= usize::from(self.frame_count()) {
            return Err(VgagrError::FrameIndex {
                slot: self.meta.slot,
                index,
                count: self.frame_count(),
            });
        }
        // Checked index < u8 frame count and u16 stride/address fit native/WASM usize.
        let offset =
            usize::from(self.meta.image_offset) + index * usize::from(self.meta.frame_size);
        self.at(offset, Some(index))
    }
    /// Preview is an absolute image address. It need not be an animation-frame
    /// boundary; its entire color and mask windows were independently checked.
    pub fn preview(self) -> PlanarImage<'a> {
        self.at(usize::from(self.meta.preview_offset), None)
            .expect("validated preview")
    }
    fn at(self, offset: usize, frame: Option<usize>) -> Result<PlanarImage<'a>, VgagrError> {
        image(
            self.bytes,
            ImageContext {
                kind: ImageKind::Object,
                slot: self.meta.slot,
                frame,
            },
            self.meta.width,
            self.meta.height,
            4,
            offset,
            offset + usize::from(self.meta.mask_offset),
        )
    }
}

/// Binds a GROUND record to two already decompressed VGAGR sections.
/// Validates every nonzero slot, all object frames and previews. All-zero slots
/// remain `None`; nonzero malformed slots fail even if a caller never uses them.
/// The caller must select matching files; this does not authenticate a graphics set.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VgagrImages<'a> {
    ground: GroundRecord<'a>,
    terrain: &'a [u8],
    objects: &'a [u8],
}
impl<'a> VgagrImages<'a> {
    pub(crate) const fn ground_record(self) -> GroundRecord<'a> {
        self.ground
    }
    pub fn new(
        ground: GroundRecord<'a>,
        terrain: &'a [u8],
        objects: &'a [u8],
    ) -> Result<Self, VgagrError> {
        let images = Self {
            ground,
            terrain,
            objects,
        };
        for slot in 0..GROUND_TERRAIN_COUNT {
            images.terrain(slot)?;
        }
        for slot in 0..GROUND_OBJECT_COUNT {
            images.object(slot)?;
        }
        Ok(images)
    }
    pub fn terrain(self, slot: usize) -> Result<Option<PlanarImage<'a>>, VgagrError> {
        let meta = self.ground.terrain(slot).ok_or(VgagrError::Slot {
            kind: ImageKind::Terrain,
            index: slot,
        })?;
        if meta.is_empty() {
            return Ok(None);
        }
        let context = ImageContext {
            kind: ImageKind::Terrain,
            slot,
            frame: None,
        };
        let plane = plane_size(context, meta.width, meta.height)?;
        let minimum = usize::from(meta.image_offset) + plane * 3;
        if usize::from(meta.mask_offset) < minimum {
            return Err(VgagrError::MaskOffset {
                kind: ImageKind::Terrain,
                slot,
                minimum,
                actual: usize::from(meta.mask_offset),
            });
        }
        image(
            self.terrain,
            context,
            meta.width,
            meta.height,
            3,
            usize::from(meta.image_offset),
            usize::from(meta.mask_offset),
        )
        .map(Some)
    }
    pub fn object(self, slot: usize) -> Result<Option<ObjectFrames<'a>>, VgagrError> {
        let meta = self.ground.object(slot).ok_or(VgagrError::Slot {
            kind: ImageKind::Object,
            index: slot,
        })?;
        if meta.is_empty() {
            return Ok(None);
        }
        let plane = plane_size(
            ImageContext {
                kind: ImageKind::Object,
                slot,
                frame: None,
            },
            meta.width,
            meta.height,
        )?;
        if meta.frame_count == 0 || meta.first_frame >= meta.frame_count {
            return Err(VgagrError::FrameCount {
                slot,
                first: meta.first_frame,
                count: meta.frame_count,
            });
        }
        let minimum = plane * 4;
        if usize::from(meta.mask_offset) < minimum {
            return Err(VgagrError::MaskOffset {
                kind: ImageKind::Object,
                slot,
                minimum,
                actual: usize::from(meta.mask_offset),
            });
        }
        let minimum = usize::from(meta.mask_offset) + plane;
        if usize::from(meta.frame_size) < minimum {
            return Err(VgagrError::FrameSize {
                slot,
                minimum,
                actual: meta.frame_size,
            });
        }
        let frames = ObjectFrames {
            bytes: self.objects,
            meta,
        };
        for index in 0..usize::from(meta.frame_count) {
            frames.frame(index)?;
        }
        frames.at(usize::from(meta.preview_offset), None)?;
        Ok(Some(frames))
    }
}

/// Exactly two framed/XOR-checked DAT sections, terrain then objects.
/// Parsing checks the whole input, including extra sections or a damaged tail;
/// compressed commands and GROUND metadata are checked by `decode_into`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VgagrFile<'a> {
    terrain: DatSection<'a>,
    objects: DatSection<'a>,
}
impl<'a> VgagrFile<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, VgagrError> {
        let mut sections = [None; 2];
        let mut count = 0;
        for (index, section) in DatSections::new(bytes).enumerate() {
            let section = section.map_err(|error| VgagrError::Dat {
                section: index,
                error,
            })?;
            if index < 2 {
                sections[index] = Some(section);
            }
            count += 1;
        }
        if count != 2 {
            return Err(VgagrError::SectionCount { actual: count });
        }
        Ok(Self {
            terrain: sections[0].unwrap(),
            objects: sections[1].unwrap(),
        })
    }
    pub fn decoded_sizes(self) -> [usize; 2] {
        [
            usize::from(self.terrain.header().decompressed_size),
            usize::from(self.objects.header().decompressed_size),
        ]
    }
    /// Both output lengths are checked before any writes. Later decompression
    /// or metadata errors may leave output changed; discard both buffers on error.
    /// The returned views borrow only GROUND and the caller's output buffers.
    pub fn decode_into<'b>(
        self,
        ground: GroundRecord<'b>,
        terrain: &'b mut [u8],
        objects: &'b mut [u8],
    ) -> Result<VgagrImages<'b>, VgagrError> {
        for (section, (expected, actual)) in self
            .decoded_sizes()
            .into_iter()
            .zip([terrain.len(), objects.len()])
            .enumerate()
        {
            if expected != actual {
                return Err(VgagrError::BufferSize {
                    section,
                    expected,
                    actual,
                });
            }
        }
        self.terrain
            .decompress_into(terrain)
            .map_err(|error| VgagrError::Decompress { section: 0, error })?;
        self.objects
            .decompress_into(objects)
            .map_err(|error| VgagrError::Decompress { section: 1, error })?;
        VgagrImages::new(ground, terrain, objects)
    }
}

/// Ranges refer to a decompressed section, not the compressed file. `frame` is
/// `Some(index)` for object animation frames and `None` for terrain or preview.
/// MaskOffset uses an absolute address for terrain, a relative address for objects.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VgagrError {
    Dat {
        section: usize,
        error: DatError,
    },
    SectionCount {
        actual: usize,
    },
    BufferSize {
        section: usize,
        expected: usize,
        actual: usize,
    },
    Decompress {
        section: usize,
        error: DecompressError,
    },
    Slot {
        kind: ImageKind,
        index: usize,
    },
    Dimensions {
        kind: ImageKind,
        slot: usize,
        width: u8,
        height: u8,
    },
    FrameCount {
        slot: usize,
        first: u8,
        count: u8,
    },
    MaskOffset {
        kind: ImageKind,
        slot: usize,
        minimum: usize,
        actual: usize,
    },
    FrameSize {
        slot: usize,
        minimum: usize,
        actual: u16,
    },
    ImageRange {
        kind: ImageKind,
        slot: usize,
        frame: Option<usize>,
        offset: usize,
        length: usize,
        available: usize,
    },
    FrameIndex {
        slot: usize,
        index: usize,
        count: u8,
    },
    OutputSize {
        expected: usize,
        actual: usize,
    },
}
impl fmt::Display for VgagrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dat { section, error } => write!(f, "VGAGR section {section}: {error}"),
            Self::SectionCount { actual } => write!(f, "VGAGR has {actual} sections; expected 2"),
            Self::BufferSize {
                section,
                expected,
                actual,
            } => write!(
                f,
                "VGAGR section {section} output length {actual}; expected {expected}"
            ),
            Self::Decompress { section, error } => write!(f, "VGAGR section {section}: {error}"),
            Self::Slot { kind, index } => write!(f, "VGAGR {kind:?} slot {index} is out of range"),
            Self::Dimensions {
                kind,
                slot,
                width,
                height,
            } => write!(
                f,
                "VGAGR {kind:?} slot {slot} dimensions {width}x{height}; expected positive height and width divisible by 8"
            ),
            Self::FrameCount { slot, first, count } => write!(
                f,
                "VGAGR Object slot {slot} first frame {first} outside {count} frames"
            ),
            Self::MaskOffset {
                kind,
                slot,
                minimum,
                actual,
            } => write!(
                f,
                "VGAGR {kind:?} slot {slot} mask offset {actual} precedes color end {minimum}"
            ),
            Self::FrameSize {
                slot,
                minimum,
                actual,
            } => write!(
                f,
                "VGAGR Object slot {slot} frame size {actual}; minimum {minimum}"
            ),
            Self::ImageRange {
                kind,
                slot,
                frame,
                offset,
                length,
                available,
            } => write!(
                f,
                "VGAGR {kind:?} slot {slot} frame {frame:?}: range {offset}+{length} exceeds section length {available}"
            ),
            Self::FrameIndex { slot, index, count } => write!(
                f,
                "VGAGR Object slot {slot} frame {index} outside {count} frames"
            ),
            Self::OutputSize { expected, actual } => {
                write!(f, "VGAGR pixel output length {actual}; expected {expected}")
            }
        }
    }
}
impl core::error::Error for VgagrError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Dat { error, .. } => Some(error),
            Self::Decompress { error, .. } => Some(error),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
struct ImageContext {
    kind: ImageKind,
    slot: usize,
    frame: Option<usize>,
}
fn plane_size(context: ImageContext, width: u8, height: u8) -> Result<usize, VgagrError> {
    if width == 0 || !width.is_multiple_of(8) || height == 0 {
        return Err(VgagrError::Dimensions {
            kind: context.kind,
            slot: context.slot,
            width,
            height,
        });
    }
    Ok(usize::from(width) / 8 * usize::from(height))
}
fn window(
    bytes: &[u8],
    context: ImageContext,
    offset: usize,
    length: usize,
) -> Result<&[u8], VgagrError> {
    // Subtraction avoids overflowing an end offset on a malformed address.
    if offset > bytes.len() || length > bytes.len() - offset {
        return Err(VgagrError::ImageRange {
            kind: context.kind,
            slot: context.slot,
            frame: context.frame,
            offset,
            length,
            available: bytes.len(),
        });
    }
    Ok(&bytes[offset..offset + length])
}
fn image(
    bytes: &[u8],
    context: ImageContext,
    width: u8,
    height: u8,
    planes: u8,
    color_offset: usize,
    mask_offset: usize,
) -> Result<PlanarImage<'_>, VgagrError> {
    let plane = plane_size(context, width, height)?;
    let colors = window(bytes, context, color_offset, plane * usize::from(planes))?;
    let mask = window(bytes, context, mask_offset, plane)?;
    Ok(PlanarImage {
        colors,
        mask,
        width,
        height,
        planes,
    })
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use std::vec;

    fn word(b: &mut [u8], offset: usize, value: u16) {
        b[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }
    fn terrain(b: &mut [u8], slot: usize, w: u8, h: u8, image: u16, mask: u16) {
        let start = 448 + slot * 8;
        b[start] = w;
        b[start + 1] = h;
        word(b, start + 2, image);
        word(b, start + 4, mask);
    }
    fn object(b: &mut [u8], slot: usize) {
        let start = 28 * slot;
        b[start + 3] = 2;
        b[start + 4] = 8;
        b[start + 5] = 1;
        word(b, start + 6, 7);
        word(b, start + 8, 5);
        word(b, start + 21, 3);
        word(b, start + 23, 10);
    }
    fn dat(payload: &[u8], size: u16) -> std::vec::Vec<u8> {
        let mut b = vec![8, payload.iter().fold(0, |a, b| a ^ b), 0, 0];
        b.extend_from_slice(&size.to_be_bytes());
        b.extend_from_slice(&[0, 0]);
        b.extend_from_slice(&((10 + payload.len()) as u16).to_be_bytes());
        b.extend_from_slice(payload);
        b
    }
    fn literal(data: &[u8]) -> std::vec::Vec<u8> {
        assert!((1..=8).contains(&data.len()));
        // Decode the reversed bytes using a short literal command (00 + 3 bits).
        let mut bits = std::vec::Vec::new();
        bits.extend_from_slice(&[0, 0]);
        for bit in (0..3).rev() {
            bits.push((((data.len() - 1) >> bit) & 1) as u8);
        }
        for &v in data.iter().rev() {
            for bit in (0..8).rev() {
                bits.push((v >> bit) & 1);
            }
        }
        let mut packed = vec![0u8; bits.len().div_ceil(8)];
        for (i, chunk) in bits.rchunks(8).enumerate() {
            for (j, &v) in chunk.iter().enumerate() {
                packed[i] |= v << j;
            }
        }
        let mut b = dat(&packed, data.len() as u16);
        b[0] = if bits.len().is_multiple_of(8) {
            8
        } else {
            (bits.len() % 8) as u8
        };
        b
    }

    #[test]
    fn terrain_planes_are_low_bit_first_with_msb_left_and_top_to_bottom_rows() {
        let mut b = [0; 1056];
        terrain(&mut b, 0, 8, 2, 2, 8);
        let src = [0x55, 0x66, 0x55, 0xaa, 0x33, 0xcc, 0x0f, 0xf0, 0x81, 0x7e];
        let images = VgagrImages::new(GroundRecord::parse(&b).unwrap(), &src, &[]).unwrap();
        let image = images.terrain(0).unwrap().unwrap();
        assert_eq!(
            (image.width(), image.height(), image.pixel_count()),
            (8, 2, 16)
        );
        let mut out = [IndexedPixel::default(); 16];
        image.decode_into(&mut out).unwrap();
        let colors = [0, 1, 2, 3, 4, 5, 6, 7, 7, 6, 5, 4, 3, 2, 1, 0];
        for (p, &color) in colors.iter().enumerate() {
            assert_eq!(
                out[p],
                IndexedPixel {
                    color,
                    opaque: if p < 8 {
                        p == 0 || p == 7
                    } else {
                        (1..=6).contains(&(p - 8))
                    }
                }
            );
            assert_eq!(image.pixel(p % 8, p / 8), Some(out[p]));
        }
        assert!(out[0].opaque);
        assert_ne!(out[3].color, 0);
        assert!(!out[3].opaque);
    }

    #[test]
    fn object_fourth_plane_and_padding_preserve_all_sixteen_colors() {
        let mut b = [0; 1056];
        object(&mut b, 0);
        b[3] = 1;
        b[4] = 16;
        word(&mut b, 6, 13);
        word(&mut b, 8, 10);
        word(&mut b, 23, 3);
        let src = [
            0, 0, 0, 0x55, 0x55, 0x33, 0x33, 0x0f, 0x0f, 0x00, 0xff, 99, 98, 0xff, 0xff, 77,
        ];
        let images = VgagrImages::new(GroundRecord::parse(&b).unwrap(), &[], &src).unwrap();
        let image = images.object(0).unwrap().unwrap().frame(0).unwrap();
        let mut out = [IndexedPixel::default(); 16];
        image.decode_into(&mut out).unwrap();
        for (i, p) in out.iter().enumerate() {
            assert_eq!(
                *p,
                IndexedPixel {
                    color: i as u8,
                    opaque: true
                }
            );
        }
    }

    #[test]
    fn animation_stride_first_frame_and_preview_do_not_reorder_disk_frames() {
        let mut b = [0; 1056];
        object(&mut b, 15);
        b[15 * 28 + 2] = 1;
        let mut src = [0; 16];
        src[3] = 0x80;
        src[8] = 0x80;
        src[10 + 3] = 0x80;
        src[10 + 5] = 0x80;
        let images = VgagrImages::new(GroundRecord::parse(&b).unwrap(), &[], &src).unwrap();
        let frames = images.object(15).unwrap().unwrap();
        assert_eq!((frames.frame_count(), frames.first_frame()), (2, 1));
        assert_eq!(
            frames.frame(0).unwrap().pixel(0, 0),
            Some(IndexedPixel {
                color: 1,
                opaque: true
            })
        );
        assert_eq!(
            frames.frame(1).unwrap().pixel(0, 0),
            Some(IndexedPixel {
                color: 8,
                opaque: true
            })
        );
        assert_eq!(frames.preview(), frames.frame(1).unwrap());
        assert!(images.object(14).unwrap().is_none());
    }

    #[test]
    fn empty_slots_and_bad_slot_or_pixel_indexes_are_distinct_and_safe() {
        let mut b = [0; 1056];
        terrain(&mut b, 63, 8, 1, 0, 3);
        let src = [0; 4];
        let images = VgagrImages::new(GroundRecord::parse(&b).unwrap(), &src, &[]).unwrap();
        assert!(images.terrain(62).unwrap().is_none());
        assert!(images.object(0).unwrap().is_none());
        for index in [64, usize::MAX] {
            assert_eq!(
                images.terrain(index),
                Err(VgagrError::Slot {
                    kind: ImageKind::Terrain,
                    index
                })
            );
        }
        for index in [16, usize::MAX] {
            assert_eq!(
                images.object(index),
                Err(VgagrError::Slot {
                    kind: ImageKind::Object,
                    index
                })
            );
        }
        let image = images.terrain(63).unwrap().unwrap();
        for (x, y) in [(8, 0), (0, 1), (usize::MAX, 0), (0, usize::MAX)] {
            assert_eq!(image.pixel(x, y), None);
        }
    }

    #[test]
    fn pixel_output_requires_exact_length_and_errors_preserve_the_buffer() {
        let mut b = [0; 1056];
        terrain(&mut b, 0, 8, 1, 0, 3);
        let src = [0xff; 4];
        let image = VgagrImages::new(GroundRecord::parse(&b).unwrap(), &src, &[])
            .unwrap()
            .terrain(0)
            .unwrap()
            .unwrap();
        for actual in [0, 7, 9] {
            let mut out = vec![
                IndexedPixel {
                    color: 99,
                    opaque: true
                };
                actual
            ];
            let before = out.clone();
            assert_eq!(
                image.decode_into(&mut out),
                Err(VgagrError::OutputSize {
                    expected: 8,
                    actual
                })
            );
            assert_eq!(out, before);
        }
    }

    #[test]
    fn all_nonempty_slots_require_positive_height_and_byte_aligned_width() {
        for (w, h) in [(0, 1), (7, 1), (9, 8), (8, 0)] {
            let mut b = [0; 1056];
            terrain(&mut b, 63, w, h, 0, 3);
            assert_eq!(
                VgagrImages::new(GroundRecord::parse(&b).unwrap(), &[0; 128], &[]),
                Err(VgagrError::Dimensions {
                    kind: ImageKind::Terrain,
                    slot: 63,
                    width: w,
                    height: h
                })
            );
            let mut b = [0; 1056];
            object(&mut b, 15);
            b[15 * 28 + 4] = w;
            b[15 * 28 + 5] = h;
            assert_eq!(
                VgagrImages::new(GroundRecord::parse(&b).unwrap(), &[], &[0; 128]),
                Err(VgagrError::Dimensions {
                    kind: ImageKind::Object,
                    slot: 15,
                    width: w,
                    height: h
                })
            );
        }
        let mut b = [0; 1056];
        word(&mut b, 448 + 63 * 8 + 6, 123);
        assert!(matches!(
            VgagrImages::new(GroundRecord::parse(&b).unwrap(), &[], &[]),
            Err(VgagrError::Dimensions { slot: 63, .. })
        ));
    }

    #[test]
    fn object_frame_count_and_first_frame_are_validated_before_reading_pixels() {
        for (first, count) in [(0, 0), (2, 2), (255, 1)] {
            let mut b = [0; 1056];
            object(&mut b, 0);
            b[2] = first;
            b[3] = count;
            assert_eq!(
                VgagrImages::new(GroundRecord::parse(&b).unwrap(), &[], &[0; 16]),
                Err(VgagrError::FrameCount {
                    slot: 0,
                    first,
                    count
                })
            );
        }
    }

    #[test]
    fn mask_overlap_and_short_frame_stride_are_rejected_but_padding_is_allowed() {
        let mut b = [0; 1056];
        terrain(&mut b, 0, 8, 1, 2, 4);
        assert_eq!(
            VgagrImages::new(GroundRecord::parse(&b).unwrap(), &[0; 8], &[]),
            Err(VgagrError::MaskOffset {
                kind: ImageKind::Terrain,
                slot: 0,
                minimum: 5,
                actual: 4
            })
        );
        let mut b = [0; 1056];
        object(&mut b, 0);
        word(&mut b, 8, 3);
        assert_eq!(
            VgagrImages::new(GroundRecord::parse(&b).unwrap(), &[], &[0; 16]),
            Err(VgagrError::MaskOffset {
                kind: ImageKind::Object,
                slot: 0,
                minimum: 4,
                actual: 3
            })
        );
        word(&mut b, 8, 5);
        word(&mut b, 6, 5);
        assert_eq!(
            VgagrImages::new(GroundRecord::parse(&b).unwrap(), &[], &[0; 16]),
            Err(VgagrError::FrameSize {
                slot: 0,
                minimum: 6,
                actual: 5
            })
        );
        word(&mut b, 6, 7);
        assert!(VgagrImages::new(GroundRecord::parse(&b).unwrap(), &[], &[0; 16]).is_ok());
    }

    #[test]
    fn truncated_color_and_mask_windows_report_slot_and_exact_range() {
        let mut b = [0; 1056];
        terrain(&mut b, 63, 8, 1, 2, 6);
        for (len, offset, length) in [(4, 2, 3), (6, 6, 1)] {
            assert_eq!(
                VgagrImages::new(GroundRecord::parse(&b).unwrap(), &vec![0; len], &[]),
                Err(VgagrError::ImageRange {
                    kind: ImageKind::Terrain,
                    slot: 63,
                    frame: None,
                    offset,
                    length,
                    available: len
                })
            );
        }
        assert!(VgagrImages::new(GroundRecord::parse(&b).unwrap(), &[0; 7], &[]).is_ok());
        let mut b = [0; 1056];
        object(&mut b, 15);
        assert_eq!(
            VgagrImages::new(GroundRecord::parse(&b).unwrap(), &[], &[0; 15]),
            Err(VgagrError::ImageRange {
                kind: ImageKind::Object,
                slot: 15,
                frame: Some(1),
                offset: 15,
                length: 1,
                available: 15
            })
        );
        assert!(VgagrImages::new(GroundRecord::parse(&b).unwrap(), &[], &[0; 16]).is_ok());
    }

    #[test]
    fn preview_is_independently_checked_and_frame_indexes_cannot_overflow() {
        let mut b = [0; 1056];
        object(&mut b, 0);
        word(&mut b, 23, 12);
        assert_eq!(
            VgagrImages::new(GroundRecord::parse(&b).unwrap(), &[], &[0; 17]),
            Err(VgagrError::ImageRange {
                kind: ImageKind::Object,
                slot: 0,
                frame: None,
                offset: 17,
                length: 1,
                available: 17
            })
        );
        let src = [0; 18];
        let frames = VgagrImages::new(GroundRecord::parse(&b).unwrap(), &[], &src)
            .unwrap()
            .object(0)
            .unwrap()
            .unwrap();
        for index in [2, usize::MAX] {
            assert_eq!(
                frames.frame(index),
                Err(VgagrError::FrameIndex {
                    slot: 0,
                    index,
                    count: 2
                })
            );
        }
        assert_eq!(frames.preview().pixel_count(), 8);
    }

    #[test]
    fn maximum_disk_values_return_bounds_errors_without_arithmetic_overflow() {
        let mut b = [0; 1056];
        object(&mut b, 15);
        let s = 15 * 28;
        b[s + 3] = 255;
        b[s + 4] = 248;
        b[s + 5] = 255;
        word(&mut b, s + 6, 65535);
        word(&mut b, s + 8, 40000);
        word(&mut b, s + 21, 65535);
        assert_eq!(
            VgagrImages::new(GroundRecord::parse(&b).unwrap(), &[], &[]),
            Err(VgagrError::ImageRange {
                kind: ImageKind::Object,
                slot: 15,
                frame: Some(0),
                offset: 65535,
                length: 31620,
                available: 0
            })
        );
    }

    #[test]
    fn vgagr_requires_exactly_two_valid_sections_and_checks_the_trailing_input() {
        for count in [0, 1, 3] {
            let b = literal(&[0]).repeat(count);
            assert_eq!(
                VgagrFile::parse(&b),
                Err(VgagrError::SectionCount { actual: count })
            );
        }
        let b = literal(&[0]).repeat(2);
        assert_eq!(VgagrFile::parse(&b).unwrap().decoded_sizes(), [1, 1]);
        let mut bad = b.clone();
        bad.push(0);
        assert!(matches!(
            VgagrFile::parse(&bad),
            Err(VgagrError::Dat {
                section: 2,
                error: DatError::TruncatedHeader { .. }
            })
        ));
        let mut bad = b.clone();
        bad[b.len() - 1] ^= 1;
        assert!(matches!(
            VgagrFile::parse(&bad),
            Err(VgagrError::Dat {
                section: 1,
                error: DatError::ChecksumMismatch { .. }
            })
        ));
    }

    #[test]
    fn decoding_validates_both_output_sizes_before_writing_either_buffer() {
        let b = literal(&[0x80, 0, 0, 0x80]);
        let mut dat_bytes = b.clone();
        dat_bytes.extend_from_slice(&literal(&[0]));
        let file = VgagrFile::parse(&dat_bytes).unwrap();
        let mut ground = [0; 1056];
        terrain(&mut ground, 0, 8, 1, 0, 3);
        let ground = GroundRecord::parse(&ground).unwrap();
        for (tlen, olen, section, expected, actual) in [(3, 1, 0, 4, 3), (4, 2, 1, 1, 2)] {
            let mut t = vec![99; tlen];
            let mut o = vec![88; olen];
            assert_eq!(
                file.decode_into(ground, &mut t, &mut o),
                Err(VgagrError::BufferSize {
                    section,
                    expected,
                    actual
                })
            );
            assert_eq!(t, vec![99; tlen]);
            assert_eq!(o, vec![88; olen]);
        }
        let mut t = [0; 4];
        let mut o = [0; 1];
        let images = file.decode_into(ground, &mut t, &mut o).unwrap();
        assert_eq!(
            images.terrain(0).unwrap().unwrap().pixel(0, 0),
            Some(IndexedPixel {
                color: 1,
                opaque: true
            })
        );
    }

    #[test]
    fn decompression_and_metadata_errors_keep_their_section_or_slot_context() {
        let mut b = literal(&[0]);
        b.extend_from_slice(&dat(&[0xff], 1));
        let file = VgagrFile::parse(&b).unwrap();
        let ground = [0; 1056];
        let mut t = [0; 1];
        let mut o = [0; 1];
        let error = file
            .decode_into(GroundRecord::parse(&ground).unwrap(), &mut t, &mut o)
            .unwrap_err();
        assert!(matches!(error, VgagrError::Decompress { section: 1, .. }));
        assert!(core::error::Error::source(&error).is_some());
        let mut ground = [0; 1056];
        terrain(&mut ground, 63, 8, 1, 0, 3);
        let b = literal(&[0]).repeat(2);
        let file = VgagrFile::parse(&b).unwrap();
        assert!(matches!(
            file.decode_into(GroundRecord::parse(&ground).unwrap(), &mut t, &mut o),
            Err(VgagrError::ImageRange { slot: 63, .. })
        ));
    }
}
