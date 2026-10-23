//! Shared HD draw list and revisioned texture cache. Hosts upload changed textures
//! and composite them; this module never performs I/O, advances time or writes a run.
use super::{
    Control,
    assets::{AssetFrame, AssetKey, AssetPack},
    snapshot,
};
use crate::{Assets, LevelRun, assets::object_frame, panel, render::glyph};
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
    material: bool,
    mini: usize,
    terrain_rebuilds: u64,
    missing: Vec<(u8, u16)>,
    drawn_hd: Vec<(u8, u16)>,
    epoch: u64,
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
            material: true,
            mini: 0,
            terrain_rebuilds: 0,
            missing: Vec::new(),
            drawn_hd: Vec::new(),
            epoch: 0,
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
        format!(
            "{{\"version\":1,\"width\":1920,\"height\":1080,\"terrain_rebuilds\":{},\"source_terrain\":{},\"missing_actor_frames\":[{missing}],\"hd_actor_frames\":[{drawn}]}}",
            self.terrain_rebuilds, !self.material
        )
    }
    pub fn reset(&mut self, assets: &Assets) -> Result<(), Box<dyn Error>> {
        if assets.width == 0 || assets.width > 4096 || assets.height != 160 {
            return Err("unsupported HD world dimensions".into());
        }
        self.textures.truncate(self.base);
        self.fallback_frames.clear();
        self.objects.clear();
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
        self.material = true;
        if let Some((set, level, ground, images)) = assets.hd_art()? {
            self.material = set == 0
                && level.header().special_graphic_set().is_none()
                && !assets
                    .one_way_left
                    .iter()
                    .chain(&assets.one_way_right)
                    .any(|&v| v != 0);
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
    fn text(out: &mut Vec<Draw>, x: i32, y: i32, value: &str, scale: i32, color: [u8; 4]) {
        for (i, c) in value.chars().enumerate() {
            for (dy, bits) in glyph(c.to_ascii_uppercase()).iter().enumerate() {
                for dx in 0..5 {
                    if bits & (1 << (4 - dx)) != 0 {
                        Self::fill(
                            out,
                            [
                                x + (i as i32 * 6 + dx) * scale,
                                y + dy as i32 * scale,
                                scale,
                                scale,
                            ],
                            color,
                        );
                    }
                }
            }
        }
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
        let rock = self
            .frames
            .iter()
            .find(|f| f.key == AssetKey::Rock)
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
                            } else if self.material {
                                if let Some(r) = rock {
                                    let source = &self.textures[r.atlas as usize];
                                    let sx = r.rect[0]
                                        + (((x * 6 + px) * r.rect[2] as usize / 256)
                                            % r.rect[2] as usize)
                                            as u32;
                                    let sy = r.rect[1]
                                        + (((y * 6 + py) * r.rect[3] as usize / 256)
                                            % r.rect[3] as usize)
                                            as u32;
                                    let at =
                                        (sy as usize * source.width as usize + sx as usize) * 4;
                                    [
                                        source.rgba[at],
                                        source.rgba[at + 1],
                                        source.rgba[at + 2],
                                        255,
                                    ]
                                } else {
                                    [130, 85, 45, 255]
                                }
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
                Self::text(
                    &mut out,
                    actor.foot.0 as i32,
                    actor.foot.1 as i32 - frame.anchor[1] as i32 - 32,
                    &digit.to_string(),
                    4,
                    [255, 239, 153, 255],
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
        Self::fill(&mut out, [0, 960, 1920, 120], [28, 47, 53, 255]);
        let white = [222, 237, 226, 255];
        Self::text(
            &mut out,
            16,
            976,
            &format!("{} {}", s.level.difficulty().name(), s.level.number()),
            2,
            white,
        );
        Self::text(
            &mut out,
            16,
            998,
            &format!("OUT {} SAVED {}%", s.hud.alive, s.hud.rescued_percent),
            2,
            white,
        );
        Self::text(
            &mut out,
            16,
            1020,
            &format!(
                "TIME {}:{:02} NEED {}%",
                s.hud.seconds_remaining / 60,
                s.hud.seconds_remaining % 60,
                s.hud.target_percent.unwrap_or(0)
            ),
            2,
            white,
        );
        Self::text(
            &mut out,
            16,
            1050,
            "HD PREVIEW - SOME ORIGINAL ART",
            1,
            [159, 187, 177, 255],
        );
        for region in super::control_regions() {
            let c = region.control;
            if c == Control::Minimap {
                continue;
            }
            let x = region.rect.x as i32;
            let w = region.rect.width as i32;
            let active = matches!(c,Control::Skill(skill) if skill==s.hud.selected)
                || (c == Control::Pause && s.hud.paused);
            Self::fill(
                &mut out,
                [x + 2, 964, w - 4, 112],
                if active {
                    [53, 104, 83, 255]
                } else {
                    [34, 61, 66, 255]
                },
            );
            if let Some(f) = self.frames.iter().find(|f| f.key == AssetKey::Icon(c)) {
                Self::blit(&mut out, f, x + w / 2, 1004);
            } else {
                let name = match c {
                    Control::Slower => "-",
                    Control::Faster => "+",
                    Control::Pause => "P",
                    Control::Nuke => "N",
                    Control::Restart => "R",
                    _ => "SKILL",
                };
                Self::text(&mut out, x + 8, 992, name, 2, white);
            }
            let value = match c {
                Control::Slower | Control::Faster => s.hud.release_rate.to_string(),
                Control::Skill(skill) => s.hud.skills
                    [panel::SKILLS.iter().position(|&v| v == skill).unwrap()]
                .to_string(),
                Control::Pause => if s.hud.paused { "PLAY" } else { "PAUSE" }.into(),
                Control::Nuke => "NUKE".into(),
                Control::Restart => "RETRY".into(),
                _ => String::new(),
            };
            Self::text(
                &mut out,
                x + (w - value.len() as i32 * 12) / 2,
                1044,
                &value,
                2,
                white,
            );
        }
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
        if let Some(result) = s.hud.result {
            Self::fill(&mut out, [300, 260, 1320, 400], [24, 43, 50, 245]);
            Self::text(
                &mut out,
                390,
                310,
                if result.success {
                    "LEVEL COMPLETE"
                } else {
                    "TRY AGAIN"
                },
                6,
                white,
            );
            Self::text(
                &mut out,
                390,
                400,
                &format!(
                    "SAVED {}%  NEED {}%",
                    result.rescued_percent,
                    s.hud.target_percent.unwrap_or(0)
                ),
                4,
                white,
            );
            Self::text(&mut out, 390, 480, "ENTER / CLICK: NEXT OR RETRY", 3, white);
            Self::text(&mut out, 390, 535, "R: RESTART   ESC: EXIT", 3, white);
        } else if s.hud.paused {
            Self::text(&mut out, 24, 24, "PAUSED", 4, white);
        }
        Ok(out)
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
    use super::*;
    use crate::pacing::Pacing;
    use lemmings_resources::ObjectFlags;

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
}
