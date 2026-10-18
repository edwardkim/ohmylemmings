use lemmings_core::{Difficulty, EntrancePoint, LevelId, StartupConfig};
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
    pub mine_masks: [u8; 104],
    pub bash_masks: [u8; 160],
    pub one_way_left: Vec<u8>,
    pub one_way_right: Vec<u8>,
    pub title: String,
    pub entrances: Vec<EntrancePoint>,
    pub config: StartupConfig,
    pub screen_x: i32,
    pub sprites: Vec<u8>,
    pub palette: [[u8; 3]; 16],
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
            width: LEVEL_TERRAIN_WIDTH,
            height: LEVEL_TERRAIN_HEIGHT,
            terrain: mask,
            steel,
            diggers,
            bashers,
            miners: header.skills[6],
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
            mine_masks: [255; 104], // own full rectangular synthetic erase masks
            // Own synthetic masks (rectangular slices), never original bits.
            bash_masks: synthetic_bash_masks(),
            one_way_left: vec![0; width * height],
            one_way_right: vec![0; width * height],
            title: "JUST DIG!".into(),
            entrances: vec![EntrancePoint { x: 40, y: 40 }],
            config: StartupConfig {
                time_limit_minutes: 5,
                total_lemmings: 10,
                initial_release_rate: 50,
                entrance_count: 1,
                pause_glitch: true,
            },
            screen_x: 0,
            sprites: vec![255; LEMMING_SPRITE_DATA_SIZE],
            palette: [
                [0, 0, 0],
                [0, 220, 0],
                [0, 80, 240],
                [240, 240, 240],
                [0; 3],
                [0; 3],
                [0; 3],
                [0; 3],
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
    pub fn scene_with_terrain(
        &self,
        iteration: u64,
        mask: &[u8],
    ) -> Result<Vec<[u8; 4]>, Box<dyn Error>> {
        lemmings_core::TerrainMask::new(self.width, self.height, mask)?;
        let Some(original) = &self.original else {
            return Ok(mask
                .iter()
                .map(|&v| {
                    if v == 1 {
                        [130, 85, 45, 255]
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
        for (pixel, &solid) in edited.iter_mut().zip(mask) {
            if solid == 0 {
                *pixel = IndexedPixel::default();
            }
        }
        let terrain = terrain
            .with_pixels(&edited)
            .ok_or("invalid edited terrain")?;
        let mut indices = [0; 32];
        for object in level.objects() {
            let meta = ground
                .object(usize::from(object.id))
                .ok_or("object metadata missing")?;
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
#[cfg(test)]
mod tests {
    use super::*;
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
