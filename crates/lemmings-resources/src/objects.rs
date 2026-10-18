use crate::{
    IndexedPixel, LEVEL_TERRAIN_HEIGHT, LEVEL_TERRAIN_PIXEL_COUNT, LEVEL_TERRAIN_WIDTH,
    LevelRecord, LevelTerrain, ObjectFrames, ObjectPlacement, PlanarImage, VgagrError, VgagrImages,
};
use core::fmt;

/// Caller-tagged GROUND/VGAGR set for objects, including special-background levels.
/// Matching file provenance and the terrain's level are the caller's responsibility.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObjectGraphics<'a> {
    index: u8,
    images: VgagrImages<'a>,
}
impl<'a> ObjectGraphics<'a> {
    pub fn new(index: u8, images: VgagrImages<'a>) -> Result<Self, ObjectComposeError> {
        if index > 4 {
            return Err(ObjectComposeError::GraphicsIndex { found: index });
        }
        Ok(Self { index, images })
    }
    pub const fn index(self) -> u8 {
        self.index
    }
    fn frames(self, p: ObjectPlacement) -> Result<ObjectFrames<'a>, ObjectComposeError> {
        self.images
            .object(usize::from(p.id))
            .map_err(|error| ObjectComposeError::Image {
                slot: p.slot,
                error,
            })?
            .ok_or(ObjectComposeError::MissingObject {
                slot: p.slot,
                id: p.id,
            })
    }
}

/// No tick scheduling or frame wrapping. Explicit indices use the original 32
/// level slots, including holes; unused slots are ignored. Preview reads the
/// validated absolute preview window, which need not be a frame boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObjectFrameSelection<'a> {
    Preview,
    Initial,
    Explicit(&'a [usize; 32]),
}
impl ObjectFrameSelection<'_> {
    fn image<'a>(
        self,
        p: ObjectPlacement,
        frames: ObjectFrames<'a>,
    ) -> Result<PlanarImage<'a>, ObjectComposeError> {
        match self {
            Self::Preview => Ok(frames.preview()),
            Self::Initial => frames.frame(usize::from(frames.first_frame())),
            Self::Explicit(indices) => frames.frame(indices[p.slot]),
        }
        .map_err(|error| ObjectComposeError::Image {
            slot: p.slot,
            error,
        })
    }
}

/// Visible global palette index and independent drawing provenance bits.
/// Terrain stays set when an object covers it; this is not a collision mask.
/// Transparent pixels are canonical color=0, terrain=false, object=false.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ScenePixel {
    pub color: u8,
    pub terrain: bool,
    pub object: bool,
}
impl ScenePixel {
    pub const fn opaque(self) -> bool {
        self.terrain || self.object
    }
}

/// Static 1584x160 composition over caller memory. The palette is RGB8 using
/// Lemmix's mixed conversions: fixed colors use raw6<<2, custom colors use
/// floor(raw6*255/63), and special terrain uses raw6<<2. Object and terrain
/// palettes remain separate because special levels give them different RGB.
/// The result borrows only output, not the level, images, selection, or terrain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LevelScene<'a> {
    pixels: &'a [ScenePixel],
    palette: [[u8; 3]; 16],
    terrain_palette: [[u8; 3]; 8],
}
impl<'a> LevelScene<'a> {
    pub const fn pixels(self) -> &'a [ScenePixel] {
        self.pixels
    }
    /// RGB8 object/game palette, including the brick color at index 7.
    pub const fn palette(self) -> [[u8; 3]; 16] {
        self.palette
    }
    /// RGB8 terrain colors in local index order 0..7.
    pub const fn terrain_palette(self) -> [[u8; 3]; 8] {
        self.terrain_palette
    }
    /// Canonical transparent RGBA is all zero. Object colors take precedence
    /// over terrain colors, while the independent terrain provenance is retained.
    pub fn rgba(self, x: usize, y: usize) -> Option<[u8; 4]> {
        let p = self.pixel(x, y)?;
        if !p.opaque() {
            return Some([0; 4]);
        }
        let rgb = if p.object {
            self.palette[usize::from(p.color)]
        } else {
            self.terrain_palette[usize::from(p.color - 8)]
        };
        Some([rgb[0], rgb[1], rgb[2], 255])
    }
    pub fn pixel(self, x: usize, y: usize) -> Option<ScenePixel> {
        if x >= LEVEL_TERRAIN_WIDTH || y >= LEVEL_TERRAIN_HEIGHT {
            return None;
        }
        Some(self.pixels[y * LEVEL_TERRAIN_WIDTH + x])
    }
}

/// Compose objects over an initial terrain map from the same level.
///
/// Only-on-terrain objects draw first, then other objects, in original file order
/// within each pass. Only-on-terrain takes precedence over no-overwrite. The
/// former tests the preserved terrain bit; the latter tests any previous drawing.
/// Vertical flip precedes clipping. Source opacity is independent of RGB/black.
/// Every active ID/frame (even offscreen), graphics tag, and output length is
/// checked before any writes; all errors leave output unchanged.
///
/// ```
/// use lemmings_resources::{compose_objects, compose_terrain, GroundRecord,
///     IndexedPixel, LevelRecord, ObjectFrameSelection, ObjectGraphics, ScenePixel,
///     TerrainGraphics, TerrainSource, VgagrImages, LEVEL_TERRAIN_PIXEL_COUNT};
/// let ground = [0; 1056];
/// let images = VgagrImages::new(GroundRecord::parse(&ground)?, &[], &[])?;
/// let mut bytes = [0; 2048];
/// for offset in [1, 3, 5, 7] { bytes[offset] = 1; }
/// bytes[288..1888].fill(255);
/// let level = LevelRecord::parse(&bytes)?;
/// let mut terrain = vec![IndexedPixel::default(); LEVEL_TERRAIN_PIXEL_COUNT];
/// let terrain = compose_terrain(level,
///     TerrainSource::Standard(TerrainGraphics::new(0, images)?), &mut terrain)?;
/// let mut output = vec![ScenePixel::default(); LEVEL_TERRAIN_PIXEL_COUNT];
/// let scene = compose_objects(level, ObjectGraphics::new(0, images)?, terrain,
///     ObjectFrameSelection::Preview, &mut output)?;
/// assert!(!scene.pixel(0, 0).unwrap().opaque());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn compose_objects<'a>(
    level: LevelRecord<'_>,
    graphics: ObjectGraphics<'_>,
    terrain: LevelTerrain<'_>,
    selection: ObjectFrameSelection<'_>,
    output: &'a mut [ScenePixel],
) -> Result<LevelScene<'a>, ObjectComposeError> {
    if output.len() != LEVEL_TERRAIN_PIXEL_COUNT {
        return Err(ObjectComposeError::OutputSize {
            expected: LEVEL_TERRAIN_PIXEL_COUNT,
            actual: output.len(),
        });
    }
    let expected = level.header().graphics as u8;
    if graphics.index != expected {
        return Err(ObjectComposeError::WrongGraphics {
            expected,
            actual: graphics.index,
        });
    }
    // Freeze the selected validated images before clearing caller memory.
    let mut images = [None; 32];
    for p in level.objects() {
        images[p.slot] = Some(selection.image(p, graphics.frames(p)?)?);
    }
    let mut palette = [[0; 3]; 16];
    // Pinned Dos.Structures.pas default palette; Dos.Bitmaps uses a left shift.
    const FIXED: [[u8; 3]; 8] = [
        [0, 0, 0],
        [16, 16, 56],
        [0, 44, 0],
        [60, 52, 52],
        [44, 44, 0],
        [60, 8, 8],
        [32, 32, 32],
        [0, 0, 0],
    ];
    for (dst, raw) in palette[..8].iter_mut().zip(FIXED) {
        *dst = raw.map(|c| c << 2);
    }
    for (dst, raw) in palette[8..].iter_mut().zip(terrain.palette()) {
        *dst = raw.to_rgb8_full_range();
    }
    if level.header().special_graphic_set().is_some() {
        palette[8] = [124, 124, 0];
    }
    palette[7] = palette[8];
    let terrain_palette = terrain.palette().map(|raw| {
        if level.header().special_graphic_set().is_some() {
            raw.to_rgb8_shifted()
        } else {
            raw.to_rgb8_full_range()
        }
    });
    for (dst, pixel) in output.iter_mut().zip(terrain.pixels()) {
        *dst = if pixel.opaque {
            ScenePixel {
                color: pixel.color + 8,
                terrain: true,
                object: false,
            }
        } else {
            ScenePixel::default()
        };
    }
    for only_on_terrain in [true, false] {
        for p in level
            .objects()
            .filter(|p| p.flags.only_on_terrain == only_on_terrain)
        {
            draw_object(output, p, images[p.slot].expect("prevalidated object slot"));
        }
    }
    Ok(LevelScene {
        pixels: output,
        palette,
        terrain_palette,
    })
}
fn draw_object(output: &mut [ScenePixel], p: ObjectPlacement, image: PlanarImage<'_>) {
    let (x, y) = (p.x, i32::from(p.y));
    let (left, top) = (x.max(0), y.max(0));
    let right = (x + i32::from(image.width())).min(LEVEL_TERRAIN_WIDTH as i32);
    let bottom = (y + i32::from(image.height())).min(LEVEL_TERRAIN_HEIGHT as i32);
    for dy in top..bottom {
        let sy = (dy - y) as usize;
        let sy = if p.flags.upside_down {
            usize::from(image.height()) - 1 - sy
        } else {
            sy
        };
        for dx in left..right {
            let IndexedPixel { color, opaque } = image
                .pixel((dx - x) as usize, sy)
                .expect("clipped source coordinates");
            if !opaque {
                continue;
            }
            let dest = &mut output[dy as usize * LEVEL_TERRAIN_WIDTH + dx as usize];
            if p.flags.only_on_terrain {
                if !dest.terrain {
                    continue;
                }
            } else if p.flags.no_overwrite && dest.opaque() {
                continue;
            }
            dest.color = color;
            dest.object = true;
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObjectComposeError {
    GraphicsIndex { found: u8 },
    WrongGraphics { expected: u8, actual: u8 },
    MissingObject { slot: usize, id: u8 },
    Image { slot: usize, error: VgagrError },
    OutputSize { expected: usize, actual: usize },
}
impl fmt::Display for ObjectComposeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GraphicsIndex { found } => {
                write!(f, "object graphics index {found}; expected 0..=4")
            }
            Self::WrongGraphics { expected, actual } => write!(
                f,
                "level needs object graphics set {expected}; supplied {actual}"
            ),
            Self::MissingObject { slot, id } => write!(
                f,
                "object placement {slot} references empty graphics slot {id}"
            ),
            Self::Image { slot, error } => write!(f, "object placement {slot}: {error}"),
            Self::OutputSize { expected, actual } => write!(
                f,
                "scene output length {actual}; expected {expected} pixels"
            ),
        }
    }
}
impl core::error::Error for ObjectComposeError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Image { error, .. } => Some(error),
            _ => None,
        }
    }
}
#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use crate::{
        GroundRecord, SpecialTerrain, TerrainGraphics, TerrainSource, VGASPEC_PLANAR_SIZE,
        VgaSpecRecord, compose_terrain,
    };
    use std::{vec, vec::Vec};
    fn level() -> [u8; 2048] {
        let mut b = [0; 2048];
        for p in [1, 3, 5, 7] {
            b[p] = 1;
        }
        b[288..1888].fill(255);
        b
    }
    fn put(b: &mut [u8], slot: usize, x: i32, y: u16, id: u8, flags: u8, flip: bool) {
        let p = 32 + slot * 8;
        b[p..p + 2].copy_from_slice(&((x + 16) as u16).to_be_bytes());
        b[p + 2..p + 4].copy_from_slice(&y.to_be_bytes());
        b[p + 5] = id;
        b[p + 6] = flags;
        b[p + 7] = if flip { 0x8f } else { 0 };
    }
    fn ground() -> [u8; 1056] {
        let mut b = [0; 1056];
        for slot in 0..3 {
            let p = slot * 28;
            b[p + 2] = 1;
            b[p + 3] = 2;
            b[p + 4] = 8;
            b[p + 5] = 2;
            b[p + 6..p + 8].copy_from_slice(&10u16.to_le_bytes());
            b[p + 8..p + 10].copy_from_slice(&8u16.to_le_bytes());
            b[p + 21..p + 23].copy_from_slice(&((slot * 20) as u16).to_le_bytes());
            b[p + 23..p + 25].copy_from_slice(&((slot * 20) as u16).to_le_bytes());
        }
        b[448] = 8;
        b[449] = 2;
        b[452] = 6;
        for i in 0..8 {
            b[984 + i * 3..987 + i * 3].copy_from_slice(&[i as u8, (i * 2) as u8, (i * 3) as u8]);
        }
        b
    }
    // Three two-frame objects: row ramps 0..7/8..15, solid 3, and solid 5.
    fn objects() -> [u8; 60] {
        let mut b = [0; 60];
        for f in 0..2 {
            for y in 0..2 {
                for x in 0..8 {
                    let c = if f == 0 { x + y * 8 } else { 15 - x - y * 8 };
                    for plane in 0..4 {
                        if c & (1 << plane) != 0 {
                            b[f * 10 + plane * 2 + y] |= 128 >> x;
                        }
                    }
                }
            }
            b[f * 10 + 8..f * 10 + 10].fill(255);
        }
        for (start, color) in [(20, 3), (40, 5)] {
            for f in 0..2 {
                for plane in 0..4 {
                    if color & (1 << plane) != 0 {
                        b[start + f * 10 + plane * 2..start + f * 10 + plane * 2 + 2].fill(255);
                    }
                }
                b[start + f * 10 + 8..start + f * 10 + 10].fill(255);
            }
        }
        b
    }
    const T: [u8; 8] = [255, 255, 0, 0, 0, 0, 0x0f, 0xf0];
    fn images<'a>(g: &'a [u8; 1056], o: &'a [u8]) -> VgagrImages<'a> {
        VgagrImages::new(GroundRecord::parse(g).unwrap(), &T, o).unwrap()
    }
    fn terrain<'a>(
        b: &[u8; 2048],
        g: &[u8; 1056],
        o: &[u8],
        out: &'a mut [IndexedPixel],
    ) -> LevelTerrain<'a> {
        compose_terrain(
            LevelRecord::parse(b).unwrap(),
            TerrainSource::Standard(TerrainGraphics::new(0, images(g, o)).unwrap()),
            out,
        )
        .unwrap()
    }
    fn sentinel() -> ScenePixel {
        ScenePixel {
            color: 99,
            terrain: true,
            object: true,
        }
    }
    fn output() -> Vec<ScenePixel> {
        vec![sentinel(); LEVEL_TERRAIN_PIXEL_COUNT]
    }
    fn draw<'a>(
        b: &[u8; 2048],
        g: &[u8; 1056],
        o: &[u8],
        t: LevelTerrain<'_>,
        s: ObjectFrameSelection<'_>,
        out: &'a mut [ScenePixel],
    ) -> Result<LevelScene<'a>, ObjectComposeError> {
        compose_objects(
            LevelRecord::parse(b).unwrap(),
            ObjectGraphics::new(0, images(g, o)).unwrap(),
            t,
            s,
            out,
        )
    }
    fn ter(b: &mut [u8; 2048]) {
        b[288..292].copy_from_slice(&[0, 16, 2, 0]);
    }
    #[test]
    fn empty_scene_copies_terrain_and_canonicalizes_transparency_with_independent_palette() {
        let mut b = level();
        ter(&mut b);
        let g = ground();
        let o = objects();
        let mut t = vec![IndexedPixel::default(); LEVEL_TERRAIN_PIXEL_COUNT];
        let map = terrain(&b, &g, &o, &mut t);
        let mut out = output();
        let ptr = out.as_ptr();
        let scene = draw(&b, &g, &o, map, ObjectFrameSelection::Preview, &mut out).unwrap();
        assert_eq!(scene.pixels().as_ptr(), ptr);
        assert_eq!(
            scene.pixel(4, 0),
            Some(ScenePixel {
                color: 9,
                terrain: true,
                object: false
            })
        );
        assert_eq!(scene.pixel(0, 0), Some(ScenePixel::default()));
        assert_eq!(scene.palette()[9], [4, 8, 12]);
        assert_eq!(scene.palette()[1], [64, 64, 224]);
        assert_eq!(scene.rgba(4, 0), Some([4, 8, 12, 255]));
        assert_eq!(scene.rgba(0, 0), Some([0; 4]));
        assert_eq!(scene.rgba(usize::MAX, 0), None);
        assert_eq!(scene.palette()[7], scene.palette()[8]);
        for (x, y) in [(1584, 0), (0, 160), (usize::MAX, 0), (0, usize::MAX)] {
            assert_eq!(scene.pixel(x, y), None);
        }
    }
    #[test]
    fn preview_initial_and_explicit_use_disk_indices_and_original_level_slots() {
        let mut b = level();
        put(&mut b, 31, 0, 0, 0, 0, false);
        let g = ground();
        let o = objects();
        let mut t = vec![IndexedPixel::default(); LEVEL_TERRAIN_PIXEL_COUNT];
        let map = terrain(&b, &g, &o, &mut t);
        let mut out = output();
        let mut indices = [usize::MAX; 32];
        indices[31] = 1;
        for (s, color) in [
            (ObjectFrameSelection::Preview, 0),
            (ObjectFrameSelection::Initial, 15),
            (ObjectFrameSelection::Explicit(&indices), 15),
        ] {
            let scene = draw(&b, &g, &o, map, s, &mut out).unwrap();
            assert_eq!(scene.pixel(0, 0).unwrap().color, color);
            assert!(scene.pixel(0, 0).unwrap().object);
        }
    }
    #[test]
    fn absolute_preview_window_is_not_rounded_to_an_animation_frame() {
        let mut b = level();
        put(&mut b, 0, 0, 0, 0, 0, false);
        let mut g = ground();
        g[23] = 1;
        let o = objects();
        let mut t = vec![IndexedPixel::default(); LEVEL_TERRAIN_PIXEL_COUNT];
        let map = terrain(&b, &g, &o, &mut t);
        let mut out = output();
        let f = images(&g, &o).object(0).unwrap().unwrap().preview();
        let scene = draw(&b, &g, &o, map, ObjectFrameSelection::Preview, &mut out).unwrap();
        for y in 0..2 {
            for x in 0..8 {
                let p = f.pixel(x, y).unwrap();
                let dst = scene.pixel(x, y).unwrap();
                assert_eq!(dst.object, p.opaque);
                if p.opaque {
                    assert_eq!(dst.color, p.color);
                }
            }
        }
    }
    #[test]
    fn default_objects_preserve_terrain_bits_and_later_slots_overwrite_prior_objects() {
        let mut b = level();
        ter(&mut b);
        put(&mut b, 0, 0, 0, 1, 0, false);
        put(&mut b, 31, 0, 0, 2, 0, false);
        let g = ground();
        let o = objects();
        let mut t = vec![IndexedPixel::default(); LEVEL_TERRAIN_PIXEL_COUNT];
        let map = terrain(&b, &g, &o, &mut t);
        let mut out = output();
        let scene = draw(&b, &g, &o, map, ObjectFrameSelection::Preview, &mut out).unwrap();
        for x in 0..8 {
            assert_eq!(
                scene.pixel(x, 0),
                Some(ScenePixel {
                    color: 5,
                    terrain: x >= 4,
                    object: true
                })
            );
        }
    }
    #[test]
    fn no_overwrite_respects_both_original_terrain_and_previous_objects() {
        let mut b = level();
        ter(&mut b);
        put(&mut b, 0, 0, 0, 1, 0x80, false);
        put(&mut b, 31, 0, 0, 2, 0x80, false);
        let g = ground();
        let o = objects();
        let mut t = vec![IndexedPixel::default(); LEVEL_TERRAIN_PIXEL_COUNT];
        let map = terrain(&b, &g, &o, &mut t);
        let mut out = output();
        let scene = draw(&b, &g, &o, map, ObjectFrameSelection::Preview, &mut out).unwrap();
        for x in 0..8 {
            assert_eq!(scene.pixel(x, 0).unwrap().color, if x >= 4 { 9 } else { 3 });
        }
    }
    #[test]
    fn only_on_terrain_draws_before_normal_objects_even_when_its_slot_is_later() {
        let mut b = level();
        ter(&mut b);
        put(&mut b, 0, 0, 0, 1, 0, false);
        put(&mut b, 31, 0, 0, 2, 0x40, false);
        let g = ground();
        let o = objects();
        let mut t = vec![IndexedPixel::default(); LEVEL_TERRAIN_PIXEL_COUNT];
        let map = terrain(&b, &g, &o, &mut t);
        let mut out = output();
        let scene = draw(&b, &g, &o, map, ObjectFrameSelection::Preview, &mut out).unwrap();
        for x in 0..8 {
            assert_eq!(scene.pixel(x, 0).unwrap().color, 3);
        }
    }
    #[test]
    fn all_four_modifier_combinations_give_terrain_condition_priority() {
        let g = ground();
        let o = objects();
        let mut t = vec![IndexedPixel::default(); LEVEL_TERRAIN_PIXEL_COUNT];
        let mut out = output();
        for flags in [0, 0x40, 0x80, 0xc0] {
            let mut b = level();
            ter(&mut b);
            put(&mut b, 31, 0, 0, 2, flags, false);
            let map = terrain(&b, &g, &o, &mut t);
            let scene = draw(&b, &g, &o, map, ObjectFrameSelection::Preview, &mut out).unwrap();
            for x in 0..8 {
                let ground = x >= 4;
                let selected = if flags & 0x40 != 0 {
                    ground
                } else if flags & 0x80 != 0 {
                    !ground
                } else {
                    true
                };
                assert_eq!(scene.pixel(x, 0).unwrap().object, selected);
                assert_eq!(scene.pixel(x, 0).unwrap().terrain, ground);
            }
        }
    }
    #[test]
    fn transparent_source_holes_preserve_background_and_masked_black_is_opaque() {
        let mut b = level();
        put(&mut b, 0, 0, 0, 1, 0, false);
        put(&mut b, 1, 0, 0, 0, 0, false);
        let g = ground();
        let mut o = objects();
        o[8] = 0xaa;
        let mut t = vec![IndexedPixel::default(); LEVEL_TERRAIN_PIXEL_COUNT];
        let map = terrain(&b, &g, &o, &mut t);
        let mut out = output();
        let scene = draw(&b, &g, &o, map, ObjectFrameSelection::Preview, &mut out).unwrap();
        for x in 0..8 {
            assert_eq!(
                scene.pixel(x, 0).unwrap().color,
                if x % 2 == 0 { x as u8 } else { 3 }
            );
        }
        assert!(scene.pixel(0, 0).unwrap().opaque());
        assert_eq!(scene.palette()[0], [0; 3]);
    }
    #[test]
    fn vertical_flip_reverses_pixels_and_masks_before_left_and_bottom_clipping() {
        let mut b = level();
        put(&mut b, 0, -3, 159, 0, 0, true);
        let g = ground();
        let mut o = objects();
        o[9] = 0xaa;
        let mut t = vec![IndexedPixel::default(); LEVEL_TERRAIN_PIXEL_COUNT];
        let map = terrain(&b, &g, &o, &mut t);
        let mut out = vec![sentinel(); LEVEL_TERRAIN_PIXEL_COUNT + 2];
        {
            let scene = draw(
                &b,
                &g,
                &o,
                map,
                ObjectFrameSelection::Preview,
                &mut out[1..LEVEL_TERRAIN_PIXEL_COUNT + 1],
            )
            .unwrap();
            for x in 0..5 {
                assert_eq!(scene.pixel(x, 159).unwrap().object, (x + 3) % 2 == 0);
                if (x + 3) % 2 == 0 {
                    assert_eq!(scene.pixel(x, 159).unwrap().color, (x + 11) as u8);
                }
            }
        }
        assert_eq!(out[0], sentinel());
        assert_eq!(out[LEVEL_TERRAIN_PIXEL_COUNT + 1], sentinel());
    }
    #[test]
    fn right_edge_and_fully_offscreen_extreme_unsigned_coordinates_are_safe() {
        let mut b = level();
        put(&mut b, 0, 1581, 1, 0, 0, false);
        put(&mut b, 30, 65519, 65535, 1, 0, false);
        put(&mut b, 31, -16, 160, 2, 0, false);
        let g = ground();
        let o = objects();
        let mut t = vec![IndexedPixel::default(); LEVEL_TERRAIN_PIXEL_COUNT];
        let map = terrain(&b, &g, &o, &mut t);
        let mut out = output();
        let scene = draw(&b, &g, &o, map, ObjectFrameSelection::Preview, &mut out).unwrap();
        assert_eq!(scene.pixels().iter().filter(|p| p.object).count(), 6);
        for x in 0..3 {
            assert_eq!(scene.pixel(1581 + x, 1).unwrap().color, x as u8);
        }
    }
    #[test]
    fn missing_last_offscreen_object_is_rejected_before_any_output_write() {
        let mut b = level();
        put(&mut b, 0, 0, 0, 1, 0, false);
        put(&mut b, 31, 65000, 65535, 15, 0, false);
        let g = ground();
        let o = objects();
        let mut t = vec![IndexedPixel::default(); LEVEL_TERRAIN_PIXEL_COUNT];
        let map = terrain(&b, &g, &o, &mut t);
        let mut out = output();
        assert_eq!(
            draw(&b, &g, &o, map, ObjectFrameSelection::Preview, &mut out),
            Err(ObjectComposeError::MissingObject { slot: 31, id: 15 })
        );
        assert!(out.iter().all(|p| *p == sentinel()));
    }
    #[test]
    fn invalid_explicit_frame_including_usize_max_keeps_output_and_input_unchanged() {
        let mut b = level();
        put(&mut b, 31, 65519, 65535, 0, 0, false);
        let g = ground();
        let o = objects();
        let mut t = vec![IndexedPixel::default(); LEVEL_TERRAIN_PIXEL_COUNT];
        let map = terrain(&b, &g, &o, &mut t);
        let before = map.pixels().to_vec();
        let mut out = output();
        let mut indices = [0; 32];
        for index in [2, usize::MAX] {
            indices[31] = index;
            let e = ObjectComposeError::Image {
                slot: 31,
                error: VgagrError::FrameIndex {
                    slot: 0,
                    index,
                    count: 2,
                },
            };
            assert_eq!(
                draw(
                    &b,
                    &g,
                    &o,
                    map,
                    ObjectFrameSelection::Explicit(&indices),
                    &mut out
                ),
                Err(e)
            );
            assert!(out.iter().all(|p| *p == sentinel()));
            assert_eq!(map.pixels(), before);
        }
    }
    #[test]
    fn invalid_graphics_tags_and_wrong_level_selector_keep_output_unchanged() {
        let b = level();
        let g = ground();
        let o = objects();
        let img = images(&g, &o);
        for found in [5, 255] {
            assert_eq!(
                ObjectGraphics::new(found, img),
                Err(ObjectComposeError::GraphicsIndex { found })
            );
        }
        let mut t = vec![IndexedPixel::default(); LEVEL_TERRAIN_PIXEL_COUNT];
        let map = terrain(&b, &g, &o, &mut t);
        let mut out = output();
        assert_eq!(
            compose_objects(
                LevelRecord::parse(&b).unwrap(),
                ObjectGraphics::new(4, img).unwrap(),
                map,
                ObjectFrameSelection::Preview,
                &mut out
            ),
            Err(ObjectComposeError::WrongGraphics {
                expected: 0,
                actual: 4
            })
        );
        assert!(out.iter().all(|p| *p == sentinel()));
    }
    #[test]
    fn both_wrong_output_lengths_are_rejected_without_writes() {
        let b = level();
        let g = ground();
        let o = objects();
        let mut t = vec![IndexedPixel::default(); LEVEL_TERRAIN_PIXEL_COUNT];
        let map = terrain(&b, &g, &o, &mut t);
        for len in [LEVEL_TERRAIN_PIXEL_COUNT - 1, LEVEL_TERRAIN_PIXEL_COUNT + 1] {
            let mut out = vec![sentinel(); len];
            assert_eq!(
                draw(&b, &g, &o, map, ObjectFrameSelection::Preview, &mut out),
                Err(ObjectComposeError::OutputSize {
                    expected: LEVEL_TERRAIN_PIXEL_COUNT,
                    actual: len
                })
            );
            assert!(out.iter().all(|p| *p == sentinel()));
        }
    }
    #[test]
    fn result_outlives_level_graphics_selection_and_terrain() {
        let mut out = output();
        let scene = {
            let mut b = level();
            put(&mut b, 31, 0, 0, 0, 0, false);
            let g = ground();
            let o = objects();
            let mut t = vec![IndexedPixel::default(); LEVEL_TERRAIN_PIXEL_COUNT];
            let map = terrain(&b, &g, &o, &mut t);
            let indices = [1; 32];
            draw(
                &b,
                &g,
                &o,
                map,
                ObjectFrameSelection::Explicit(&indices),
                &mut out,
            )
            .unwrap()
        };
        assert_eq!(scene.pixel(0, 0).unwrap().color, 15);
    }
    #[test]
    fn special_background_retains_its_palette_and_still_uses_objects_and_brick_override() {
        let mut inner = vec![0; 40];
        inner[3] = 63;
        inner[6] = 32;
        for _ in 0..4 {
            for plane in 0..3 {
                let mut n: usize = 4800;
                while n > 0 {
                    let k = n.min(128);
                    inner.extend_from_slice(&[(257 - k) as u8, if plane == 0 { 255 } else { 0 }]);
                    n -= k;
                }
            }
            inner.push(128);
        }
        let mut b = level();
        b[29] = 1;
        put(&mut b, 0, 304, 0, 2, 0x40, false);
        let g = ground();
        let o = objects();
        let mut planes = vec![0; VGASPEC_PLANAR_SIZE];
        let mut t = vec![IndexedPixel::default(); LEVEL_TERRAIN_PIXEL_COUNT];
        let map = compose_terrain(
            LevelRecord::parse(&b).unwrap(),
            TerrainSource::Special(
                SpecialTerrain::decode_into(0, VgaSpecRecord::parse(&inner).unwrap(), &mut planes)
                    .unwrap(),
            ),
            &mut t,
        )
        .unwrap();
        let mut out = output();
        let scene = draw(&b, &g, &o, map, ObjectFrameSelection::Preview, &mut out).unwrap();
        assert_eq!(scene.palette()[7], [124, 124, 0]);
        assert_eq!(scene.palette()[8], [124, 124, 0]);
        assert_eq!(scene.palette()[9], [255, 0, 0]);
        assert_eq!(scene.terrain_palette()[1], [252, 0, 0]);
        assert_eq!(scene.rgba(304, 0), Some([240, 32, 32, 255]));
        assert_eq!(scene.rgba(313, 0), Some([252, 0, 0, 255]));
        assert_eq!(
            scene.pixel(304, 0),
            Some(ScenePixel {
                color: 5,
                terrain: true,
                object: true
            })
        );
        assert_eq!(scene.pixel(313, 0).unwrap().color, 9);
    }
}
