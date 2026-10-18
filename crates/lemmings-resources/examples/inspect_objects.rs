//! Compose 120 static object scenes; select preview, initial, or last disk frames.
use lemmings_core::{Difficulty, LevelId};
use lemmings_resources::{
    CatalogEntry, DosLevelFile, GroundRecord, IndexedPixel, LEVEL_RECORD_SIZE,
    LEVEL_TERRAIN_PIXEL_COUNT, ObjectFrameSelection, ObjectGraphics, OddTable, ScenePixel,
    SpecialTerrain, TerrainGraphics, TerrainSource, VGASPEC_PLANAR_SIZE, VgaSpecFile,
    VgaSpecRecord, VgagrFile, VgagrImages, compose_objects, compose_terrain,
};
use std::{
    env,
    error::Error,
    fs::{self, OpenOptions},
    io::{self, BufWriter, Write},
    path::Path,
};
fn read(path: &Path) -> io::Result<Vec<u8>> {
    fs::read(path).map_err(|e| io::Error::new(e.kind(), format!("{}: {e}", path.display())))
}
fn data_error(path: &Path, error: impl std::fmt::Display) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("{}: {error}", path.display()),
    )
}
fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args_os().skip(1);
    let (Some(root), Some(destination), Some(mode), None) =
        (args.next(), args.next(), args.next(), args.next())
    else {
        return Err(
            "usage: inspect_objects ORIGINAL_ASSET_DIRECTORY OUTPUT_DIRECTORY preview|initial|last"
                .into(),
        );
    };
    let mode = mode
        .to_str()
        .filter(|m| matches!(*m, "preview" | "initial" | "last"))
        .ok_or("mode must be preview, initial, or last")?;
    let root = Path::new(&root);
    let destination = Path::new(&destination);
    let ground_bytes: Vec<_> = (0..5)
        .map(|i| read(&root.join(format!("GROUND{i}O.DAT"))))
        .collect::<Result<_, _>>()?;
    let mut decoded_graphics = Vec::new();
    for (index, ground) in ground_bytes.iter().enumerate() {
        let path = root.join(format!("VGAGR{index}.DAT"));
        let bytes = read(&path)?;
        let file = VgagrFile::parse(&bytes).map_err(|e| data_error(&path, e))?;
        let [terrain_size, object_size] = file.decoded_sizes();
        let mut terrain = vec![0; terrain_size];
        let mut objects = vec![0; object_size];
        let ground_path = root.join(format!("GROUND{index}O.DAT"));
        let ground = GroundRecord::parse(ground).map_err(|e| data_error(&ground_path, e))?;
        file.decode_into(ground, &mut terrain, &mut objects)
            .map_err(|e| data_error(&path, e))?;
        decoded_graphics.push((terrain, objects));
    }
    let graphics: Vec<_> = decoded_graphics
        .iter()
        .zip(&ground_bytes)
        .enumerate()
        .map(|(index, ((terrain, objects), ground))| {
            let images = VgagrImages::new(GroundRecord::parse(ground)?, terrain, objects)?;
            Ok(TerrainGraphics::new(index as u8, images)?)
        })
        .collect::<Result<_, Box<dyn Error>>>()?;
    let object_graphics: Vec<_> = decoded_graphics
        .iter()
        .zip(&ground_bytes)
        .enumerate()
        .map(|(index, ((terrain, objects), ground))| {
            Ok(ObjectGraphics::new(
                index as u8,
                VgagrImages::new(GroundRecord::parse(ground)?, terrain, objects)?,
            )?)
        })
        .collect::<Result<_, Box<dyn Error>>>()?;
    let mut special_records = Vec::new();
    for index in 0..4 {
        let path = root.join(format!("VGASPEC{index}.DAT"));
        let bytes = read(&path)?;
        let file = VgaSpecFile::parse(&bytes).map_err(|e| data_error(&path, e))?;
        let mut inner = vec![0; file.decoded_size()];
        file.decode_into(&mut inner)
            .map_err(|e| data_error(&path, e))?;
        special_records.push(inner);
    }
    let mut special_planes = vec![vec![0; VGASPEC_PLANAR_SIZE]; 4];
    let special: Vec<_> = special_records
        .iter()
        .zip(special_planes.iter_mut())
        .enumerate()
        .map(|(index, (record, planes))| {
            Ok(SpecialTerrain::decode_into(
                index as u8,
                VgaSpecRecord::parse(record)?,
                planes,
            )?)
        })
        .collect::<Result<_, Box<dyn Error>>>()?;
    let level_bytes: Vec<_> = (0..10)
        .map(|i| read(&root.join(format!("LEVEL{i:03}.DAT"))))
        .collect::<Result<_, _>>()?;
    let files: Vec<_> = level_bytes
        .iter()
        .enumerate()
        .map(|(i, bytes)| {
            DosLevelFile::parse(i as u8, bytes)
                .map_err(|e| data_error(&root.join(format!("LEVEL{i:03}.DAT")), e))
        })
        .collect::<Result<_, _>>()?;
    let odd_bytes = read(&root.join("ODDTABLE.DAT"))?;
    let odd = OddTable::parse(&odd_bytes).map_err(|e| data_error(&root.join("ODDTABLE.DAT"), e))?;
    fs::create_dir_all(destination)?;
    let mut stdout = BufWriter::new(io::stdout().lock());
    let mut canvas = vec![IndexedPixel::default(); LEVEL_TERRAIN_PIXEL_COUNT];
    let mut scene_pixels = vec![ScenePixel::default(); LEVEL_TERRAIN_PIXEL_COUNT];
    for difficulty in [
        Difficulty::Fun,
        Difficulty::Tricky,
        Difficulty::Taxing,
        Difficulty::Mayhem,
    ] {
        for number in 1..=30 {
            let id = LevelId::new(difficulty, number)?;
            let entry = CatalogEntry::for_level(id);
            let mut base = [0; LEVEL_RECORD_SIZE];
            let level = entry.load(
                &files[usize::from(entry.file_index())],
                Some(odd),
                &mut base,
            )?;
            let header = level.header();
            let source = match header.special_graphic_set() {
                Some(index) => TerrainSource::Special(special[usize::from(index)]),
                None => TerrainSource::Standard(graphics[usize::from(header.graphics)]),
            };
            let map = compose_terrain(level.base_record(), source, &mut canvas).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("{} {difficulty:?} {number}: {e}", root.display()),
                )
            })?;
            let mut indices = [0; 32];
            if mode == "last" {
                let (terrain, objects) = &decoded_graphics[usize::from(header.graphics)];
                let images = VgagrImages::new(
                    GroundRecord::parse(&ground_bytes[usize::from(header.graphics)])?,
                    terrain,
                    objects,
                )?;
                for p in level.base_record().objects() {
                    let frames = images.object(usize::from(p.id))?.ok_or_else(|| {
                        data_error(
                            root,
                            format!("object slot {}: empty image {}", p.slot, p.id),
                        )
                    })?;
                    indices[p.slot] = usize::from(frames.frame_count()) - 1;
                }
            }
            let selection = match mode {
                "preview" => ObjectFrameSelection::Preview,
                "initial" => ObjectFrameSelection::Initial,
                _ => ObjectFrameSelection::Explicit(&indices),
            };
            let map = compose_objects(
                level.base_record(),
                object_graphics[usize::from(header.graphics)],
                map,
                selection,
                &mut scene_pixels,
            )
            .map_err(|e| data_error(root, format!("{difficulty:?} {number}: {e}")))?;
            let name = format!("level-{:03}.scene.bin", id.global_index());
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(destination.join(&name))
                .map_err(|e| {
                    io::Error::new(
                        e.kind(),
                        format!("{}: {e}", destination.join(&name).display()),
                    )
                })?;
            let bytes: Vec<u8> = map
                .pixels()
                .iter()
                .map(|p| p.color | if p.terrain { 16 } else { 0 } | if p.object { 32 } else { 0 })
                .collect();
            file.write_all(&bytes)?;
            write!(
                stdout,
                "{{\"global_index\":{},\"group\":\"{difficulty:?}\",\"number\":{number},\"file\":{},\"part\":{},\"graphics\":{},\"special\":{},\"pixel_file\":\"{name}\",\"mode\":\"{mode}\",\"object_pixels\":{},\"terrain_pixels\":{},\"opaque_pixels\":{},\"palette_hex\":\"",
                id.global_index(),
                entry.file_index(),
                entry.part_index(),
                header.graphics,
                header.special,
                map.pixels().iter().filter(|p| p.object).count(),
                map.pixels().iter().filter(|p| p.terrain).count(),
                map.pixels().iter().filter(|p| p.opaque()).count(),
            )?;
            for color in map.palette() {
                for channel in color {
                    write!(stdout, "{channel:02x}")?;
                }
            }
            write!(stdout, "\",\"terrain_palette_hex\":\"")?;
            for color in map.terrain_palette() {
                for channel in color {
                    write!(stdout, "{channel:02x}")?;
                }
            }
            writeln!(stdout, "\"}}")?;
        }
    }
    stdout.flush()?;
    Ok(())
}
