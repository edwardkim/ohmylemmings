//! Shared HD draw list and revisioned texture cache. Hosts upload changed textures
//! and composite them; this module never performs I/O, advances time or writes a run.
use super::{
    Control,
    assets::{AssetFrame, AssetKey, AssetPack},
    font, snapshot,
    ui::{self, Ink},
};
use crate::{Assets, LevelRun, assets::object_frame, panel};
use lemmings_core::{MotionAction, MotionDirection};
use lemmings_resources::{
    GroundObject, LemmingAnimationInfo, LemmingSpriteKind, LemmingSprites, ObjectPlacement,
    SpriteDirection,
};
use std::error::Error;

const CHUNK: usize = 32;
#[derive(Debug)]
pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub revision: u32,
}
impl Texture {
    fn new(width: u32, height: u32, rgba: Vec<u8>) -> Self {
        Self {
            width,
            height,
            rgba,
            revision: 1,
        }
    }
    fn replace(&mut self, width: u32, height: u32, rgba: Vec<u8>) {
        if self.width != width || self.height != height || self.rgba != rgba {
            self.width = width;
            self.height = height;
            self.rgba = rgba;
            self.revision = self.revision.wrapping_add(1).max(1);
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Draw {
    Fill {
        rect: [i32; 4],
        color: [u8; 4],
    },
    Blit {
        texture: usize,
        source: [u32; 4],
        dest: [i32; 4],
    },
    GameClip(bool),
}
impl Draw {
    /// Version1, fixed12 i32 records. This is a typed product wire contract,
    /// independent of Debug diagnostics and the legacy simulation snapshot.
    pub fn wire(&self) -> [i32; 12] {
        match *self {
            Self::Fill {
                rect: [x, y, w, h],
                color: [r, g, b, a],
            } => [
                0,
                x,
                y,
                w,
                h,
                r.into(),
                g.into(),
                b.into(),
                a.into(),
                0,
                0,
                0,
            ],
            Self::Blit {
                texture,
                source: [x, y, w, h],
                dest: [dx, dy, dw, dh],
            } => [
                1,
                texture as i32,
                x as i32,
                y as i32,
                w as i32,
                h as i32,
                dx,
                dy,
                dw,
                dh,
                0,
                0,
            ],
            Self::GameClip(enabled) => [2, i32::from(enabled), 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        }
    }
}
struct Object {
    placement: ObjectPlacement,
    meta: GroundObject,
    set: u8,
    frames: Vec<Vec<u8>>,
    texture: usize,
    last: Option<(usize, u64, Vec<usize>)>,
}
pub struct Renderer {
    textures: Vec<Texture>,
    frames: Vec<AssetFrame>,
    base: usize,
    fallback_frames: Vec<AssetFrame>,
    objects: Vec<Object>,
    width: usize,
    height: usize,
    chunks_x: usize,
    chunks_y: usize,
    chunk_base: usize,
    dirty: Vec<bool>,
    old_mask: Vec<u8>,
    old_bricks: Vec<u8>,
    initial: Vec<[u8; 4]>,
    material: Option<u8>,
    special: Option<u8>,
    mini: usize,
    terrain_rebuilds: u64,
    missing: Vec<(u8, u16)>,
    drawn_hd: Vec<(u8, u16)>,
    hd_portals: Vec<(u8, u8, u16)>,
    source_portals: Vec<(u8, u8, u16)>,
    hd_exit_effects: Vec<(u8, u8, u16)>,
    source_exit_effects: Vec<(u8, u8, u16)>,
    hd_hazards: Vec<(u8, u8, u16)>,
    source_hazards: Vec<(u8, u8, u16)>,
    hd_traps: Vec<(u8, u8, u16)>,
    source_traps: Vec<(u8, u8, u16)>,
    hd_decor: Vec<(u8, u8, u16)>,
    source_decor: Vec<(u8, u8, u16)>,
    epoch: u64,
    font: usize,
    skin: usize,
}
impl Renderer {
    pub fn fallback() -> Self {
        Self {
            textures: Vec::new(),
            frames: Vec::new(),
            base: 0,
            fallback_frames: Vec::new(),
            objects: Vec::new(),
            width: 0,
            height: 0,
            chunks_x: 0,
            chunks_y: 0,
            chunk_base: 0,
            dirty: Vec::new(),
            old_mask: Vec::new(),
            old_bricks: Vec::new(),
            initial: Vec::new(),
            material: None,
            special: None,
            mini: 0,
            terrain_rebuilds: 0,
            missing: Vec::new(),
            drawn_hd: Vec::new(),
            hd_portals: Vec::new(),
            source_portals: Vec::new(),
            hd_exit_effects: Vec::new(),
            source_exit_effects: Vec::new(),
            hd_hazards: Vec::new(),
            source_hazards: Vec::new(),
            hd_traps: Vec::new(),
            source_traps: Vec::new(),
            hd_decor: Vec::new(),
            source_decor: Vec::new(),
            epoch: 0,
            font: 0,
            skin: 0,
        }
    }
    pub fn new(bytes: &[u8]) -> Result<Self, Box<dyn Error>> {
        let pack = AssetPack::parse(bytes)?;
        let mut result = Self::fallback();
        for atlas in pack.atlases() {
            let mut rgba = atlas.rgba().to_vec();
            for p in rgba.chunks_exact_mut(4) {
                if p[3] <= atlas.alpha_cutoff() {
                    p[3] = 0;
                }
            }
            result
                .textures
                .push(Texture::new(atlas.width(), atlas.height(), rgba));
        }
        result.frames = pack.frames().to_vec();
        result.base = result.textures.len();
        Ok(result)
    }
    pub fn textures(&self) -> &[Texture] {
        &self.textures
    }
    pub fn terrain_rebuilds(&self) -> u64 {
        self.terrain_rebuilds
    }
    pub fn missing(&self) -> &[(u8, u16)] {
        &self.missing
    }
    pub fn info(&self) -> String {
        let missing = self
            .missing
            .iter()
            .map(|(s, f)| format!("[{s},{f}]"))
            .collect::<Vec<_>>()
            .join(",");
        let drawn = self
            .drawn_hd
            .iter()
            .map(|(s, f)| format!("[{s},{f}]"))
            .collect::<Vec<_>>()
            .join(",");
        let portal_keys = |keys: &[(u8, u8, u16)]| {
            keys.iter()
                .map(|(set, id, frame)| format!("[{set},{id},{frame}]"))
                .collect::<Vec<_>>()
                .join(",")
        };
        let portals = portal_keys(&self.hd_portals);
        let originals = portal_keys(&self.source_portals);
        let effects = portal_keys(&self.hd_exit_effects);
        let original_effects = portal_keys(&self.source_exit_effects);
        let hazards = portal_keys(&self.hd_hazards);
        let original_hazards = portal_keys(&self.source_hazards);
        let traps = portal_keys(&self.hd_traps);
        let original_traps = portal_keys(&self.source_traps);
        let font_first = self.font;
        let decor = portal_keys(&self.hd_decor);
        let original_decor = portal_keys(&self.source_decor);
        format!(
            "{{\"version\":1,\"width\":1920,\"ui_font\":\"Noto Sans SemiBold\",\"ui_glyphs\":95,\"ui_font_first\":{font_first},\"ui_font_atlases\":3,\"height\":1080,\"actor_frames_available\":{},\"portal_frames_available\":{},\"trap_frames_available\":{},\"hazard_frames_available\":{},\"exit_effect_frames_available\":{},\"terrain_rebuilds\":{},\"source_terrain\":{},\"special_panorama\":{},\"terrain_material\":{},\"one_way_objects\":{},\"missing_actor_frames\":[{missing}],\"hd_actor_frames\":[{drawn}],\"hd_portal_frames\":[{portals}],\"original_portal_frames\":[{originals}],\"hd_exit_effect_frames\":[{effects}],\"original_exit_effect_frames\":[{original_effects}],\"hd_hazard_frames\":[{hazards}],\"original_hazard_frames\":[{original_hazards}],\"hd_trap_frames\":[{traps}],\"original_trap_frames\":[{original_traps}],\"hd_decor_frames\":[{decor}],\"original_decor_frames\":[{original_decor}]}}",
            self.frames
                .iter()
                .filter(|f| matches!(f.key, AssetKey::Actor { .. }))
                .count(),
            self.frames
                .iter()
                .filter(|f| matches!(f.key, AssetKey::Object { id: 0 | 1, .. }))
                .count(),
            self.frames.iter().filter(|f| matches!(f.key, AssetKey::Object { set, id, .. } if super::assets::trap_frame_count(set, id).is_some())).count(),
            self.frames.iter().filter(|f| matches!(f.key, AssetKey::Object { set, id, .. } if super::assets::hazard_frame_count(set, id).is_some())).count(),
            self.frames.iter().filter(|f| matches!(f.key, AssetKey::Object { set, id, .. } if super::assets::exit_effect_frame_count(set, id).is_some())).count(),
            self.terrain_rebuilds,
            self.material.is_none() && self.special.is_none(),
            self.special.map_or_else(|| "null".into(), |set| set.to_string()),
            self.material
                .map_or_else(|| "null".into(), |set| set.to_string()),
            self.objects
                .iter()
                .filter(|o| matches!(o.meta.trigger_effect, 7 | 8))
                .count()
        )
    }
    fn select_material(&mut self, set: u8, special: bool) {
        self.material = (!special
            && AssetKey::material(set).is_some_and(|key| self.frames.iter().any(|f| f.key == key)))
        .then_some(set);
    }
    fn select_special(&mut self, set: Option<u8>) {
        self.special = set.filter(|&set| {
            self.frames
                .iter()
                .any(|f| f.key == AssetKey::Special { set })
        });
    }
    pub fn reset(&mut self, assets: &Assets) -> Result<(), Box<dyn Error>> {
        if assets.width == 0 || assets.width > 4096 || assets.height != 160 {
            return Err("unsupported HD world dimensions".into());
        }
        self.textures.truncate(self.base);
        self.font = self.textures.len();
        for rgb in font::COLORS {
            let rgba = font::ALPHA
                .iter()
                .flat_map(|&a| [rgb[0], rgb[1], rgb[2], a])
                .collect();
            self.textures
                .push(Texture::new(font::WIDTH, font::HEIGHT, rgba));
        }
        self.skin = self.textures.len();
        self.textures
            .push(Texture::new(ui::SKIN_WIDTH, ui::SKIN_HEIGHT, ui::skin()));
        self.fallback_frames.clear();
        self.objects.clear();
        self.hd_portals.clear();
        self.source_portals.clear();
        self.hd_exit_effects.clear();
        self.source_exit_effects.clear();
        self.hd_hazards.clear();
        self.source_hazards.clear();
        self.hd_traps.clear();
        self.source_traps.clear();
        self.hd_decor.clear();
        self.source_decor.clear();
        self.missing.clear();
        self.width = assets.width;
        self.height = assets.height;
        self.initial = assets.hd_terrain()?;
        let sprites = LemmingSprites::parse(&assets.sprites)?;
        let texture = self.textures.len();
        let mut atlas = vec![0u8; 1024 * 352 * 4];
        let mut tile = 0;
        for slot in 0..28 {
            let info = LemmingAnimationInfo::for_slot(slot).ok_or("animation slot")?;
            let animation =
                sprites.for_sprite(info.kind, info.direction.unwrap_or(SpriteDirection::Right));
            for frame in 0..u16::from(info.frame_count) {
                let image = animation.frame(frame as usize)?;
                let (x, y) = ((tile % 32) * 32, (tile / 32) * 32);
                for py in 0..info.height as usize {
                    for px in 0..info.width as usize {
                        let p = image.pixel(px, py).ok_or("sprite pixel")?;
                        if p.opaque {
                            let at = ((y + py) * 1024 + x + px) * 4;
                            atlas[at..at + 3].copy_from_slice(&assets.palette[p.color as usize]);
                            atlas[at + 3] = 255;
                        }
                    }
                }
                self.fallback_frames.push(AssetFrame {
                    key: AssetKey::Actor {
                        slot: slot as u8,
                        frame,
                    },
                    atlas: texture as u16,
                    rect: [x as u32, y as u32, info.width.into(), info.height.into()],
                    display: [u16::from(info.width) * 6, u16::from(info.height) * 6],
                    anchor: [i16::from(info.foot_x) * 6, i16::from(info.foot_y) * 6],
                });
                tile += 1;
            }
        }
        self.textures.push(Texture::new(1024, 352, atlas));
        self.select_material(0, false);
        self.select_special(None);
        if let Some((set, level, ground, images)) = assets.hd_art()? {
            self.select_material(set, level.header().special_graphic_set().is_some());
            self.select_special(level.header().special_graphic_set());
            // HD keys select object art, including arrows clipped to current
            // terrain. Original trigger masks never select terrain color.
            for placement in level.objects() {
                let meta = ground
                    .object(placement.id as usize)
                    .ok_or("object metadata")?;
                let animation = images
                    .object(placement.id as usize)?
                    .ok_or("object frames")?;
                let mut frames = Vec::new();
                for frame in 0..meta.frame_count {
                    let image = animation.frame(frame as usize)?;
                    let mut rgba = vec![0; meta.width as usize * meta.height as usize * 4];
                    for y in 0..meta.height as usize {
                        for x in 0..meta.width as usize {
                            let p = image.pixel(x, y).ok_or("object pixel")?;
                            if p.opaque {
                                let at = (y * meta.width as usize + x) * 4;
                                rgba[at..at + 3].copy_from_slice(&assets.palette[p.color as usize]);
                                rgba[at + 3] = 255;
                            }
                        }
                    }
                    frames.push(rgba);
                }
                let texture = self.textures.len();
                self.textures.push(Texture::new(1, 1, vec![0; 4]));
                self.objects.push(Object {
                    placement,
                    meta,
                    set,
                    frames,
                    texture,
                    last: None,
                });
            }
        }
        self.chunks_x = self.width.div_ceil(CHUNK);
        self.chunks_y = self.height.div_ceil(CHUNK);
        self.chunk_base = self.textures.len();
        self.dirty = vec![true; self.chunks_x * self.chunks_y];
        for _ in 0..self.dirty.len() {
            let mut texture = Texture::new(1, 1, vec![0; 4]);
            texture.revision = 0;
            self.textures.push(texture);
        }
        self.old_mask = vec![255; self.width * self.height];
        self.old_bricks = self.old_mask.clone();
        self.mini = self.textures.len();
        self.textures
            .push(Texture::new(248, 88, vec![0; 248 * 88 * 4]));
        self.epoch = self.epoch.wrapping_add(1);
        Ok(())
    }
    fn fill(out: &mut Vec<Draw>, rect: [i32; 4], color: [u8; 4]) {
        out.push(Draw::Fill { rect, color });
    }
    fn text(&self, out: &mut Vec<Draw>, x: i32, y: i32, value: &str, size: i32, ink: Ink) {
        font::draw(out, self.font + ink as usize, x, y, value, size);
    }
    fn centered(&self, out: &mut Vec<Draw>, [x, w, y]: [i32; 3], value: &str, size: i32, ink: Ink) {
        self.text(
            out,
            x + (w - font::width(value, size)) / 2,
            y,
            value,
            size,
            ink,
        );
    }
    fn blit(out: &mut Vec<Draw>, f: &AssetFrame, x: i32, y: i32) {
        out.push(Draw::Blit {
            texture: f.atlas as usize,
            source: f.rect,
            dest: [
                x - f.anchor[0] as i32,
                y - f.anchor[1] as i32,
                f.display[0] as i32,
                f.display[1] as i32,
            ],
        });
    }
    fn terrain(
        &mut self,
        mask: &[u8],
        bricks: &[u8],
        steel: &[u8],
        camera: i32,
        out: &mut Vec<Draw>,
    ) -> Result<(), Box<dyn Error>> {
        if mask.len() != self.old_mask.len() || bricks.len() != mask.len() {
            return Err("HD mask length".into());
        }
        let mut changed = false;
        for i in 0..mask.len() {
            if mask[i] != self.old_mask[i] || bricks[i] != self.old_bricks[i] {
                self.dirty[(i / self.width / CHUNK) * self.chunks_x + (i % self.width / CHUNK)] =
                    true;
                changed = true;
            }
        }
        if changed {
            self.old_mask.copy_from_slice(mask);
            self.old_bricks.copy_from_slice(bricks);
            self.epoch = self.epoch.wrapping_add(1);
        }
        let material = self
            .material
            .and_then(AssetKey::material)
            .and_then(|key| self.frames.iter().find(|f| f.key == key))
            .copied();
        let special = self
            .special
            .and_then(|set| {
                self.frames
                    .iter()
                    .find(|f| f.key == AssetKey::Special { set })
            })
            .copied();
        for cy in 0..self.chunks_y {
            for cx in 0..self.chunks_x {
                let x = cx * CHUNK;
                let y = cy * CHUNK;
                let w = CHUNK.min(self.width - x);
                let h = CHUNK.min(self.height - y);
                if x as i32 + w as i32 <= camera || x as i32 >= camera + 320 {
                    continue;
                }
                let id = cy * self.chunks_x + cx;
                let texture = self.chunk_base + id;
                if self.dirty[id] {
                    let mut rgba = vec![0; w * h * 36 * 4];
                    for py in 0..h * 6 {
                        for px in 0..w * 6 {
                            let wx = x + px / 6;
                            let wy = y + py / 6;
                            let i = wy * self.width + wx;
                            if mask[i] == 0 {
                                continue;
                            }
                            let color = if bricks[i] != 0 {
                                if py % 6 == 0 || (x * 6 + px).is_multiple_of(36) {
                                    [118, 75, 32, 255]
                                } else {
                                    [207, 158, 79, 255]
                                }
                            } else if steel[i] != 0 {
                                [132, 150, 166, 255]
                            } else if let Some(r) = special.filter(|_| (304..1264).contains(&wx)) {
                                // VGASPEC is960x160 at x304 (terrain.rs). Sample in
                                // world HD coordinates so scrolling/chunk edits never rephase it.
                                let source = &self.textures[r.atlas as usize];
                                let sx = r.rect[0] as usize
                                    + (x * 6 + px - 304 * 6) * r.rect[2] as usize / (960 * 6);
                                let sy = r.rect[1] as usize
                                    + (y * 6 + py) * r.rect[3] as usize / (160 * 6);
                                let at = (sy * source.width as usize + sx) * 4;
                                [
                                    source.rgba[at],
                                    source.rgba[at + 1],
                                    source.rgba[at + 2],
                                    255,
                                ]
                            } else if let Some(r) = material {
                                let source = &self.textures[r.atlas as usize];
                                let sx = r.rect[0]
                                    + (((x * 6 + px) * r.rect[2] as usize / 256)
                                        % r.rect[2] as usize)
                                        as u32;
                                let sy = r.rect[1]
                                    + (((y * 6 + py) * r.rect[3] as usize / 256)
                                        % r.rect[3] as usize)
                                        as u32;
                                let at = (sy as usize * source.width as usize + sx as usize) * 4;
                                [
                                    source.rgba[at],
                                    source.rgba[at + 1],
                                    source.rgba[at + 2],
                                    255,
                                ]
                            } else {
                                let mut c = self.initial[i];
                                c[3] = 255;
                                c
                            };
                            let at = (py * w * 6 + px) * 4;
                            rgba[at..at + 4].copy_from_slice(&color);
                        }
                    }
                    self.textures[texture].replace((w * 6) as u32, (h * 6) as u32, rgba);
                    self.dirty[id] = false;
                    self.terrain_rebuilds += 1;
                }
                out.push(Draw::Blit {
                    texture,
                    source: [0, 0, (w * 6) as u32, (h * 6) as u32],
                    dest: [
                        (x as i32 - camera) * 6,
                        y as i32 * 6,
                        (w * 6) as i32,
                        (h * 6) as i32,
                    ],
                });
            }
        }
        if changed {
            let mut rgba = vec![0; 248 * 88 * 4];
            for y in 0..88 {
                for x in 0..248 {
                    let wx = x * self.width / 248;
                    let wy = y * self.height / 88;
                    let solid = mask[wy * self.width + wx] != 0;
                    let at = (y * 248 + x) * 4;
                    rgba[at..at + 4].copy_from_slice(if solid {
                        &[146, 183, 134, 255]
                    } else {
                        &[15, 28, 33, 255]
                    });
                }
            }
            self.textures[self.mini].replace(248, 88, rgba);
        }
        Ok(())
    }
    fn objects(
        &mut self,
        run: &LevelRun,
        camera: i32,
        out: &mut Vec<Draw>,
    ) -> Result<(), Box<dyn Error>> {
        self.hd_portals.clear();
        self.source_portals.clear();
        self.hd_exit_effects.clear();
        self.source_exit_effects.clear();
        self.hd_hazards.clear();
        self.source_hazards.clear();
        self.hd_traps.clear();
        self.source_traps.clear();
        self.hd_decor.clear();
        self.source_decor.clear();
        let mask = run.session.render_terrain();
        let mut occupied = mask.to_vec();
        let mut compact = 0;
        let mut selected = Vec::new();
        for o in &self.objects {
            let trap = compact;
            if o.placement.id != 1 {
                compact += 1;
            }
            let frame = if o.placement.id != 1 && o.meta.trigger_effect == 4 && trap < 16 {
                run.session.traps()[trap].map(|t| t.frame() as usize)
            } else {
                None
            }
            .unwrap_or_else(|| {
                object_frame(
                    o.placement.id == 1,
                    o.meta.animation_loops(),
                    o.meta.first_frame as usize,
                    o.meta.frame_count as usize,
                    run.session.scheduler.step_state().iteration,
                )
            });
            selected.push(frame);
        }
        for on_terrain in [true, false] {
            for index in 0..self.objects.len() {
                let o = &mut self.objects[index];
                let p = o.placement;
                if p.flags.only_on_terrain != on_terrain {
                    continue;
                }
                let frame = selected[index];
                let original = o.frames.get(frame).ok_or("object frame out of range")?;
                let hd = self.frames.iter().find(|f| {
                    f.key
                        == AssetKey::Object {
                            set: o.set,
                            id: p.id,
                            frame: frame as u16,
                        }
                });
                if p.id <= 1 {
                    let key = (o.set, p.id, frame as u16);
                    if hd.is_some() {
                        self.hd_portals.push(key);
                    } else {
                        self.source_portals.push(key);
                    }
                }
                if super::assets::exit_effect_frame_count(o.set, p.id).is_some() {
                    let key = (o.set, p.id, frame as u16);
                    if hd.is_some() {
                        self.hd_exit_effects.push(key);
                    } else {
                        self.source_exit_effects.push(key);
                    }
                }
                if super::assets::hazard_frame_count(o.set, p.id).is_some() {
                    let key = (o.set, p.id, frame as u16);
                    if hd.is_some() {
                        self.hd_hazards.push(key);
                    } else {
                        self.source_hazards.push(key);
                    }
                }
                if super::assets::trap_frame_count(o.set, p.id).is_some() {
                    let key = (o.set, p.id, frame as u16);
                    if hd.is_some() {
                        self.hd_traps.push(key);
                    } else {
                        self.source_traps.push(key);
                    }
                }
                if super::assets::decor_frame_count(o.set, p.id).is_some() {
                    let key = (o.set, p.id, frame as u16);
                    if hd.is_some() {
                        self.hd_decor.push(key);
                    } else {
                        self.source_decor.push(key);
                    }
                }
                let (w, h, ax, ay) = hd.map_or(
                    (o.meta.width as usize * 6, o.meta.height as usize * 6, 0, 0),
                    |f| {
                        (
                            f.display[0] as usize,
                            f.display[1] as usize,
                            i32::from(f.anchor[0]),
                            i32::from(f.anchor[1]),
                        )
                    },
                );
                let origin_x = p.x * 6 - ax;
                let origin_y = if p.flags.upside_down {
                    i32::from(p.y) * 6 + i32::from(o.meta.height) * 6 - h as i32 + ay
                } else {
                    i32::from(p.y) * 6 - ay
                };
                // Flags operate on the original logical drawing provenance. Atlas
                // color/alpha never becomes physical terrain or a trigger.
                if o.last.as_ref().is_none_or(|(f, e, phases)| {
                    *f != frame || *e != self.epoch || phases != &selected
                }) {
                    let mut rgba = vec![0; w * h * 4];
                    for y in 0..h {
                        for x in 0..w {
                            let world_x = (origin_x + x as i32).div_euclid(6);
                            let world_y = (origin_y + y as i32).div_euclid(6);
                            if world_x < 0
                                || world_y < 0
                                || world_x >= self.width as i32
                                || world_y >= self.height as i32
                            {
                                continue;
                            }
                            let at = world_y as usize * self.width + world_x as usize;
                            if if p.flags.only_on_terrain {
                                mask[at] == 0
                            } else {
                                p.flags.no_overwrite && occupied[at] != 0
                            } {
                                continue;
                            }
                            let sy = if p.flags.upside_down { h - 1 - y } else { y };
                            let color = if let Some(f) = hd {
                                let t = &self.textures[f.atlas as usize];
                                let sx = f.rect[0] as usize + x * f.rect[2] as usize / w;
                                let sy = f.rect[1] as usize + sy * f.rect[3] as usize / h;
                                let at = (sy * t.width as usize + sx) * 4;
                                &t.rgba[at..at + 4]
                            } else {
                                let at = (sy / 6 * o.meta.width as usize + x / 6) * 4;
                                &original[at..at + 4]
                            };
                            rgba[(y * w + x) * 4..(y * w + x) * 4 + 4].copy_from_slice(color);
                        }
                    }
                    self.textures[o.texture].replace(w as u32, h as u32, rgba);
                    o.last = Some((frame, self.epoch, selected.clone()));
                }
                for y in 0..o.meta.height as usize {
                    for x in 0..o.meta.width as usize {
                        let wx = p.x + x as i32;
                        let wy = p.y as usize + y;
                        if wx < 0 || wx >= self.width as i32 || wy >= self.height {
                            continue;
                        }
                        let sy = if p.flags.upside_down {
                            o.meta.height as usize - 1 - y
                        } else {
                            y
                        };
                        if original[(sy * o.meta.width as usize + x) * 4 + 3] == 0 {
                            continue;
                        }
                        let at = wy * self.width + wx as usize;
                        if (p.flags.only_on_terrain && mask[at] == 0)
                            || (!p.flags.only_on_terrain
                                && p.flags.no_overwrite
                                && occupied[at] != 0)
                        {
                            continue;
                        }
                        occupied[at] = 1;
                    }
                }
                out.push(Draw::Blit {
                    texture: o.texture,
                    source: [0, 0, w as u32, h as u32],
                    dest: [origin_x - camera * 6, origin_y, w as i32, h as i32],
                });
            }
        }
        Ok(())
    }
    pub fn frame(&mut self, run: &LevelRun) -> Result<Vec<Draw>, Box<dyn Error>> {
        let s = snapshot(run)?;
        if (self.width, self.height) != (s.terrain.width, s.terrain.height) {
            return Err("reset HD scene after replacement".into());
        }
        let mut out = Vec::new();
        Self::fill(&mut out, [0, 0, 1920, 1080], [14, 25, 34, 255]);
        out.push(Draw::GameClip(true));
        self.terrain(
            s.terrain.visible,
            s.terrain.bricks,
            s.terrain.steel,
            s.camera,
            &mut out,
        )?;
        self.objects(run, s.camera, &mut out)?;
        self.missing.clear();
        self.drawn_hd.clear();
        for actor in &s.actors {
            let a = actor.state;
            if a.removed.is_some() {
                if a.particle_timer > 0
                    && let Some(data) = &run.assets.particles
                {
                    let particles = lemmings_resources::ExplosionParticles::parse(data)?;
                    for i in 0..80 {
                        if let Some((dx, dy)) = particles.offset(a.particle_frame as usize, i) {
                            Self::fill(
                                &mut out,
                                [
                                    (a.x + dx as i32 - s.camera) * 6,
                                    (a.y + dy as i32) * 6,
                                    3,
                                    3,
                                ],
                                [242, 188, 77, 255],
                            );
                        }
                    }
                }
                continue;
            }
            if !(-4096..=6016).contains(&actor.foot.0) || !(-4096..=5056).contains(&actor.foot.1) {
                continue;
            }
            let slot =
                actor_slot(a.action, a.direction).ok_or("missing original animation metadata")?;
            let key = AssetKey::Actor {
                slot,
                frame: a.frame.into(),
            };
            if self.frames.iter().any(|f| f.key == key) {
                self.drawn_hd.push((slot, a.frame.into()));
            }
            let frame = self
                .frames
                .iter()
                .find(|f| f.key == key)
                .or_else(|| {
                    if !self.missing.contains(&(slot, a.frame.into())) {
                        self.missing.push((slot, a.frame.into()));
                    }
                    self.fallback_frames.iter().find(|f| f.key == key)
                })
                .ok_or("actor frame out of range")?;
            Self::blit(&mut out, frame, actor.foot.0 as i32, actor.foot.1 as i32);
            if let Some(digit) = a.countdown_digit() {
                self.text(
                    &mut out,
                    actor.foot.0 as i32,
                    actor.foot.1 as i32 - frame.anchor[1] as i32 - 32,
                    &digit.to_string(),
                    24,
                    Ink::Accent,
                );
            }
            if s.hud.hits.primary == Some(a.id) {
                Self::fill(
                    &mut out,
                    [actor.foot.0 as i32 - 15, actor.foot.1 as i32 + 3, 30, 3],
                    [116, 238, 163, 255],
                );
            }
        }
        out.push(Draw::GameClip(false));
        Self::fill(&mut out, [0, 960, 1920, 120], ui::BACKGROUND);
        Self::fill(&mut out, [0, 960, 1920, 2], [79, 110, 89, 255]);
        let title = font::fit(
            &format!(
                "{} {:02}  -  {}",
                s.level.difficulty().name(),
                s.level.number(),
                s.title
            ),
            20,
            368,
        );
        self.text(&mut out, 16, 968, &title, 20, Ink::Main);
        self.text(
            &mut out,
            16,
            995,
            &format!(
                "Alive {}   Saved {}%{}",
                s.hud.alive,
                s.hud.rescued_percent,
                s.hud
                    .target_percent
                    .map_or_else(String::new, |t| format!(" / {t}%"))
            ),
            17,
            Ink::Subtle,
        );
        self.text(
            &mut out,
            16,
            1020,
            &format!(
                "Time {}:{:02}",
                s.hud.seconds_remaining / 60,
                s.hud.seconds_remaining % 60
            ),
            24,
            Ink::Main,
        );
        self.text(
            &mut out,
            16,
            1052,
            &format!("Released {} / {}", s.hud.released, s.hud.population),
            15,
            Ink::Subtle,
        );
        let selected = skill_name(s.hud.selected).0;
        self.text(&mut out, 212, 1022, selected, 20, Ink::Accent);
        self.text(
            &mut out,
            212,
            1052,
            "Select skill, click a lemming",
            12,
            Ink::Subtle,
        );
        for region in super::control_regions() {
            let c = region.control;
            if c == Control::Minimap {
                continue;
            }
            let x = region.rect.x as i32;
            let w = region.rect.width as i32;
            let active = matches!(c,Control::Skill(skill) if skill==s.hud.selected)
                || c == Control::Pause && s.hud.paused
                || c == Control::Nuke && s.hud.nuked;
            ui::card(
                &mut out,
                self.skin,
                u32::from(active),
                [x + 3, 965, w - 6, 110],
            );
            let (name, key, value, available) = match c {
                Control::Slower => (
                    "Rate -",
                    "Down",
                    s.hud.release_rate.to_string(),
                    s.hud.release_rate > s.hud.minimum_release_rate,
                ),
                Control::Faster => (
                    "Rate +",
                    "Up",
                    s.hud.release_rate.to_string(),
                    s.hud.release_rate < 99,
                ),
                Control::Skill(skill) => {
                    let (name, key) = skill_name(skill);
                    let count =
                        s.hud.skills[panel::SKILLS.iter().position(|&v| v == skill).unwrap()];
                    (name, key, count.to_string(), count > 0)
                }
                Control::Pause => (
                    "Pause",
                    "Space",
                    if s.hud.paused { "Paused" } else { "Pause" }.into(),
                    true,
                ),
                Control::Nuke => (
                    "Nuke",
                    "N",
                    if s.hud.nuked { "Active" } else { "2 clicks" }.into(),
                    !s.hud.nuked,
                ),
                Control::Restart => (
                    "Retry",
                    if s.hud.result.is_some() { "R" } else { "" },
                    "Restart".into(),
                    true,
                ),
                Control::Minimap => unreachable!(),
            };
            let ink = if available { Ink::Main } else { Ink::Subtle };
            self.text(
                &mut out,
                x + w - 9 - font::width(key, 11),
                968,
                key,
                11,
                Ink::Subtle,
            );
            if let Some(f) = self.frames.iter().find(|f| f.key == AssetKey::Icon(c)) {
                out.push(Draw::Blit {
                    texture: f.atlas as usize,
                    source: f.rect,
                    dest: [x + (w - 52) / 2, 978, 52, 52],
                });
            } else {
                self.centered(&mut out, [x, w, 987], name, 16, ink);
            }
            self.centered(
                &mut out,
                [x, w, 1032],
                name,
                14,
                if active { Ink::Accent } else { ink },
            );
            self.centered(
                &mut out,
                [x, w, 1050],
                &value,
                if matches!(c, Control::Skill(_) | Control::Slower | Control::Faster) {
                    22
                } else {
                    14
                },
                ink,
            );
        }
        ui::card(&mut out, self.skin, 0, [1660, 972, 256, 96]);
        self.centered(&mut out, [1664, 248, 960], "Map", 12, Ink::Subtle);
        out.push(Draw::Blit {
            texture: self.mini,
            source: [0, 0, 248, 88],
            dest: [1664, 976, 248, 88],
        });
        let marker = 1664 + s.camera * 248 / self.width as i32;
        let mw = 320 * 248 / self.width as i32;
        for rect in [
            [marker, 976, mw, 2],
            [marker, 1062, mw, 2],
            [marker, 976, 2, 88],
            [marker + mw - 2, 976, 2, 88],
        ] {
            Self::fill(&mut out, rect, [227, 242, 182, 255]);
        }
        self.centered(
            &mut out,
            [1664, 248, 1067],
            "Drag to look around",
            11,
            Ink::Subtle,
        );
        if let Some(result) = s.hud.result {
            Self::fill(&mut out, [0, 0, 1920, 960], [5, 12, 16, 150]);
            ui::card(&mut out, self.skin, 2, [340, 210, 1240, 550]);
            let color = if result.success {
                [159, 211, 111, 255]
            } else {
                [234, 168, 122, 255]
            };
            Self::fill(&mut out, [380, 250, 5, 440], color);
            self.text(
                &mut out,
                420,
                250,
                if result.success {
                    "Level complete!"
                } else {
                    "Try again"
                },
                48,
                Ink::Main,
            );
            self.text(
                &mut out,
                420,
                320,
                &font::fit(s.title, 24, 1080),
                24,
                Ink::Subtle,
            );
            self.text(
                &mut out,
                420,
                402,
                &format!(
                    "Saved {}%   /   Target {}%",
                    result.rescued_percent,
                    s.hud.target_percent.unwrap_or(0)
                ),
                32,
                Ink::Main,
            );
            self.text(
                &mut out,
                420,
                360,
                match result.reason {
                    lemmings_core::EndReason::TimeExpired => "Time is up",
                    lemmings_core::EndReason::PopulationExhausted => "All lemmings accounted for",
                },
                18,
                Ink::Subtle,
            );
            self.text(
                &mut out,
                420,
                450,
                &format!(
                    "Rescued {} / {}    Time left {}:{:02}",
                    result.rescued,
                    result.lemming_count,
                    s.hud.seconds_remaining / 60,
                    s.hud.seconds_remaining % 60
                ),
                20,
                Ink::Subtle,
            );
            let action = if s.hud.result_error {
                "Could not load the level. Press R to try again."
            } else if s.hud.replaying {
                "Replay complete"
            } else if result.success && s.hud.can_continue {
                "Enter or click to continue to the next level"
            } else if result.success {
                "All levels complete! Press R to play this level again"
            } else {
                "Enter or click to retry this level"
            };
            self.text(&mut out, 420, 500, action, 24, Ink::Accent);
            if !s.hud.replaying {
                self.text(&mut out, 420, 570, "R  Restart this level", 20, Ink::Subtle);
            }
            self.text(&mut out, 420, 610, "Esc  Exit", 20, Ink::Subtle);
        } else if s.hud.paused {
            ui::card(&mut out, self.skin, 2, [24, 24, 350, 98]);
            self.text(&mut out, 46, 36, "Paused", 32, Ink::Main);
            self.text(
                &mut out,
                46,
                81,
                "Space or Pause to resume",
                17,
                Ink::Subtle,
            );
        }
        Ok(out)
    }
}
fn skill_name(skill: crate::render::SelectedSkill) -> (&'static str, &'static str) {
    use crate::render::SelectedSkill::*;
    match skill {
        Climber => ("Climber", "C"),
        Floater => ("Floater", "F"),
        Bomber => ("Bomber", "E"),
        Blocker => ("Blocker", "K"),
        Builder => ("Builder", "U"),
        Basher => ("Basher", "B"),
        Miner => ("Miner", "M"),
        Digger => ("Digger", "D"),
    }
}

fn actor_slot(action: MotionAction, direction: MotionDirection) -> Option<u8> {
    let kind = match action {
        MotionAction::Walking => LemmingSpriteKind::Walking,
        MotionAction::Jumping => LemmingSpriteKind::Jumping,
        MotionAction::Falling => LemmingSpriteKind::Falling,
        MotionAction::Splatting => LemmingSpriteKind::Splatting,
        MotionAction::Digging => LemmingSpriteKind::Digging,
        MotionAction::Bashing => LemmingSpriteKind::Bashing,
        MotionAction::Mining => LemmingSpriteKind::Mining,
        MotionAction::Building => LemmingSpriteKind::Building,
        MotionAction::Shrugging => LemmingSpriteKind::Shrugging,
        MotionAction::Blocking => LemmingSpriteKind::Blocking,
        MotionAction::Climbing => LemmingSpriteKind::Climbing,
        MotionAction::Floating => LemmingSpriteKind::Umbrella,
        MotionAction::Drowning => LemmingSpriteKind::Drowning,
        MotionAction::Vaporizing => LemmingSpriteKind::Vaporizing,
        MotionAction::Exiting => LemmingSpriteKind::Exiting,
        MotionAction::Hoisting => LemmingSpriteKind::Hoisting,
        MotionAction::OhNo => LemmingSpriteKind::OhNo,
        MotionAction::Exploding => LemmingSpriteKind::Explosion,
    };
    let d = if direction == MotionDirection::Left {
        SpriteDirection::Left
    } else {
        SpriteDirection::Right
    };
    (0..28).find_map(|slot| {
        let info = LemmingAnimationInfo::for_slot(slot)?;
        (info.kind == kind && (info.direction.is_none() || info.direction == Some(d)))
            .then_some(slot as u8)
    })
}

#[cfg(test)]
mod tests {
    fn ui_text(renderer: &Renderer, draws: &[Draw]) -> String {
        draws
            .iter()
            .filter_map(|draw| match draw {
                Draw::Blit {
                    texture, source, ..
                } if (renderer.font..renderer.font + 3).contains(texture) => {
                    char::from_u32(32 + source[0] / 64 + source[1] / 64 * 16)
                }
                _ => None,
            })
            .collect()
    }

    #[test]
    fn hd_ui_uses_cached_smooth_glyphs_and_keeps_live_state_and_masks_read_only() {
        let mut run =
            LevelRun::new(Assets::synthetic_builder(), 1, Some(2), Pacing::Reference).unwrap();
        let mut renderer = Renderer::fallback();
        renderer.reset(&run.assets).unwrap();
        for t in &renderer.textures[renderer.font..renderer.font + 3] {
            assert!(t.rgba.chunks_exact(4).any(|p| p[3] > 0 && p[3] < 255));
        }
        let revisions: Vec<_> = renderer.textures[renderer.font..renderer.font + 3]
            .iter()
            .map(|t| t.revision)
            .collect();
        let terrain = run.session.terrain().to_vec();
        let objects = run.session.object_map_data().to_vec();
        for skill in panel::SKILLS {
            run.view.selected_skill = skill;
            let before = crate::diagnostics::state(&run.session);
            let audio = run.sound.diagnostic_json();
            let draws = renderer.frame(&run).unwrap();
            let text = ui_text(&renderer, &draws);
            assert!(text.contains(skill_name(skill).0));
            assert!(text.contains("Released0/"));
            assert!(!text.contains("HDPREVIEW"));
            assert_eq!(before, crate::diagnostics::state(&run.session));
            assert_eq!(audio, run.sound.diagnostic_json());
            assert_eq!(terrain, run.session.terrain());
            assert_eq!(objects, run.session.object_map_data());
            assert_eq!(
                revisions,
                renderer.textures[renderer.font..renderer.font + 3]
                    .iter()
                    .map(|t| t.revision)
                    .collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn result_text_respects_retry_terminal_and_failed_load_instead_of_promising_an_unavailable_next()
     {
        for (kind, next, error, replaying, expected, forbidden) in [
            (
                "win",
                Some(2),
                false,
                false,
                "Enterorclicktocontinuetothenextlevel",
                "Alllevelscomplete",
            ),
            (
                "win",
                None,
                false,
                false,
                "PressRtoplaythislevelagain",
                "Enterorclick",
            ),
            (
                "loss",
                Some(2),
                false,
                false,
                "Enterorclicktoretrythislevel",
                "nextlevel",
            ),
            (
                "win",
                Some(2),
                true,
                false,
                "Couldnotloadthelevel",
                "nextlevel",
            ),
            (
                "win",
                Some(2),
                false,
                true,
                "Replaycomplete",
                "Enterorclick",
            ),
        ] {
            let mut run = LevelRun::new(
                Assets::synthetic_completion(kind),
                1,
                next,
                Pacing::Reference,
            )
            .unwrap();
            for _ in 0..800 {
                if run.session.result().is_some() {
                    break;
                }
                run.tick().unwrap();
            }
            assert!(run.session.result().is_some());
            run.view.result_error = error;
            run.view.replaying = replaying;
            let before = crate::diagnostics::state(&run.session);
            let audio = run.sound.diagnostic_json();
            let mut renderer = Renderer::fallback();
            renderer.reset(&run.assets).unwrap();
            let draws = renderer.frame(&run).unwrap();
            let text = ui_text(&renderer, &draws);
            assert!(text.contains(expected), "{text}");
            assert!(!text.contains(forbidden), "{text}");
            assert!(text.contains("Timeleft"));
            assert!(text.contains("Rescued"));
            assert_eq!(before, crate::diagnostics::state(&run.session));
            assert_eq!(audio, run.sound.diagnostic_json());
        }
    }
    use super::*;
    use crate::pacing::Pacing;
    use lemmings_resources::ObjectFlags;

    #[test]
    fn all_portal_frames_follow_disk_phase_without_changing_state_or_sound() {
        let mut renderer = Renderer::fallback();
        for set in 0..5 {
            for id in 0..2 {
                for frame in 0..super::super::assets::object_frame_count(set, id).unwrap() {
                    let atlas = renderer.textures.len() as u16;
                    renderer.textures.push(Texture::new(
                        1,
                        1,
                        vec![set * 40, id * 60, frame as u8 * 20, 255],
                    ));
                    renderer.frames.push(AssetFrame {
                        key: AssetKey::Object { set, id, frame },
                        atlas,
                        rect: [0, 0, 1, 1],
                        display: [12, 6],
                        anchor: [0, 0],
                    });
                }
            }
        }
        renderer.base = renderer.textures.len();
        let mut seen = std::collections::BTreeSet::new();
        for set in 0..5 {
            let mut run = run();
            renderer.reset(&run.assets).unwrap();
            for id in 0..2 {
                object(&mut renderer, id, false);
                let o = renderer.objects.last_mut().unwrap();
                o.set = set;
                o.placement.flags.no_overwrite = false;
                o.meta.first_frame = if id == 1 { 1 } else { 0 };
                o.meta.frame_count = if id == 1 { 10 } else { 1 };
                o.frames = vec![vec![255, 0, 0, 255, 0, 255, 0, 255]; o.meta.frame_count as usize];
            }
            for _ in 0..60 {
                let state = crate::diagnostics::state(&run.session);
                let sound = run.sound.diagnostic_json();
                renderer.frame(&run).unwrap();
                assert!(renderer.source_portals.is_empty());
                assert_eq!(renderer.hd_portals.len(), 2);
                for &(theme, id, frame) in &renderer.hd_portals {
                    let expected = object_frame(
                        id == 1,
                        false,
                        usize::from(id == 1),
                        if id == 1 { 10 } else { 1 },
                        run.session.scheduler.step_state().iteration,
                    );
                    assert_eq!(frame as usize, expected);
                    seen.insert((theme, id, frame));
                    let o = renderer
                        .objects
                        .iter()
                        .find(|o| o.placement.id == id)
                        .unwrap();
                    assert_eq!(
                        &renderer.textures[o.texture].rgba[..4],
                        &[theme * 40, id * 60, frame as u8 * 20, 255]
                    );
                }
                assert_eq!(state, crate::diagnostics::state(&run.session));
                assert_eq!(sound, run.sound.diagnostic_json());
                run.tick().unwrap();
            }
        }
        assert_eq!(seen.len(), 55);
        assert!(renderer.info().contains("\"portal_frames_available\":55"));
        let run = run();
        let mut legacy = Renderer::fallback();
        legacy.reset(&run.assets).unwrap();
        object(&mut legacy, 1, false);
        legacy.objects[0].set = 4;
        legacy.frame(&run).unwrap();
        assert_eq!(legacy.source_portals, vec![(4, 1, 0)]);
        assert!(legacy.hd_portals.is_empty());
    }

    #[test]
    fn exit_flames_follow_six_frame_loop_without_mutating_state_or_audio() {
        let mut renderer = Renderer::fallback();
        for set in 0..5 {
            for id in 0..16 {
                if let Some(count) = super::super::assets::exit_effect_frame_count(set, id) {
                    for frame in 0..count {
                        let atlas = renderer.textures.len() as u16;
                        renderer.textures.push(Texture::new(
                            1,
                            1,
                            vec![set * 40, id * 20, frame as u8 * 18, 255],
                        ));
                        renderer.frames.push(AssetFrame {
                            key: AssetKey::Object { set, id, frame },
                            atlas,
                            rect: [0, 0, 1, 1],
                            display: [12, 6],
                            anchor: [0, 0],
                        });
                    }
                }
            }
        }
        renderer.base = renderer.textures.len();
        let mut seen = std::collections::BTreeSet::new();
        for set in 0..5 {
            let mut run = run();
            renderer.reset(&run.assets).unwrap();
            {
                let id = match set {
                    0 => 7,
                    4 => 8,
                    _ => 6,
                };
                object(&mut renderer, id, false);
                let o = renderer.objects.last_mut().unwrap();
                o.set = set;
                o.placement.flags.no_overwrite = false;
                o.meta.animation_flags = 2;
                o.meta.frame_count = 6;
                o.frames = vec![vec![255, 0, 0, 255, 0, 255, 0, 255]; 6];
            }
            for _ in 0..12 {
                let state = crate::diagnostics::state(&run.session);
                let sound = run.sound.diagnostic_json();
                renderer.frame(&run).unwrap();
                assert!(renderer.source_exit_effects.is_empty());
                assert_eq!(renderer.hd_exit_effects.len(), 1);
                for &(theme, id, frame) in &renderer.hd_exit_effects {
                    assert_eq!(
                        u64::from(frame),
                        (run.session.scheduler.step_state().iteration + 2) % 6
                    );
                    let o = renderer
                        .objects
                        .iter()
                        .find(|o| o.placement.id == id)
                        .unwrap();
                    assert_eq!(
                        &renderer.textures[o.texture].rgba[..4],
                        &[theme * 40, id * 20, frame as u8 * 18, 255]
                    );
                    seen.insert((theme, id, frame));
                }
                assert_eq!(state, crate::diagnostics::state(&run.session));
                assert_eq!(sound, run.sound.diagnostic_json());
                run.tick().unwrap();
            }
        }
        assert_eq!(seen.len(), 30);
        assert!(
            renderer
                .info()
                .contains("\"exit_effect_frames_available\":30")
        );
        let run = run();
        let mut legacy = Renderer::fallback();
        legacy.reset(&run.assets).unwrap();
        object(&mut legacy, 8, false);
        legacy.frame(&run).unwrap();
        assert_eq!(legacy.source_exit_effects, vec![(4, 8, 0)]);
        assert!(legacy.hd_exit_effects.is_empty());
    }

    #[test]
    fn hazards_follow_all_84_disk_frames_without_mutating_state_or_audio() {
        let mut renderer = Renderer::fallback();
        for set in 0..5 {
            for id in 0..16 {
                if let Some(count) = super::super::assets::hazard_frame_count(set, id) {
                    for frame in 0..count {
                        let atlas = renderer.textures.len() as u16;
                        renderer.textures.push(Texture::new(
                            1,
                            1,
                            vec![set * 40, id * 20, frame as u8 * 13, 255],
                        ));
                        renderer.frames.push(AssetFrame {
                            key: AssetKey::Object { set, id, frame },
                            atlas,
                            rect: [0, 0, 1, 1],
                            display: [12, 6],
                            anchor: [0, 0],
                        });
                    }
                }
            }
        }
        renderer.base = renderer.textures.len();
        let mut seen = std::collections::BTreeSet::new();
        for (set, id, count) in [
            (0, 5, 8),
            (1, 5, 8),
            (1, 7, 8),
            (1, 8, 10),
            (1, 10, 10),
            (2, 5, 8),
            (2, 9, 16),
            (3, 5, 8),
            (4, 6, 8),
        ] {
            let mut run = run();
            renderer.reset(&run.assets).unwrap();
            {
                object(&mut renderer, id, false);
                let o = renderer.objects.last_mut().unwrap();
                o.set = set;
                o.placement.flags.no_overwrite = false;
                o.meta.animation_flags = 2;
                o.meta.frame_count = count;
                o.meta.trigger_effect = if id == 5 || set == 4 { 5 } else { 6 };
                o.frames = vec![vec![255, 0, 0, 255, 0, 255, 0, 255]; count as usize];
            }
            for _ in 0..count * 2 {
                let state = crate::diagnostics::state(&run.session);
                let sound = run.sound.diagnostic_json();
                renderer.frame(&run).unwrap();
                assert!(renderer.source_hazards.is_empty());
                assert_eq!(renderer.hd_hazards.len(), 1);
                for &(theme, id, frame) in &renderer.hd_hazards {
                    assert_eq!(
                        u64::from(frame),
                        (run.session.scheduler.step_state().iteration + 2) % u64::from(count)
                    );
                    let o = renderer
                        .objects
                        .iter()
                        .find(|o| o.placement.id == id)
                        .unwrap();
                    assert_eq!(
                        &renderer.textures[o.texture].rgba[..4],
                        &[theme * 40, id * 20, frame as u8 * 13, 255]
                    );
                    seen.insert((theme, id, frame));
                }
                assert_eq!(state, crate::diagnostics::state(&run.session));
                assert_eq!(sound, run.sound.diagnostic_json());
                run.tick().unwrap();
            }
        }
        assert_eq!(seen.len(), 84);
        assert!(renderer.info().contains("\"hazard_frames_available\":84"));
        let run = run();
        let mut legacy = Renderer::fallback();
        legacy.reset(&run.assets).unwrap();
        object(&mut legacy, 6, false);
        legacy.frame(&run).unwrap();
        assert_eq!(legacy.source_hazards, vec![(4, 6, 0)]);
        assert!(legacy.hd_hazards.is_empty());
    }

    #[test]
    fn flags_and_arrows_follow_all_212_disk_frames_without_mutating_masks_state_or_audio() {
        let mut renderer = Renderer::fallback();
        for set in 0..5 {
            for id in 0..16 {
                if let Some(count) = super::super::assets::decor_frame_count(set, id) {
                    for frame in 0..count {
                        let atlas = renderer.textures.len() as u16;
                        renderer.textures.push(Texture::new(
                            1,
                            1,
                            vec![set * 40, id * 20, frame as u8 * 13, 255],
                        ));
                        renderer.frames.push(AssetFrame {
                            key: AssetKey::Object { set, id, frame },
                            atlas,
                            rect: [0, 0, 1, 1],
                            display: [12, 6],
                            anchor: [0, 0],
                        });
                    }
                }
            }
        }
        renderer.base = renderer.textures.len();
        let mut seen = std::collections::BTreeSet::new();
        for (set, id, count) in [
            (0, 2, 14),
            (0, 9, 14),
            (1, 2, 14),
            (1, 9, 14),
            (2, 2, 14),
            (2, 7, 14),
            (3, 2, 14),
            (3, 7, 14),
            (4, 2, 14),
            (4, 3, 14),
            (0, 3, 7),
            (0, 4, 7),
            (1, 3, 8),
            (1, 4, 8),
            (2, 3, 7),
            (2, 4, 7),
            (3, 3, 7),
            (3, 4, 7),
            (4, 4, 7),
            (4, 5, 7),
        ] {
            let mut run = run();
            renderer.reset(&run.assets).unwrap();
            {
                object(&mut renderer, id, false);
                let o = renderer.objects.last_mut().unwrap();
                o.set = set;
                o.placement.flags.no_overwrite = false;
                o.meta.animation_flags = 2;
                o.meta.frame_count = count;
                o.meta.trigger_effect = if count == 14 {
                    0
                } else if id == 3 || set == 4 && id == 4 {
                    7
                } else {
                    8
                };
                o.frames = vec![vec![255, 0, 0, 255, 0, 255, 0, 255]; count as usize];
            }
            let terrain = run.session.terrain().to_vec();
            let bricks = run.session.brick_mask().to_vec();
            let objects = run.session.object_map_data().to_vec();
            let one_way = (
                run.assets.one_way_left.clone(),
                run.assets.one_way_right.clone(),
            );
            for _ in 0..count * 2 {
                let state = crate::diagnostics::state(&run.session);
                let sound = run.sound.diagnostic_json();
                renderer.frame(&run).unwrap();
                assert!(renderer.source_decor.is_empty());
                assert_eq!(renderer.hd_decor.len(), 1);
                for &(theme, id, frame) in &renderer.hd_decor {
                    assert_eq!(
                        u64::from(frame),
                        (run.session.scheduler.step_state().iteration + 2) % u64::from(count)
                    );
                    let o = renderer
                        .objects
                        .iter()
                        .find(|o| o.placement.id == id)
                        .unwrap();
                    assert_eq!(
                        &renderer.textures[o.texture].rgba[..4],
                        &[theme * 40, id * 20, frame as u8 * 13, 255]
                    );
                    seen.insert((theme, id, frame));
                }
                assert_eq!(state, crate::diagnostics::state(&run.session));
                assert_eq!(sound, run.sound.diagnostic_json());
                assert_eq!(run.session.terrain(), terrain);
                assert_eq!(run.session.brick_mask(), bricks);
                assert_eq!(run.session.object_map_data(), objects);
                assert_eq!(
                    (&run.assets.one_way_left, &run.assets.one_way_right),
                    (&one_way.0, &one_way.1)
                );
                run.tick().unwrap();
            }
        }
        assert_eq!(seen.len(), 212);
    }

    #[test]
    fn trap_art_tracks_live_trigger_frames_reuse_and_pause_without_mutating_game() {
        let mut renderer = Renderer::fallback();
        for set in 0..5 {
            for id in 0..16 {
                if let Some(count) = super::super::assets::trap_frame_count(set, id) {
                    for frame in 0..count {
                        let atlas = renderer.textures.len() as u16;
                        renderer.textures.push(Texture::new(
                            1,
                            1,
                            vec![set * 40, id * 20, frame as u8 * 6, 255],
                        ));
                        renderer.frames.push(AssetFrame {
                            key: AssetKey::Object { set, id, frame },
                            atlas,
                            rect: [0, 0, 1, 1],
                            display: [12, 6],
                            anchor: [0, 0],
                        });
                    }
                }
            }
        }
        renderer.base = renderer.textures.len();
        let mut seen = std::collections::BTreeSet::new();
        for (set, id, count) in [
            (0, 6, 15),
            (0, 8, 17),
            (0, 10, 12),
            (2, 8, 15),
            (3, 8, 37),
            (3, 9, 7),
            (3, 10, 7),
            (4, 7, 25),
            (4, 9, 8),
            (4, 10, 16),
        ] {
            let mut assets = Assets::synthetic_hazards(0, false);
            assets.traps[0] = Some(lemmings_core::TrapState::new(count, 0).unwrap());
            let mut run = LevelRun::new(assets, 1, None, Pacing::Dos).unwrap();
            renderer.reset(&run.assets).unwrap();
            let target = object(&mut renderer, id, false);
            let o = renderer.objects.last_mut().unwrap();
            o.set = set;
            o.placement.flags.no_overwrite = false;
            o.meta.trigger_effect = 4;
            o.meta.frame_count = count;
            o.meta.animation_flags = 1;
            o.frames = vec![vec![255, 0, 0, 255, 0, 255, 0, 255]; count as usize];
            let mut paused = false;
            let mut started = false;
            let mut reused = false;
            let mut cycles = 0;
            let mut was_triggered = false;
            for _ in 0..300 {
                let trap = run.session.traps()[0].unwrap();
                if trap.triggered() && !was_triggered {
                    if started {
                        reused = true;
                    }
                    started = true;
                }
                if !trap.triggered() && was_triggered {
                    cycles += 1;
                }
                was_triggered = trap.triggered();
                let state = crate::diagnostics::state(&run.session);
                let sound = run.sound.diagnostic_json();
                renderer.frame(&run).unwrap();
                assert!(renderer.source_traps.is_empty());
                assert_eq!(renderer.hd_traps, vec![(set, id, u16::from(trap.frame()))]);
                assert_eq!(
                    &renderer.textures[target].rgba[..4],
                    &[set * 40, id * 20, trap.frame() * 6, 255]
                );
                assert_eq!(state, crate::diagnostics::state(&run.session));
                assert_eq!(sound, run.sound.diagnostic_json());
                seen.insert((set, id, trap.frame()));
                if trap.triggered() && !paused {
                    run.session
                        .queue(lemmings_core::InputCommand::Pause)
                        .unwrap();
                    run.tick().unwrap();
                    let frozen = run.session.traps()[0];
                    for _ in 0..3 {
                        run.tick().unwrap();
                        renderer.frame(&run).unwrap();
                        assert_eq!(run.session.traps()[0], frozen);
                    }
                    run.session
                        .queue(lemmings_core::InputCommand::Resume)
                        .unwrap();
                    run.tick().unwrap();
                    paused = true;
                }
                run.tick().unwrap();
            }
            assert!(started && reused && cycles > 0 && paused, "{set}/{id}");
        }
        assert_eq!(seen.len(), 159);
        assert!(renderer.info().contains("\"trap_frames_available\":159"));
    }

    fn run() -> LevelRun {
        let mut assets = Assets::synthetic_builder();
        assets.terrain.fill(0);
        assets.terrain[10 * assets.width + 10] = 1;
        LevelRun::new(assets, 1, None, Pacing::Reference).unwrap()
    }
    fn object(renderer: &mut Renderer, id: u8, on_terrain: bool) -> usize {
        let texture = renderer.textures.len();
        renderer.textures.push(Texture::new(1, 1, vec![0; 4]));
        renderer.objects.push(Object {
            placement: ObjectPlacement {
                slot: id as usize,
                x: 10,
                y: 10,
                id,
                flags: ObjectFlags {
                    upside_down: false,
                    no_overwrite: true,
                    only_on_terrain: on_terrain,
                },
            },
            meta: GroundObject {
                slot: id as usize,
                width: 2,
                height: 1,
                frame_count: 1,
                ..Default::default()
            },
            set: 4,
            frames: vec![vec![255, 0, 0, 255, 0, 255, 0, 255]],
            texture,
            last: None,
        });
        texture
    }
    #[test]
    fn logical_terrain_and_prior_objects_clip_hd_pixels_in_original_draw_order() {
        let run = run();
        let mut r = Renderer::fallback();
        r.reset(&run.assets).unwrap();
        let first = object(&mut r, 2, false);
        let second = object(&mut r, 3, false);
        r.objects(&run, 0, &mut Vec::new()).unwrap();
        assert_eq!(r.textures[first].rgba[3], 0);
        assert_eq!(&r.textures[first].rgba[24..28], &[0, 255, 0, 255]);
        assert!(r.textures[second].rgba.iter().all(|&v| v == 0));
        let rev = r.textures[first].revision;
        r.objects(&run, 0, &mut Vec::new()).unwrap();
        assert_eq!(r.textures[first].revision, rev);
        r.objects.clear();
        let on_terrain = object(&mut r, 2, true);
        r.objects(&run, 0, &mut Vec::new()).unwrap();
        assert_eq!(&r.textures[on_terrain].rgba[..4], &[255, 0, 0, 255]);
        assert_eq!(r.textures[on_terrain].rgba[27], 0);
    }
    #[test]
    fn a_single_visible_mask_edit_rebuilds_one_chunk_and_keeps_other_textures() {
        let run = run();
        let mut r = Renderer::fallback();
        r.reset(&run.assets).unwrap();
        r.frame(&run).unwrap();
        let before = r.terrain_rebuilds();
        let other = r.textures[r.chunk_base + 1].rgba.clone();
        let mut mask = run.session.render_terrain().to_vec();
        mask[10 * run.assets.width + 10] = 0;
        r.terrain(
            &mask,
            run.session.render_bricks(),
            &run.assets.steel,
            0,
            &mut Vec::new(),
        )
        .unwrap();
        assert_eq!(r.terrain_rebuilds(), before + 1);
        assert_eq!(r.textures[r.chunk_base + 1].rgba, other);
        assert!(r.textures[r.chunk_base].rgba.iter().all(|&v| v == 0));
    }
    #[test]
    fn draw_wire_has_fixed_records_and_keeps_clip_and_alpha_explicit() {
        assert_eq!(
            Draw::GameClip(true).wire(),
            [2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(
            Draw::Fill {
                rect: [-6, 2, 8, 9],
                color: [10, 20, 30, 128]
            }
            .wire(),
            [0, -6, 2, 8, 9, 10, 20, 30, 128, 0, 0, 0]
        );
        assert_eq!(
            Draw::Blit {
                texture: 7,
                source: [1, 2, 3, 4],
                dest: [-5, 6, 7, 8]
            }
            .wire(),
            [1, 7, 1, 2, 3, 4, -5, 6, 7, 8, 0, 0]
        );
    }

    #[test]
    fn all_materials_keep_masks_bricks_steel_one_way_and_terrain_edits() {
        let mut run = run();
        let at = 10 * run.assets.width + 10;
        run.assets.one_way_left[at] = 1;
        let mut r = Renderer::fallback();
        for set in 0..5 {
            r.textures
                .push(Texture::new(1, 1, vec![set * 40, 20, 30, 255]));
            r.frames.push(AssetFrame {
                key: AssetKey::material(set).unwrap(),
                atlas: set.into(),
                rect: [0, 0, 1, 1],
                display: [256, 256],
                anchor: [0, 0],
            });
        }
        r.base = r.textures.len();
        r.reset(&run.assets).unwrap();
        assert_eq!(r.material, Some(0));
        let state = crate::diagnostics::state(&run.session);
        let sound = run.sound.diagnostic_json();
        let pixel = (60 * 192 + 60) * 4;
        for set in 0..5 {
            r.select_material(set, false);
            r.dirty.fill(true);
            r.frame(&run).unwrap();
            assert_eq!(
                &r.textures[r.chunk_base].rgba[pixel..pixel + 4],
                &[set * 40, 20, 30, 255]
            );
            assert_eq!(r.textures[r.chunk_base].rgba[3], 0);
            assert_eq!(state, crate::diagnostics::state(&run.session));
            assert_eq!(sound, run.sound.diagnostic_json());
            let mut removed = run.session.render_terrain().to_vec();
            removed[at] = 0;
            let rebuilds = r.terrain_rebuilds();
            r.terrain(
                &removed,
                run.session.render_bricks(),
                &run.assets.steel,
                0,
                &mut Vec::new(),
            )
            .unwrap();
            assert_eq!(r.terrain_rebuilds(), rebuilds + 1);
            assert!(r.textures[r.chunk_base].rgba.iter().all(|&v| v == 0));
        }
        r.select_material(4, true);
        assert_eq!(r.material, None);
        r.select_material(5, false);
        assert_eq!(r.material, None);
        r.select_material(4, false);
        r.frames.pop();
        r.select_material(4, false);
        assert_eq!(r.material, None);
        r.select_material(3, false);
        let mut steel = run.assets.steel.clone();
        steel[at] = 1;
        let mut bricks = run.session.render_bricks().to_vec();
        r.dirty.fill(true);
        r.terrain(
            run.session.render_terrain(),
            &bricks,
            &steel,
            0,
            &mut Vec::new(),
        )
        .unwrap();
        assert_eq!(
            &r.textures[r.chunk_base].rgba[pixel..pixel + 4],
            &[132, 150, 166, 255]
        );
        bricks[at] = 1;
        r.terrain(
            run.session.render_terrain(),
            &bricks,
            &steel,
            0,
            &mut Vec::new(),
        )
        .unwrap();
        assert_eq!(
            &r.textures[r.chunk_base].rgba[pixel..pixel + 4],
            &[118, 75, 32, 255]
        );
    }

    #[test]
    fn material_sampling_keeps_world_alignment_across_chunks_and_scroll() {
        let mut assets = Assets::synthetic_builder();
        assets.terrain.fill(1);
        let run = LevelRun::new(assets, 1, None, Pacing::Reference).unwrap();
        let mut r = Renderer::fallback();
        r.textures
            .push(Texture::new(2, 1, vec![10, 20, 30, 255, 40, 50, 60, 255]));
        r.frames.push(AssetFrame {
            key: AssetKey::Rock,
            atlas: 0,
            rect: [0, 0, 2, 1],
            display: [256, 256],
            anchor: [0, 0],
        });
        r.base = 1;
        r.reset(&run.assets).unwrap();
        r.frame(&run).unwrap();
        assert_eq!(
            &r.textures[r.chunk_base].rgba[191 * 4..192 * 4],
            &[40, 50, 60, 255]
        );
        assert_eq!(&r.textures[r.chunk_base + 1].rgba[..4], &[40, 50, 60, 255]);
        let texture = r.textures[r.chunk_base + 1].rgba.clone();
        let rebuilds = r.terrain_rebuilds();
        r.terrain(
            run.session.render_terrain(),
            run.session.render_bricks(),
            &run.assets.steel,
            32,
            &mut Vec::new(),
        )
        .unwrap();
        assert_eq!(texture, r.textures[r.chunk_base + 1].rgba);
        assert_eq!(r.terrain_rebuilds(), rebuilds + r.chunks_y as u64);
    }
    #[test]
    fn special_panorama_is_world_aligned_masked_and_resets_to_original() {
        let mut assets = Assets::synthetic_builder();
        assets.width = 1600;
        assets.terrain = vec![1; 1600 * 160];
        assets.steel = vec![0; 1600 * 160];
        assets.one_way_left = vec![0; 1600 * 160];
        assets.one_way_right = vec![0; 1600 * 160];
        let run = LevelRun::new(assets, 1, None, Pacing::Reference).unwrap();
        let mut r = Renderer::fallback();
        for set in 0..4 {
            r.textures.push(Texture::new(
                2,
                1,
                vec![20 + set, 30, 40, 255, 80 + set, 90, 100, 255],
            ));
            r.frames.push(AssetFrame {
                key: AssetKey::Special { set },
                atlas: set.into(),
                rect: [0, 0, 2, 1],
                display: [960, 160],
                anchor: [0, 0],
            });
        }
        r.base = 4;
        r.reset(&run.assets).unwrap();
        let state = crate::diagnostics::state(&run.session);
        let sound = run.sound.diagnostic_json();
        let sample = |r: &Renderer, x: usize| {
            let id = x / 32;
            let at = (x % 32) * 6 * 4;
            r.textures[r.chunk_base + id].rgba[at..at + 4].to_vec()
        };
        for set in 0..4 {
            r.select_special(Some(set));
            r.dirty.fill(true);
            for camera in [288, 768, 960] {
                r.terrain(
                    run.session.render_terrain(),
                    run.session.render_bricks(),
                    &run.assets.steel,
                    camera,
                    &mut Vec::new(),
                )
                .unwrap();
            }
            assert_eq!(sample(&r, 304), vec![20 + set, 30, 40, 255]);
            assert_eq!(sample(&r, 783), vec![20 + set, 30, 40, 255]);
            assert_eq!(sample(&r, 784), vec![80 + set, 90, 100, 255]);
            assert_eq!(sample(&r, 1263), vec![80 + set, 90, 100, 255]);
            assert_ne!(sample(&r, 1264), vec![80 + set, 90, 100, 255]);
            let mut mask = run.session.render_terrain().to_vec();
            mask[304] = 0;
            r.terrain(
                &mask,
                run.session.render_bricks(),
                &run.assets.steel,
                288,
                &mut Vec::new(),
            )
            .unwrap();
            assert_eq!(sample(&r, 304), vec![0; 4]);
            assert_eq!(state, crate::diagnostics::state(&run.session));
            assert_eq!(sound, run.sound.diagnostic_json());
        }
        r.select_special(Some(4));
        assert_eq!(r.special, None);
        r.select_special(Some(2));
        r.reset(&run.assets).unwrap();
        assert_eq!(r.special, None);
        let mut fallback = Renderer::fallback();
        fallback.select_special(Some(0));
        assert_eq!(fallback.special, None);
    }
}
