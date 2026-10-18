use crate::{
    GroundPalette, IndexedPixel, LevelRecord, PlanarImage, Rgb6, TerrainPlacement, VgaSpecError,
    VgaSpecImage, VgaSpecRecord, VgagrError, VgagrImages,
};
use core::fmt;

/// DOS/Lemmix world width, fixed by the compatibility contract (not TS's 1600).
pub const LEVEL_TERRAIN_WIDTH: usize = 1584;
pub const LEVEL_TERRAIN_HEIGHT: usize = 160;
pub const LEVEL_TERRAIN_PIXEL_COUNT: usize = LEVEL_TERRAIN_WIDTH * LEVEL_TERRAIN_HEIGHT;
const SPECIAL_X: usize = 304;

/// Caller-tagged GROUND/VGAGR set. The tag binds a level selector; file provenance
/// and matching GROUND/VGAGR bytes are the caller's responsibility.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TerrainGraphics<'a> {
    index: u8,
    images: VgagrImages<'a>,
}
impl<'a> TerrainGraphics<'a> {
    pub fn new(index: u8, images: VgagrImages<'a>) -> Result<Self, TerrainComposeError> {
        if index > 4 {
            return Err(TerrainComposeError::GraphicsIndex { found: index });
        }
        Ok(Self { index, images })
    }
    pub const fn index(self) -> u8 {
        self.index
    }
    fn image(self, placement: TerrainPlacement) -> Result<PlanarImage<'a>, TerrainComposeError> {
        self.images
            .terrain(usize::from(placement.id))
            .map_err(|error| TerrainComposeError::Image {
                slot: placement.slot,
                error,
            })?
            .ok_or(TerrainComposeError::MissingTerrain {
                slot: placement.slot,
                id: placement.id,
            })
    }
}

/// Caller-tagged VGASPEC0..3 image and its copied six-bit palette. Construction
/// keeps the palette and pixels from the same validated record. Only the planar
/// output is borrowed; the compressed/inner source buffers may then be released.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpecialTerrain<'a> {
    index: u8,
    image: VgaSpecImage<'a>,
    palette: [Rgb6; 8],
}
impl<'a> SpecialTerrain<'a> {
    /// Invalid tag or planar-output length leaves output unchanged.
    pub fn decode_into(
        index: u8,
        record: VgaSpecRecord<'_>,
        output: &'a mut [u8],
    ) -> Result<Self, TerrainComposeError> {
        if index > 3 {
            return Err(TerrainComposeError::SpecialIndex { found: index });
        }
        let palette = record.vga_palette();
        let image = record
            .decode_into(output)
            .map_err(TerrainComposeError::Special)?;
        Ok(Self {
            index,
            image,
            palette,
        })
    }
    pub const fn index(self) -> u8 {
        self.index
    }
}

/// A special level uses only VGASPEC; normal terrain placements are not applied.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TerrainSource<'a> {
    Standard(TerrainGraphics<'a>),
    Special(SpecialTerrain<'a>),
}

/// Initial indexed terrain over caller memory, with a copied terrain palette.
/// Pixel opacity is independent of RGB/black; transparent pixels are canonical
/// color=0, opaque=false. This is not a physics/steel mask or an object layer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LevelTerrain<'a> {
    pixels: &'a [IndexedPixel],
    palette: [Rgb6; 8],
}
impl<'a> LevelTerrain<'a> {
    /// Rebind indexed terrain after gameplay edits, preserving this palette.
    /// Reject wrong canvas length, colors outside 0..7 or noncanonical empty cells.
    pub fn with_pixels(self, pixels: &'a [IndexedPixel]) -> Option<Self> {
        (pixels.len() == LEVEL_TERRAIN_PIXEL_COUNT
            && pixels
                .iter()
                .all(|p| p.color < 8 && (p.opaque || p.color == 0)))
        .then_some(Self {
            pixels,
            palette: self.palette,
        })
    }

    pub const fn pixels(self) -> &'a [IndexedPixel] {
        self.pixels
    }
    pub const fn palette(self) -> [Rgb6; 8] {
        self.palette
    }
    pub fn pixel(self, x: usize, y: usize) -> Option<IndexedPixel> {
        if x >= LEVEL_TERRAIN_WIDTH || y >= LEVEL_TERRAIN_HEIGHT {
            return None;
        }
        Some(self.pixels[y * LEVEL_TERRAIN_WIDTH + x])
    }
}

/// Compose a 1584x160 initial terrain canvas without allocation or I/O.
///
/// Normal slots draw in file order; masked pixels are ignored, vertical flip
/// precedes clipping, and no-overwrite takes precedence over erase (Lemmix).
/// Special images start at (304,0) and bypass normal placements entirely.
/// Output size, source selector, and every normal terrain ID (even offscreen)
/// are checked before any writes. Every error leaves output unchanged.
///
/// ```
/// use lemmings_resources::{compose_terrain, GroundRecord, IndexedPixel, LevelRecord,
///     TerrainGraphics, TerrainSource, VgagrImages, LEVEL_TERRAIN_PIXEL_COUNT};
/// let ground = [0; 1056];
/// let images = VgagrImages::new(GroundRecord::parse(&ground)?, &[], &[])?;
/// let graphics = TerrainGraphics::new(0, images)?;
/// let mut bytes = [0; 2048];
/// for offset in [1, 3, 5, 7] { bytes[offset] = 1; }
/// bytes[288..1888].fill(255); // no terrain placements
/// let level = LevelRecord::parse(&bytes)?;
/// let mut pixels = vec![IndexedPixel::default(); LEVEL_TERRAIN_PIXEL_COUNT];
/// let map = compose_terrain(level, TerrainSource::Standard(graphics), &mut pixels)?;
/// assert!(!map.pixel(0, 0).unwrap().opaque);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn compose_terrain<'a>(
    level: LevelRecord<'_>,
    source: TerrainSource<'_>,
    output: &'a mut [IndexedPixel],
) -> Result<LevelTerrain<'a>, TerrainComposeError> {
    if output.len() != LEVEL_TERRAIN_PIXEL_COUNT {
        return Err(TerrainComposeError::OutputSize {
            expected: LEVEL_TERRAIN_PIXEL_COUNT,
            actual: output.len(),
        });
    }
    let expected_special = level.header().special_graphic_set();
    let palette = match (source, expected_special) {
        (TerrainSource::Standard(_), Some(expected)) => {
            return Err(TerrainComposeError::MissingSpecial { expected });
        }
        (TerrainSource::Special(special), None) => {
            return Err(TerrainComposeError::UnexpectedSpecial {
                actual: special.index,
            });
        }
        (TerrainSource::Special(special), Some(expected)) => {
            if special.index != expected {
                return Err(TerrainComposeError::WrongSpecial {
                    expected,
                    actual: special.index,
                });
            }
            special.palette
        }
        (TerrainSource::Standard(graphics), None) => {
            let expected = level.header().graphics as u8;
            if graphics.index != expected {
                return Err(TerrainComposeError::WrongGraphics {
                    expected,
                    actual: graphics.index,
                });
            }
            for placement in level.terrains() {
                graphics.image(placement)?;
            }
            graphics
                .images
                .ground_record()
                .vga_palette(GroundPalette::Custom)
        }
    };
    output.fill(IndexedPixel::default());
    match source {
        TerrainSource::Standard(graphics) => {
            for placement in level.terrains() {
                let image = graphics
                    .image(placement)
                    .expect("prevalidated terrain slot");
                draw_terrain(output, placement, image);
            }
        }
        TerrainSource::Special(special) => {
            for y in 0..crate::VGASPEC_HEIGHT {
                for x in 0..crate::VGASPEC_WIDTH {
                    let pixel = special
                        .image
                        .pixel(x, y)
                        .expect("fixed special coordinates");
                    if pixel.opaque {
                        output[y * LEVEL_TERRAIN_WIDTH + SPECIAL_X + x] = pixel;
                    }
                }
            }
        }
    }
    Ok(LevelTerrain {
        pixels: output,
        palette,
    })
}

fn draw_terrain(output: &mut [IndexedPixel], placement: TerrainPlacement, image: PlanarImage<'_>) {
    let x = i32::from(placement.x);
    let y = i32::from(placement.y);
    let left = x.max(0);
    let top = y.max(0);
    let right = (x + i32::from(image.width())).min(LEVEL_TERRAIN_WIDTH as i32);
    let bottom = (y + i32::from(image.height())).min(LEVEL_TERRAIN_HEIGHT as i32);
    for dy in top..bottom {
        let sy = (dy - y) as usize;
        let sy = if placement.flags.upside_down {
            usize::from(image.height()) - 1 - sy
        } else {
            sy
        };
        for dx in left..right {
            let pixel = image
                .pixel((dx - x) as usize, sy)
                .expect("clipped source coordinates");
            if !pixel.opaque {
                continue;
            }
            let dest = &mut output[dy as usize * LEVEL_TERRAIN_WIDTH + dx as usize];
            if placement.flags.no_overwrite {
                if !dest.opaque {
                    *dest = pixel;
                }
            } else if placement.flags.erase {
                *dest = IndexedPixel::default();
            } else {
                *dest = pixel;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TerrainComposeError {
    GraphicsIndex { found: u8 },
    SpecialIndex { found: u8 },
    WrongGraphics { expected: u8, actual: u8 },
    MissingSpecial { expected: u8 },
    UnexpectedSpecial { actual: u8 },
    WrongSpecial { expected: u8, actual: u8 },
    MissingTerrain { slot: usize, id: u8 },
    Image { slot: usize, error: VgagrError },
    Special(VgaSpecError),
    OutputSize { expected: usize, actual: usize },
}
impl fmt::Display for TerrainComposeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GraphicsIndex { found } => {
                write!(f, "terrain graphics index {found}; expected 0..=4")
            }
            Self::SpecialIndex { found } => {
                write!(f, "special terrain index {found}; expected 0..=3")
            }
            Self::WrongGraphics { expected, actual } => {
                write!(f, "level needs graphics set {expected}; supplied {actual}")
            }
            Self::MissingSpecial { expected } => write!(
                f,
                "level needs special background {expected}; supplied standard terrain"
            ),
            Self::UnexpectedSpecial { actual } => {
                write!(f, "normal level received special background {actual}")
            }
            Self::WrongSpecial { expected, actual } => write!(
                f,
                "level needs special background {expected}; supplied {actual}"
            ),
            Self::MissingTerrain { slot, id } => write!(
                f,
                "terrain placement {slot} references empty graphics slot {id}"
            ),
            Self::Image { slot, error } => write!(f, "terrain placement {slot}: {error}"),
            Self::Special(error) => write!(f, "special terrain: {error}"),
            Self::OutputSize { expected, actual } => write!(
                f,
                "terrain output length {actual}; expected {expected} pixels"
            ),
        }
    }
}
impl core::error::Error for TerrainComposeError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Image { error, .. } => Some(error),
            Self::Special(error) => Some(error),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use crate::{GroundRecord, VGASPEC_PLANAR_SIZE};
    use std::{vec, vec::Vec};
    fn level() -> [u8; 2048] {
        let mut b = [0; 2048];
        for (i, w) in [50u16, 10, 5, 2].into_iter().enumerate() {
            b[i * 2..i * 2 + 2].copy_from_slice(&w.to_be_bytes());
        }
        b[288..1888].fill(255);
        b[2016..].fill(b' ');
        b
    }
    fn put(b: &mut [u8], slot: usize, x: i16, y: i16, id: u8, flags: u8) {
        let x = (x + 16) as u16;
        let y = ((i32::from(y) + 4) & 511) as u16;
        let p = 288 + slot * 4;
        b[p] = ((flags & 7) << 5) | ((x >> 8) as u8);
        b[p + 1] = x as u8;
        b[p + 2] = (y >> 1) as u8;
        b[p + 3] = ((y as u8 & 1) << 7) | id;
    }
    fn ground() -> [u8; 1056] {
        let mut b = [0; 1056];
        for i in 0..3 {
            let p = 448 + i * 8;
            b[p] = 8;
            b[p + 1] = 2;
            b[p + 2..p + 4].copy_from_slice(&((i * 8) as u16).to_le_bytes());
            b[p + 4..p + 6].copy_from_slice(&((i * 8 + 6) as u16).to_le_bytes());
        }
        for i in 1..8 {
            b[984 + i * 3..987 + i * 3].copy_from_slice(&[i as u8, (i * 2) as u8, (i * 3) as u8]);
        }
        b
    }
    const IMAGES: [u8; 24] = [
        0x55, 0xaa, 0x33, 0xcc, 0x0f, 0xf0, 0xff, 0xff, 0, 0, 0xff, 0xff, 0, 0, 0x0f, 0xf0, 0xff,
        0xff, 0xff, 0xff, 0, 0, 0xff, 0xff,
    ];
    fn graphics(b: &[u8; 1056], index: u8) -> TerrainGraphics<'_> {
        TerrainGraphics::new(
            index,
            VgagrImages::new(GroundRecord::parse(b).unwrap(), &IMAGES, &[]).unwrap(),
        )
        .unwrap()
    }
    fn sentinel() -> IndexedPixel {
        IndexedPixel {
            color: 99,
            opaque: true,
        }
    }
    fn out() -> Vec<IndexedPixel> {
        vec![sentinel(); LEVEL_TERRAIN_PIXEL_COUNT]
    }
    fn special_record(color: u8) -> Vec<u8> {
        let mut b = vec![0; 40];
        b[0] = 17;
        b[3] = 63;
        for _ in 0..4 {
            for bit in 0..3 {
                let mut n = 4800;
                while n > 0 {
                    let count = n.min(128);
                    b.extend_from_slice(&[
                        (257 - count) as u8,
                        if color & (1 << bit) != 0 { 255 } else { 0 },
                    ]);
                    n -= count;
                }
            }
            b.push(128);
        }
        b
    }

    #[test]
    fn empty_terrain_clears_every_pixel_borrows_output_and_checks_large_coordinates() {
        let b = level();
        let g = ground();
        let mut pixels = out();
        let ptr = pixels.as_ptr();
        let map = compose_terrain(
            LevelRecord::parse(&b).unwrap(),
            TerrainSource::Standard(graphics(&g, 0)),
            &mut pixels,
        )
        .unwrap();
        assert_eq!(map.pixels().len(), 1584 * 160);
        assert_eq!(map.pixels().as_ptr(), ptr);
        assert!(map.pixels().iter().all(|p| *p == IndexedPixel::default()));
        assert_eq!(map.palette()[7].components(), [7, 14, 21]);
        for (x, y) in [(1584, 0), (0, 160), (usize::MAX, 0), (0, usize::MAX)] {
            assert_eq!(map.pixel(x, y), None);
        }
    }
    #[test]
    fn placements_keep_file_order_and_vertical_flip_reverses_source_rows() {
        let mut b = level();
        put(&mut b, 10, 2, 1, 0, 0);
        put(&mut b, 399, 16, 1, 0, 2);
        let g = ground();
        let mut pixels = out();
        let map = compose_terrain(
            LevelRecord::parse(&b).unwrap(),
            TerrainSource::Standard(graphics(&g, 0)),
            &mut pixels,
        )
        .unwrap();
        for x in 0..8 {
            assert_eq!(map.pixel(x + 2, 1).unwrap().color, x as u8);
            assert_eq!(map.pixel(x + 16, 1).unwrap().color, 7 - x as u8);
            assert_eq!(map.pixel(x + 16, 2).unwrap().color, x as u8);
        }
    }
    #[test]
    fn later_opaque_pixels_overwrite_while_source_mask_holes_preserve_the_destination() {
        let mut b = level();
        put(&mut b, 0, 0, 0, 2, 0);
        put(&mut b, 20, 0, 0, 1, 0);
        let g = ground();
        let mut pixels = out();
        let map = compose_terrain(
            LevelRecord::parse(&b).unwrap(),
            TerrainSource::Standard(graphics(&g, 0)),
            &mut pixels,
        )
        .unwrap();
        for x in 0..8 {
            assert_eq!(map.pixel(x, 0).unwrap().color, if x >= 4 { 2 } else { 3 });
        }
        put(&mut b, 20, 0, 0, 2, 0);
        put(&mut b, 0, 0, 0, 1, 0);
        let map = compose_terrain(
            LevelRecord::parse(&b).unwrap(),
            TerrainSource::Standard(graphics(&g, 0)),
            &mut pixels,
        )
        .unwrap();
        for x in 0..8 {
            assert_eq!(map.pixel(x, 0).unwrap().color, 3);
        }
    }
    #[test]
    fn all_eight_flag_combinations_give_no_overwrite_priority_over_erasure() {
        for flags in 0..8 {
            let mut b = level();
            put(&mut b, 0, 0, 0, 1, 0);
            put(&mut b, 1, 0, 0, 0, flags);
            let g = ground();
            let mut pixels = out();
            let map = compose_terrain(
                LevelRecord::parse(&b).unwrap(),
                TerrainSource::Standard(graphics(&g, 0)),
                &mut pixels,
            )
            .unwrap();
            for y in 0..2 {
                for x in 0..8 {
                    let occupied = if y == 0 { x >= 4 } else { x < 4 };
                    let source_row = if flags & 2 != 0 { 1 - y } else { y };
                    let color = if source_row == 0 {
                        x as u8
                    } else {
                        7 - x as u8
                    };
                    let expected = if flags & 4 != 0 && occupied {
                        IndexedPixel {
                            color: 2,
                            opaque: true,
                        }
                    } else if flags & 4 == 0 && flags & 1 != 0 {
                        IndexedPixel::default()
                    } else {
                        IndexedPixel {
                            color,
                            opaque: true,
                        }
                    };
                    assert_eq!(map.pixel(x, y), Some(expected), "flags={flags} x={x} y={y}");
                }
            }
        }
    }
    #[test]
    fn flipped_erasure_uses_the_flipped_opacity_mask_and_zero_color_can_stay_opaque() {
        let mut b = level();
        put(&mut b, 0, 0, 0, 2, 0);
        put(&mut b, 1, 0, 0, 1, 3);
        let g = ground();
        let mut pixels = out();
        let map = compose_terrain(
            LevelRecord::parse(&b).unwrap(),
            TerrainSource::Standard(graphics(&g, 0)),
            &mut pixels,
        )
        .unwrap();
        for x in 0..8 {
            assert_eq!(map.pixel(x, 0).unwrap().opaque, x >= 4);
            assert_eq!(map.pixel(x, 1).unwrap().opaque, x < 4);
        }
        put(&mut b, 0, 0, 0, 0, 0);
        put(&mut b, 1, 0, 0, 2, 4);
        let map = compose_terrain(
            LevelRecord::parse(&b).unwrap(),
            TerrainSource::Standard(graphics(&g, 0)),
            &mut pixels,
        )
        .unwrap();
        assert_eq!(
            map.pixel(0, 0),
            Some(IndexedPixel {
                color: 0,
                opaque: true
            })
        );
        assert_eq!(map.palette()[0].components(), [0; 3]);
    }
    #[test]
    fn clipping_preserves_source_alignment_at_all_four_edges_and_canaries() {
        let mut b = level();
        put(&mut b, 0, -3, -1, 0, 0);
        put(&mut b, 1, 1581, 159, 0, 0);
        let g = ground();
        let mut pixels = vec![sentinel(); LEVEL_TERRAIN_PIXEL_COUNT + 2];
        {
            let map = compose_terrain(
                LevelRecord::parse(&b).unwrap(),
                TerrainSource::Standard(graphics(&g, 0)),
                &mut pixels[1..LEVEL_TERRAIN_PIXEL_COUNT + 1],
            )
            .unwrap();
            for (x, color) in [4, 3, 2, 1, 0].into_iter().enumerate() {
                assert_eq!(
                    map.pixel(x, 0),
                    Some(IndexedPixel {
                        color,
                        opaque: true
                    })
                );
            }
            assert_eq!(map.pixel(5, 0), Some(IndexedPixel::default()));
            for x in 0..3 {
                assert_eq!(map.pixel(1581 + x, 159).unwrap().color, x as u8);
            }
        }
        assert_eq!(pixels[0], sentinel());
        assert_eq!(*pixels.last().unwrap(), sentinel());
    }
    #[test]
    fn fully_offscreen_valid_placements_are_clipped_without_wrapping() {
        let mut b = level();
        for (slot, (x, y)) in [(-16, 0), (1584, 0), (4079, 251), (0, -260), (0, 160)]
            .into_iter()
            .enumerate()
        {
            put(&mut b, slot, x, y, 0, 0);
        }
        let g = ground();
        let mut pixels = out();
        let map = compose_terrain(
            LevelRecord::parse(&b).unwrap(),
            TerrainSource::Standard(graphics(&g, 0)),
            &mut pixels,
        )
        .unwrap();
        assert!(map.pixels().iter().all(|p| !p.opaque));
    }
    #[test]
    fn missing_terrain_in_a_late_offscreen_slot_fails_before_any_output_changes() {
        let mut b = level();
        put(&mut b, 0, 0, 0, 0, 0);
        put(&mut b, 399, 4079, 251, 63, 0);
        let g = ground();
        let mut pixels = out();
        assert_eq!(
            compose_terrain(
                LevelRecord::parse(&b).unwrap(),
                TerrainSource::Standard(graphics(&g, 0)),
                &mut pixels
            ),
            Err(TerrainComposeError::MissingTerrain { slot: 399, id: 63 })
        );
        assert!(pixels.iter().all(|p| *p == sentinel()));
    }
    #[test]
    fn wrong_canvas_length_and_graphics_binding_preserve_output() {
        let b = level();
        let g = ground();
        for actual in [
            0,
            LEVEL_TERRAIN_PIXEL_COUNT - 1,
            LEVEL_TERRAIN_PIXEL_COUNT + 1,
        ] {
            let mut pixels = vec![sentinel(); actual];
            assert_eq!(
                compose_terrain(
                    LevelRecord::parse(&b).unwrap(),
                    TerrainSource::Standard(graphics(&g, 0)),
                    &mut pixels
                ),
                Err(TerrainComposeError::OutputSize {
                    expected: LEVEL_TERRAIN_PIXEL_COUNT,
                    actual
                })
            );
            assert!(pixels.iter().all(|p| *p == sentinel()));
        }
        let mut pixels = out();
        assert_eq!(
            compose_terrain(
                LevelRecord::parse(&b).unwrap(),
                TerrainSource::Standard(graphics(&g, 1)),
                &mut pixels
            ),
            Err(TerrainComposeError::WrongGraphics {
                expected: 0,
                actual: 1
            })
        );
        assert!(pixels.iter().all(|p| *p == sentinel()));
    }
    #[test]
    fn source_tags_and_special_output_errors_are_checked_before_writes() {
        let g = ground();
        let images = VgagrImages::new(GroundRecord::parse(&g).unwrap(), &IMAGES, &[]).unwrap();
        assert_eq!(
            TerrainGraphics::new(5, images),
            Err(TerrainComposeError::GraphicsIndex { found: 5 })
        );
        let b = special_record(1);
        let record = VgaSpecRecord::parse(&b).unwrap();
        let mut planes = vec![99; VGASPEC_PLANAR_SIZE];
        assert_eq!(
            SpecialTerrain::decode_into(4, record, &mut planes),
            Err(TerrainComposeError::SpecialIndex { found: 4 })
        );
        assert!(planes.iter().all(|&v| v == 99));
        let mut short = [99; 1];
        let error = SpecialTerrain::decode_into(0, record, &mut short).unwrap_err();
        assert!(matches!(
            error,
            TerrainComposeError::Special(VgaSpecError::OutputSize { .. })
        ));
        assert!(core::error::Error::source(&error).is_some());
        assert_eq!(short, [99]);
    }
    #[test]
    fn all_four_special_selectors_use_their_palette_at_x_304_and_skip_normal_terrain() {
        let special = special_record(1);
        let record = VgaSpecRecord::parse(&special).unwrap();
        for index in 0..4 {
            let mut b = level();
            b[28..30].copy_from_slice(&u16::from(index + 1).to_be_bytes());
            put(&mut b, 399, 0, 0, 63, 0);
            let mut planes = vec![0; VGASPEC_PLANAR_SIZE];
            let selected = SpecialTerrain::decode_into(index, record, &mut planes).unwrap();
            assert_eq!(selected.index(), index);
            let mut pixels = out();
            let map = compose_terrain(
                LevelRecord::parse(&b).unwrap(),
                TerrainSource::Special(selected),
                &mut pixels,
            )
            .unwrap();
            assert_eq!(map.palette()[1].components(), [63, 0, 0]);
            for (x, y, opaque) in [
                (303, 0, false),
                (304, 0, true),
                (1263, 159, true),
                (1264, 159, false),
                (1583, 0, false),
            ] {
                assert_eq!(map.pixel(x, y).unwrap().opaque, opaque);
            }
            assert_eq!(map.pixels().iter().filter(|p| p.opaque).count(), 960 * 160);
        }
    }
    #[test]
    fn transparent_special_pixels_leave_a_canonical_empty_canvas_even_with_colored_index_zero() {
        let special = special_record(0);
        let record = VgaSpecRecord::parse(&special).unwrap();
        let mut b = level();
        b[29] = 1;
        put(&mut b, 0, 0, 0, 2, 0);
        let mut planes = vec![0; VGASPEC_PLANAR_SIZE];
        let source = SpecialTerrain::decode_into(0, record, &mut planes).unwrap();
        let mut pixels = out();
        let map = compose_terrain(
            LevelRecord::parse(&b).unwrap(),
            TerrainSource::Special(source),
            &mut pixels,
        )
        .unwrap();
        assert!(map.pixels().iter().all(|p| *p == IndexedPixel::default()));
        assert_eq!(map.palette()[0].components(), [17, 0, 0]);
    }
    #[test]
    fn missing_unexpected_or_wrong_special_sources_preserve_the_canvas() {
        let mut b = level();
        let special = special_record(1);
        let mut planes = vec![0; VGASPEC_PLANAR_SIZE];
        let source =
            SpecialTerrain::decode_into(0, VgaSpecRecord::parse(&special).unwrap(), &mut planes)
                .unwrap();
        let mut pixels = out();
        assert_eq!(
            compose_terrain(
                LevelRecord::parse(&b).unwrap(),
                TerrainSource::Special(source),
                &mut pixels
            ),
            Err(TerrainComposeError::UnexpectedSpecial { actual: 0 })
        );
        b[29] = 2;
        let g = ground();
        assert_eq!(
            compose_terrain(
                LevelRecord::parse(&b).unwrap(),
                TerrainSource::Standard(graphics(&g, 0)),
                &mut pixels
            ),
            Err(TerrainComposeError::MissingSpecial { expected: 1 })
        );
        assert_eq!(
            compose_terrain(
                LevelRecord::parse(&b).unwrap(),
                TerrainSource::Special(source),
                &mut pixels
            ),
            Err(TerrainComposeError::WrongSpecial {
                expected: 1,
                actual: 0
            })
        );
        assert!(pixels.iter().all(|p| *p == sentinel()));
    }
    #[test]
    fn composed_image_retains_only_output_and_copied_palette_after_sources_are_dropped() {
        let mut pixels = out();
        let map = {
            let b = level();
            let g = ground();
            compose_terrain(
                LevelRecord::parse(&b).unwrap(),
                TerrainSource::Standard(graphics(&g, 0)),
                &mut pixels,
            )
            .unwrap()
        };
        assert_eq!(map.palette()[7].components(), [7, 14, 21]);
        assert!(!map.pixel(0, 0).unwrap().opaque);
    }
}

#[cfg(test)]
mod edited_canvas_tests {
    use super::*;
    #[test]
    fn edited_canvas_retains_palette_and_rejects_bad_length_colors_or_transparency() {
        extern crate std;
        let palette = [Rgb6::from_components([1, 2, 3]).unwrap(); 8];
        let mut pixels = std::vec![IndexedPixel::default(); LEVEL_TERRAIN_PIXEL_COUNT];
        let original = LevelTerrain {
            pixels: &pixels,
            palette,
        };
        assert_eq!(original.with_pixels(&pixels).unwrap().palette(), palette);
        assert!(original.with_pixels(&pixels[..1]).is_none());
        let source = pixels.clone();
        let original = LevelTerrain {
            pixels: &source,
            palette,
        };
        pixels[0] = IndexedPixel {
            color: 8,
            opaque: true,
        };
        assert!(original.with_pixels(&pixels).is_none());
        pixels[0] = IndexedPixel {
            color: 1,
            opaque: false,
        };
        assert!(original.with_pixels(&pixels).is_none());
        pixels[0] = IndexedPixel {
            color: 7,
            opaque: true,
        };
        assert_eq!(
            original.with_pixels(&pixels).unwrap().pixel(0, 0),
            Some(pixels[0])
        );
    }
}
