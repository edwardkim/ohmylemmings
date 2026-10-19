use lemmings_core::{
    DOS_OBJECT_MAP_HEIGHT, DOS_OBJECT_MAP_ORIGIN, DOS_OBJECT_MAP_WIDTH, Difficulty, EntrancePoint,
    LevelId, OBJECT_NONE, ObjectMapMut, StartupConfig,
};
use lemmings_resources::*;
use std::{error::Error, fs, io, path::Path};

pub struct Assets {
    pub width: usize,
    pub height: usize,
    pub terrain: Vec<u8>,
    pub steel: Vec<u8>,
    pub diggers: u16,
    pub bashers: u16,
    pub miners: u16,
    pub builders: u16,
    pub blockers: u16,
    pub bombers: u16,
    pub climbers: u16,
    pub floaters: u16,
    pub explosion_mask: [u8; 44],
    pub countdown_masks: [u8; 80],
    pub particles: Option<Vec<u8>>,
    pub object_map: Vec<u8>,
    pub traps: lemmings_core::TrapStates,
    pub mine_masks: [u8; 104],
    pub bash_masks: [u8; 160],
    pub one_way_left: Vec<u8>,
    pub one_way_right: Vec<u8>,
    pub title: String,
    pub entrances: Vec<EntrancePoint>,
    pub config: StartupConfig,
    pub required: Option<u32>,
    pub screen_x: i32,
    pub sprites: Vec<u8>,
    pub palette: [[u8; 3]; 16],
    pub panel: crate::panel::Art,
    original: Option<Original>,
}
struct Original {
    graphics: u8,
    level: [u8; LEVEL_RECORD_SIZE],
    ground: Vec<u8>,
    terrain_images: Vec<u8>,
    object_images: Vec<u8>,
}
fn read(root: &Path, name: &str) -> io::Result<Vec<u8>> {
    let path = root.join(name);
    fs::read(&path).map_err(|e| io::Error::new(e.kind(), format!("{}: {e}", path.display())))
}
// Disk object ID 1 is the entrance: pinned Dos.Consts.pas DOS_OBJECT_ID_ENTRANCE.
pub fn object_frame(
    is_entrance: bool,
    continuous: bool,
    first: usize,
    count: usize,
    iteration: u64,
) -> usize {
    if is_entrance {
        if iteration < 35 {
            first
        } else {
            let frame = iteration - 35 + 2;
            if frame < count as u64 {
                frame as usize
            } else {
                0
            }
        }
    } else if continuous {
        // #30: DOS performs one initial object update. At matching F00 actor/
        // entrance images its startup counter is native iteration + 1. Thus the
        // visible continuous frame is first + iteration + 2 (entrances differ).
        (first + (iteration % count as u64) as usize + 2) % count
    } else {
        first
    }
}
impl Assets {
    pub fn load(root: &Path) -> Result<Self, Box<dyn Error>> {
        Self::load_level(root, 1)
    }
    pub fn load_level(root: &Path, number: u8) -> Result<Self, Box<dyn Error>> {
        let entry = CatalogEntry::for_level(LevelId::new(Difficulty::Fun, number)?);
        let level_bytes = read(root, &format!("LEVEL{:03}.DAT", entry.file_index()))?;
        let file = DosLevelFile::parse(entry.file_index(), &level_bytes)?;
        let mut record = [0; LEVEL_RECORD_SIZE];
        let odd = if entry.odd_index().is_some() {
            Some(read(root, "ODDTABLE.DAT")?)
        } else {
            None
        };
        let resolved = entry.load(
            &file,
            odd.as_deref().map(OddTable::parse).transpose()?,
            &mut record,
        )?;
        let header = resolved.header();
        if header.special_graphic_set().is_some() {
            return Err("desktop currently requires a standard graphics level".into());
        }
        let ground_bytes = read(root, &format!("GROUND{}O.DAT", header.graphics))?;
        let ground = GroundRecord::parse(&ground_bytes)?;
        let dat = read(root, &format!("VGAGR{}.DAT", header.graphics))?;
        let graphics = VgagrFile::parse(&dat)?;
        let [ts, os] = graphics.decoded_sizes();
        let mut terrain_images = vec![0; ts];
        let mut object_images = vec![0; os];
        graphics.decode_into(ground, &mut terrain_images, &mut object_images)?;
        let images = VgagrImages::new(ground, &terrain_images, &object_images)?;
        let mut canvas = vec![IndexedPixel::default(); LEVEL_TERRAIN_PIXEL_COUNT];
        let terrain = compose_terrain(
            resolved.base_record(),
            TerrainSource::Standard(TerrainGraphics::new(
                u8::try_from(header.graphics)?,
                images,
            )?),
            &mut canvas,
        )?;
        let mask = terrain
            .pixels()
            .iter()
            .map(|p| u8::from(p.opaque))
            .collect();
        let mut steel = vec![0; LEVEL_TERRAIN_PIXEL_COUNT];
        for area in resolved.base_record().steel() {
            for y in
                usize::from(area.y)..usize::from(area.y + area.height).min(LEVEL_TERRAIN_HEIGHT)
            {
                for x in i32::from(area.x).max(0)
                    ..(i32::from(area.x) + i32::from(area.width)).min(LEVEL_TERRAIN_WIDTH as i32)
                {
                    steel[y * LEVEL_TERRAIN_WIDTH + x as usize] = 1;
                }
            }
        }
        let object_map = build_object_map(resolved.base_record(), ground)?;
        let diggers = header.skills[7];
        let bashers = header.skills[5];
        let mut one_way_left = vec![0; LEVEL_TERRAIN_PIXEL_COUNT];
        let mut one_way_right = vec![0; LEVEL_TERRAIN_PIXEL_COUNT];
        // DOS rounds object origins down to the 4-pixel object-map grid;
        // later object trigger rectangles overwrite earlier steel/objects.
        for object in resolved.base_record().objects().filter(|o| o.slot < 16) {
            let meta = ground
                .object(usize::from(object.id))
                .ok_or("object metadata missing")?;
            let rect = meta.trigger_rect();
            let left = (object.x & !3) + rect.left;
            let top = (i32::from(object.y) & !3) + rect.top;
            for y in top.max(0)..(top + i32::from(rect.height)).min(LEVEL_TERRAIN_HEIGHT as i32) {
                for x in left.max(0)..(left + i32::from(rect.width)).min(LEVEL_TERRAIN_WIDTH as i32)
                {
                    let i = y as usize * LEVEL_TERRAIN_WIDTH + x as usize;
                    steel[i] = u8::from(meta.trigger_effect == 9);
                    one_way_left[i] = u8::from(meta.trigger_effect == 7);
                    one_way_right[i] = u8::from(meta.trigger_effect == 8);
                }
            }
        }
        let mut scene = vec![ScenePixel::default(); LEVEL_TERRAIN_PIXEL_COUNT];
        let scene = compose_objects(
            resolved.base_record(),
            ObjectGraphics::new(u8::try_from(header.graphics)?, images)?,
            terrain,
            ObjectFrameSelection::Initial,
            &mut scene,
        )?;
        let palette = scene.palette();
        let entrances: Vec<_> = resolved
            .base_record()
            .objects()
            .filter(|o| o.id == 1)
            .map(|o| EntrancePoint {
                x: o.x,
                y: i32::from(o.y),
            })
            .collect();
        let main = read(root, "MAIN.DAT")?;
        let mut sprites = vec![0; LEMMING_SPRITE_DATA_SIZE];
        MainDatFile::parse(&main)?.decode_sprites_into(&mut sprites)?;
        let mut masks = [0; MAIN_MASK_RECORD_SIZE];
        MainDatFile::parse(&main)?.decode_masks_into(&mut masks)?;
        let config = StartupConfig {
            time_limit_minutes: header.minutes,
            total_lemmings: u32::from(header.total),
            initial_release_rate: u8::try_from(header.release_rate)?,
            entrance_count: u8::try_from(entrances.len())?,
            pause_glitch: true,
        };
        lemmings_core::StartupScheduler::new(config)?;
        Ok(Self {
            required: Some(u32::from(header.required)),
            width: LEVEL_TERRAIN_WIDTH,
            height: LEVEL_TERRAIN_HEIGHT,
            terrain: mask,
            steel,
            diggers,
            bashers,
            miners: header.skills[6],
            builders: header.skills[4],
            blockers: header.skills[3],
            bombers: header.skills[2],
            climbers: header.skills[0],
            floaters: header.skills[1],
            explosion_mask: masks[264..308].try_into()?,
            countdown_masks: masks[308..388].try_into()?,
            particles: load_particles(root)?,
            traps: build_traps(resolved.base_record(), ground)?,
            object_map,
            mine_masks: masks[160..264].try_into()?,
            bash_masks: masks[..160].try_into()?,
            one_way_left,
            one_way_right,
            title: resolved.title()?.trim().to_uppercase(),
            entrances,
            config,
            screen_x: i32::from(header.screen_x),
            sprites,
            palette,
            panel: crate::panel::Art::load(&MainDatFile::parse(&main)?)?,
            original: Some(Original {
                graphics: u8::try_from(header.graphics)?,
                level: record,
                ground: ground_bytes,
                terrain_images,
                object_images,
            }),
        })
    }
    pub fn synthetic() -> Self {
        let (width, height) = (640, 160);
        let terrain = (0..width * height)
            .map(|i| u8::from(i / width >= 100 && i % width < 260))
            .collect();
        Self {
            width,
            height,
            terrain,
            steel: vec![0; width * height],
            diggers: 10,
            bashers: 10,
            miners: 10,
            builders: 10,
            blockers: 10,
            bombers: 10,
            climbers: 10,
            floaters: 10,
            traps: [None; 16],
            explosion_mask: [255; 44],
            countdown_masks: synthetic_countdown(),
            particles: None,
            object_map: vec![OBJECT_NONE; DOS_OBJECT_MAP_WIDTH * DOS_OBJECT_MAP_HEIGHT],
            mine_masks: [255; 104], // own full rectangular synthetic erase masks
            // Own synthetic masks (rectangular slices), never original bits.
            bash_masks: synthetic_bash_masks(),
            one_way_left: vec![0; width * height],
            one_way_right: vec![0; width * height],
            title: "JUST DIG!".into(),
            entrances: vec![EntrancePoint { x: 40, y: 40 }],
            required: None,
            config: StartupConfig {
                time_limit_minutes: 5,
                total_lemmings: 10,
                initial_release_rate: 50,
                entrance_count: 1,
                pause_glitch: true,
            },
            screen_x: 0,
            sprites: vec![255; LEMMING_SPRITE_DATA_SIZE],
            panel: crate::panel::Art::synthetic(),
            palette: [
                [0, 0, 0],
                [0, 220, 0],
                [0, 80, 240],
                [240, 240, 240],
                [0; 3],
                [0; 3],
                [0; 3],
                [220, 140, 70], // palette7: builder material, as in Styles.Base.pas
                [0; 3],
                [0; 3],
                [0; 3],
                [0; 3],
                [0; 3],
                [0; 3],
                [0; 3],
                [0; 3],
            ],
            original: None,
        }
    }
    pub fn synthetic_basher() -> Self {
        let mut scene = Self::synthetic();
        for y in 70..100 {
            scene.terrain[y * scene.width + 80..y * scene.width + 150].fill(1);
            scene.terrain[y * scene.width + 5..y * scene.width + 50].fill(1);
        }
        scene
    }
    pub fn synthetic_builder() -> Self {
        let mut scene = Self::synthetic();
        for y in 0..scene.height {
            for x in 0..scene.width {
                scene.terrain[y * scene.width + x] = u8::from(
                    (y >= 100 && x < 500) || ((60..100).contains(&y) && (160..170).contains(&x)),
                );
            }
        }
        scene
    }
    pub fn synthetic_climber() -> Self {
        let mut scene = Self::synthetic_builder();
        for y in 60..100 {
            for x in 20..30 {
                scene.terrain[y * scene.width + x] = 1;
            }
        }
        scene
    }
    pub fn synthetic_floater() -> Self {
        let mut scene = Self::synthetic();
        scene.entrances = vec![EntrancePoint { x: 40, y: 0 }];
        for y in 0..scene.height {
            for x in 0..scene.width {
                scene.terrain[y * scene.width + x] = u8::from(
                    (y >= 140 && x < 500)
                        || ((60..140).contains(&y) && x < 120)
                        || ((90..140).contains(&y) && (200..210).contains(&x)),
                );
            }
        }
        scene
    }
    pub fn synthetic_completion(kind: &str) -> Self {
        let mut scene = match kind {
            "win" | "mixed" => Self::synthetic_exit(false),
            "loss" => Self::synthetic_hazards(133, false),
            _ => Self::synthetic_builder(),
        };
        scene.config.total_lemmings = if matches!(kind, "clock" | "mixed") {
            80
        } else if kind == "particles" {
            1
        } else {
            3
        };
        scene.config.time_limit_minutes = if kind == "zero" {
            0
        } else if kind == "clock" {
            1
        } else {
            5
        };
        scene.required = Some(if matches!(kind, "clock" | "mixed") {
            40
        } else if kind == "particles" {
            1
        } else {
            2
        });
        if kind == "mixed" {
            scene.entrances.push(EntrancePoint { x: 300, y: 40 });
            scene.config.entrance_count = 2;
            scene.config.initial_release_rate = 99;
        }
        scene.title = format!("SYNTHETIC {}", kind.to_ascii_uppercase());
        scene
    }
    pub fn synthetic_exit(drop: bool) -> Self {
        let mut scene = Self::synthetic();
        let floor = if drop { 140 } else { 100 };
        if drop {
            scene.entrances = vec![EntrancePoint { x: 40, y: 0 }];
        }
        for y in 0..scene.height {
            for x in 0..scene.width {
                scene.terrain[y * scene.width + x] = u8::from(y >= floor && x < 500);
            }
        }
        let mut map = ObjectMapMut::new(
            DOS_OBJECT_MAP_WIDTH,
            DOS_OBJECT_MAP_HEIGHT,
            -16,
            -16,
            &mut scene.object_map,
        )
        .unwrap();
        for y in if drop { 60..144 } else { 96..104 } {
            for x in if drop { 64..72 } else { 96..104 } {
                map.write(x, y, lemmings_core::OBJECT_EXIT);
            }
        }
        scene.title = if drop {
            "Synthetic drop exit"
        } else {
            "Synthetic exit"
        }
        .into();
        scene
    }
    pub fn synthetic_hazards(kind: u8, drop: bool) -> Self {
        let mut scene = Self::synthetic_exit(drop);
        let late = matches!(kind, 143 | 144);
        let kind = if late { kind - 10 } else { kind };
        scene.object_map.fill(OBJECT_NONE);
        let code = if kind < 128 { 0 } else { kind };
        let mut map = ObjectMapMut::new(
            DOS_OBJECT_MAP_WIDTH,
            DOS_OBJECT_MAP_HEIGHT,
            -16,
            -16,
            &mut scene.object_map,
        )
        .unwrap();
        for y in if drop { 60..144 } else { 96..104 } {
            for x in if drop {
                64..72
            } else if late {
                120..128
            } else {
                96..104
            } {
                map.write(x, y, code);
            }
        }
        if kind < 128 {
            scene.traps[0] = Some(
                lemmings_core::TrapState::new(if kind == 1 { 48 } else { 12 }, 0)
                    .unwrap()
                    .with_continuous(kind == 2),
            );
        }
        scene.title = match kind {
            133 => "WATER",
            134 => "FIRE",
            _ => "TRAP",
        }
        .into();
        scene
    }
    #[cfg(test)]
    pub fn scene_with_terrain(
        &self,
        iteration: u64,
        mask: &[u8],
        brick_mask: &[u8],
    ) -> Result<Vec<[u8; 4]>, Box<dyn Error>> {
        self.scene_with_traps(iteration, mask, brick_mask, &[None; 16])
    }
    pub fn scene_with_traps(
        &self,
        iteration: u64,
        mask: &[u8],
        brick_mask: &[u8],
        traps: &lemmings_core::TrapStates,
    ) -> Result<Vec<[u8; 4]>, Box<dyn Error>> {
        lemmings_core::TerrainMask::new(self.width, self.height, mask)?;
        lemmings_core::TerrainMask::new(self.width, self.height, brick_mask)?;
        let Some(original) = &self.original else {
            return Ok(mask
                .iter()
                .zip(brick_mask)
                .map(|(&v, &brick)| {
                    if v == 1 {
                        if brick == 1 {
                            let [r, g, b] = self.palette[7];
                            [r, g, b, 255]
                        } else {
                            [130, 85, 45, 255]
                        }
                    } else {
                        [0, 0, 0, 255]
                    }
                })
                .collect());
        };
        let level = LevelRecord::parse(&original.level)?;
        let ground = GroundRecord::parse(&original.ground)?;
        let images = VgagrImages::new(ground, &original.terrain_images, &original.object_images)?;
        let mut terrain = vec![IndexedPixel::default(); self.width * self.height];
        let terrain = compose_terrain(
            level,
            TerrainSource::Standard(TerrainGraphics::new(original.graphics, images)?),
            &mut terrain,
        )?;
        let mut edited = terrain.pixels().to_vec();
        for ((pixel, &solid), &brick) in edited.iter_mut().zip(mask).zip(brick_mask) {
            if solid == 0 {
                *pixel = IndexedPixel::default();
            } else if brick == 1 {
                *pixel = IndexedPixel {
                    // Terrain colors are local0..7, then shifted to8..15.
                    // Custom0 is also copied to combined palette7 for bridges.
                    color: 0,
                    opaque: true,
                };
            }
        }
        let terrain = terrain
            .with_pixels(&edited)
            .ok_or("invalid edited terrain")?;
        let mut indices = [0; 32];
        let mut compact = 0;
        for object in level.objects() {
            let trap_index = compact;
            if object.id != 1 {
                compact += 1;
            }
            let meta = ground
                .object(usize::from(object.id))
                .ok_or("object metadata missing")?;
            if object.id != 1
                && meta.trigger_effect == 4
                && trap_index < 16
                && let Some(trap) = traps[trap_index]
            {
                indices[object.slot] = usize::from(trap.frame());
                continue;
            }
            indices[object.slot] = object_frame(
                object.id == 1,
                meta.animation_loops(),
                usize::from(meta.first_frame),
                usize::from(meta.frame_count),
                iteration,
            );
        }
        let mut output = vec![ScenePixel::default(); self.width * self.height];
        let scene = compose_objects(
            level,
            ObjectGraphics::new(original.graphics, images)?,
            terrain,
            ObjectFrameSelection::Explicit(&indices),
            &mut output,
        )?;
        Ok((0..self.width * self.height)
            .map(|i| scene.rgba(i % self.width, i / self.width).unwrap())
            .collect())
    }
}
fn build_object_map(
    level: LevelRecord<'_>,
    ground: GroundRecord<'_>,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut object_map = vec![OBJECT_NONE; DOS_OBJECT_MAP_WIDTH * DOS_OBJECT_MAP_HEIGHT];
    let mut raw = ObjectMapMut::new(
        DOS_OBJECT_MAP_WIDTH,
        DOS_OBJECT_MAP_HEIGHT,
        DOS_OBJECT_MAP_ORIGIN,
        DOS_OBJECT_MAP_ORIGIN,
        &mut object_map,
    )?;
    for area in level.steel() {
        for y in i64::from(area.y)..i64::from(area.y) + i64::from(area.height) {
            for x in i64::from(area.x)..i64::from(area.x) + i64::from(area.width) {
                raw.write(x, y, 137);
            }
        }
    }
    for (index, object) in level.objects().filter(|o| o.id != 1).take(16).enumerate() {
        let meta = ground
            .object(usize::from(object.id))
            .ok_or("object metadata missing")?;
        let rect = meta.trigger_rect();
        let left = i64::from((object.x & !3) + rect.left);
        let top = i64::from((i32::from(object.y) & !3) + rect.top);
        let code = if meta.trigger_effect == 4 {
            index as u8
        } else {
            meta.trigger_effect
                .checked_add(128)
                .ok_or("invalid object trigger effect")?
        };
        for y in top..top + i64::from(rect.height) {
            for x in left..left + i64::from(rect.width) {
                raw.write(x, y, code);
            }
        }
    }
    Ok(object_map)
}
fn synthetic_bash_masks() -> [u8; 160] {
    let mut masks = [0; 160];
    for direction in 0..2 {
        for frame in 0..4 {
            for y in 0..10 {
                for x in 0..16 {
                    let selected = if direction == 0 {
                        x == 8 + frame * 2 || x == 9 + frame * 2
                    } else {
                        x == 6 - frame * 2 || x == 7 - frame * 2
                    };
                    if selected {
                        masks[(direction * 4 + frame) * 20 + y * 2 + x / 8] |= 128 >> (x % 8);
                    }
                }
            }
        }
    }
    masks
}

fn load_particles(root: &Path) -> Result<Option<Vec<u8>>, Box<dyn Error>> {
    // Exact table in the pinned original EXE, matching Lemmix Particles.dat.
    let path = root.join("VGALEMMI.EXE");
    let bytes = match fs::read(&path) {
        Ok(v) => v,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(io::Error::new(e.kind(), format!("{}: {e}", path.display())).into()),
    };
    if bytes.len() != 83194 || !bytes.starts_with(b"MZ") {
        return Err(format!("{}: unsupported particle EXE profile", path.display()).into());
    }
    let table = bytes[49976..49976 + EXPLOSION_PARTICLE_DATA_SIZE].to_vec();
    ExplosionParticles::parse(&table)?;
    Ok(Some(table))
}
fn synthetic_countdown() -> [u8; 80] {
    // Own 5x7 glyphs, stored in the resource bank's9..0 order.
    let glyphs = [
        [14, 17, 19, 21, 25, 17, 14],
        [4, 12, 4, 4, 4, 4, 14],
        [14, 17, 1, 2, 4, 8, 31],
        [30, 1, 1, 14, 1, 1, 30],
        [2, 6, 10, 18, 31, 2, 2],
        [31, 16, 16, 30, 1, 1, 30],
        [14, 16, 16, 30, 17, 17, 14],
        [31, 1, 2, 4, 8, 8, 8],
        [14, 17, 17, 14, 17, 17, 14],
        [14, 17, 17, 15, 1, 1, 14],
    ];
    let mut out = [0; 80];
    for digit in 0..10 {
        for y in 0..7 {
            out[(9 - digit) * 8 + y] = glyphs[digit][y] << 2;
        }
    }
    out
}

fn build_traps(
    level: LevelRecord<'_>,
    ground: GroundRecord<'_>,
) -> Result<lemmings_core::TrapStates, Box<dyn Error>> {
    let mut traps = [None; 16];
    for (i, object) in level.objects().filter(|o| o.id != 1).take(16).enumerate() {
        let meta = ground
            .object(usize::from(object.id))
            .ok_or("object metadata missing")?;
        if meta.trigger_effect == 4 {
            traps[i] = Some(
                lemmings_core::TrapState::new(meta.frame_count, meta.first_frame)?
                    .with_continuous(meta.animation_loops()),
            );
        }
    }
    Ok(traps)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exit_trigger_uses_compact_slot_fifteen_but_disables_slot_sixteen() {
        let mut level = [0; LEVEL_RECORD_SIZE];
        for offset in [1, 3, 5, 7] {
            level[offset] = 1;
        }
        level[288..1888].fill(255);
        level[32..40].copy_from_slice(&[0, 16, 0, 0, 0, 1, 0, 0]);
        for slot in 1..=17 {
            let at = 32 + slot * 8;
            level[at..at + 2].copy_from_slice(&(16 + slot as u16 * 4).to_be_bytes());
            level[at + 5] = 2;
        }
        let mut ground = [0; GROUND_RECORD_SIZE];
        ground[56 + 14..56 + 21].copy_from_slice(&[0, 0, 1, 0, 1, 1, 1]);
        let bytes = build_object_map(
            LevelRecord::parse(&level).unwrap(),
            GroundRecord::parse(&ground).unwrap(),
        )
        .unwrap();
        let map = lemmings_core::ObjectMap::new(
            DOS_OBJECT_MAP_WIDTH,
            DOS_OBJECT_MAP_HEIGHT,
            -16,
            -16,
            &bytes,
        )
        .unwrap();
        assert_eq!(map.read(0, 0), 128);
        assert_eq!(map.read(4, 0), lemmings_core::OBJECT_EXIT);
        assert_eq!(map.read(64, 0), lemmings_core::OBJECT_EXIT);
        assert_eq!(map.read(68, 0), 128);
    }
    #[test]
    fn optional_original_particles_reject_bad_profiles_and_extract_the_exact_table() {
        let dir = std::env::temp_dir().join(format!("lemmings-particles-{}", std::process::id()));
        fs::create_dir(&dir).unwrap();
        assert!(load_particles(&dir).unwrap().is_none());
        fs::write(dir.join("VGALEMMI.EXE"), b"MZ").unwrap();
        assert!(load_particles(&dir).is_err());
        let mut bytes = vec![0; 83194];
        bytes[..2].copy_from_slice(b"MZ");
        bytes[49976..49976 + EXPLOSION_PARTICLE_DATA_SIZE].fill(128);
        bytes[49976] = 255;
        bytes[49977] = 3;
        fs::write(dir.join("VGALEMMI.EXE"), bytes).unwrap();
        let table = load_particles(&dir).unwrap().unwrap();
        assert_eq!(table.len(), EXPLOSION_PARTICLE_DATA_SIZE);
        assert_eq!(
            ExplosionParticles::parse(&table).unwrap().offset(0, 0),
            Some((-1, 3))
        );
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn raw_objects_override_steel_on_the_padded_grid_and_ignore_entrances() {
        let mut level = [0; LEVEL_RECORD_SIZE];
        for offset in [1, 3, 5, 7] {
            level[offset] = 1;
        }
        level[288..1888].fill(255);
        // Steel starts at world(-16,0), followed by an object at(-13,0).
        level[1888..1892].copy_from_slice(&[0, 0, 1, 0]);
        for (slot, x, y, id) in [(0, 3u16, 0u16, 0u16), (16, 3, 0, 1)] {
            let at = 32 + slot * 8;
            level[at..at + 2].copy_from_slice(&x.to_be_bytes());
            level[at + 2..at + 4].copy_from_slice(&y.to_be_bytes());
            level[at + 4..at + 6].copy_from_slice(&id.to_be_bytes());
        }
        let mut ground = [0; GROUND_RECORD_SIZE];
        // Trigger [0,4)x[0,4) becomes cell(-16,0) after grid rounding.
        ground[14..16].copy_from_slice(&[0, 0]);
        ground[16..18].copy_from_slice(&[1, 0]);
        ground[18..20].copy_from_slice(&[1, 1]);
        ground[20] = 2;
        ground[28 + 14..28 + 21].copy_from_slice(&[0, 0, 1, 0, 1, 1, 3]);
        let bytes = build_object_map(
            LevelRecord::parse(&level).unwrap(),
            GroundRecord::parse(&ground).unwrap(),
        )
        .unwrap();
        let map = lemmings_core::ObjectMap::new(
            DOS_OBJECT_MAP_WIDTH,
            DOS_OBJECT_MAP_HEIGHT,
            -16,
            -16,
            &bytes,
        )
        .unwrap();
        assert_eq!(map.read(-16, 0), 130); // first object overrides steel
        assert_eq!(map.read(-12, 0), 128); // no ceil expansion
        assert_eq!(map.read(-17, 0), 128); // clipped outside padding
        ground[20] = 255;
        assert!(
            build_object_map(
                LevelRecord::parse(&level).unwrap(),
                GroundRecord::parse(&ground).unwrap()
            )
            .is_err()
        );
    }
    #[test]
    fn object_limit_and_trap_codes_use_the_compact_non_entrance_list() {
        let mut level = [0; LEVEL_RECORD_SIZE];
        for offset in [1, 3, 5, 7] {
            level[offset] = 1;
        }
        level[288..1888].fill(255);
        level[32..40].copy_from_slice(&[0, 16, 0, 0, 0, 1, 0, 0]);
        for slot in 1..=17 {
            let at = 32 + slot * 8;
            level[at..at + 2].copy_from_slice(&(16 + slot as u16 * 4).to_be_bytes());
            level[at + 5] = 2;
        }
        let mut ground = [0; GROUND_RECORD_SIZE];
        ground[28 + 14..28 + 21].copy_from_slice(&[0, 0, 1, 0, 1, 1, 3]);
        ground[56 + 14..56 + 21].copy_from_slice(&[0, 0, 1, 0, 1, 1, 4]);
        let bytes = build_object_map(
            LevelRecord::parse(&level).unwrap(),
            GroundRecord::parse(&ground).unwrap(),
        )
        .unwrap();
        let map = lemmings_core::ObjectMap::new(
            DOS_OBJECT_MAP_WIDTH,
            DOS_OBJECT_MAP_HEIGHT,
            -16,
            -16,
            &bytes,
        )
        .unwrap();
        assert_eq!(map.read(0, 0), 128); // entrance is not an interactive-map entry
        assert_eq!(map.read(4, 0), 0); // compact trap index, not file slot one
        assert_eq!(map.read(64, 0), 15); // sixteenth non-entrance is active
        assert_eq!(map.read(68, 0), 128); // seventeenth is disabled
    }
    #[test]
    fn original_bricks_use_first_custom_terrain_color_as_the_shared_bridge_color() {
        let mut a = Assets::synthetic();
        a.width = LEVEL_TERRAIN_WIDTH;
        a.height = LEVEL_TERRAIN_HEIGHT;
        let mut level = [0; LEVEL_RECORD_SIZE];
        for offset in [1, 3, 5, 7] {
            level[offset] = 1;
        }
        level[288..1888].fill(255);
        let mut ground = vec![0; GROUND_RECORD_SIZE];
        ground[984..987].copy_from_slice(&[48, 44, 60]);
        ground[1005..1008].copy_from_slice(&[36, 8, 4]);
        a.original = Some(Original {
            graphics: 0,
            level,
            ground,
            terrain_images: Vec::new(),
            object_images: Vec::new(),
        });
        let mut mask = vec![0; a.width * a.height];
        let mut bricks = mask.clone();
        let index = 99 * a.width + 70;
        mask[index] = 1;
        bricks[index] = 1;
        assert_eq!(
            a.scene_with_terrain(0, &mask, &bricks).unwrap()[index],
            [194, 178, 242, 255]
        );
    }
    #[test]
    fn brick_material_survives_regrowth_and_is_removed_with_the_collision_pixel() {
        let a = Assets::synthetic();
        let mut mask = a.terrain.clone();
        let mut bricks = vec![0; mask.len()];
        let index = 99 * a.width + 70;
        mask[index] = 1;
        bricks[index] = 1;
        let scene = a.scene_with_terrain(0, &mask, &bricks).unwrap();
        let [r, g, b] = a.palette[7];
        assert_eq!(scene[index], [r, g, b, 255]);
        mask[index] = 0;
        assert_eq!(
            a.scene_with_terrain(0, &mask, &bricks).unwrap()[index],
            [0, 0, 0, 255]
        );
    }
    #[test]
    fn entrance_and_continuous_visual_frames_use_iteration_only() {
        assert_eq!(object_frame(true, false, 1, 10, 34), 1);
        assert_eq!(object_frame(true, false, 1, 10, 35), 2);
        assert_eq!(object_frame(true, false, 1, 10, 42), 9);
        assert_eq!(object_frame(true, false, 1, 10, 43), 0);
        assert_eq!(object_frame(true, false, 1, 10, 999), 0);
        assert_eq!(object_frame(false, true, 2, 8, 9), 5);
        assert_eq!(object_frame(false, false, 2, 8, 9), 2);
    }
    #[test]
    fn dos_continuous_frames_match_the_observed_fun01_flag_phase() {
        for (iteration, frame) in [
            (0, 2),
            (35, 1),
            (54, 2),
            (60, 2),
            (100, 0),
            (170, 4),
            (340, 0),
        ] {
            assert_eq!(object_frame(false, true, 0, 6, iteration), frame);
        }
        assert_eq!(object_frame(false, true, 0, 6, u64::MAX), 5);
        assert_eq!(object_frame(false, true, 0, 1, u64::MAX), 0);
    }
    #[test]
    fn missing_or_truncated_original_files_report_the_path() {
        let directory =
            std::env::temp_dir().join(format!("lemmings-assets-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let error = Assets::load(&directory).err().unwrap().to_string();
        assert!(error.contains("LEVEL009.DAT"));
        fs::write(directory.join("LEVEL009.DAT"), [0; 4]).unwrap();
        assert!(Assets::load(&directory).is_err());
        fs::remove_dir_all(directory).unwrap();
    }
}
