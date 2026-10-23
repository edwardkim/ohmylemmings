//! Original catalog HD terrain checks. Filesystem access stays in this host example.
//! Initial scene coverage plus selected 120-tick pairs is not level completion.
use lemmings_core::{Difficulty, LevelId};
use lemmings_game::{Assets, LevelRun, diagnostics, hd::scene::Renderer, pacing::Pacing};
use lemmings_resources::{CatalogEntry, DosLevelFile, LEVEL_RECORD_SIZE, OddTable};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs,
    path::Path,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
fn load(bank: &BTreeMap<String, Vec<u8>>, id: LevelId) -> Result<Assets> {
    Assets::from_level_source(&mut |name: &str| Ok(bank.get(name).cloned()), id)
}
fn same(a: &LevelRun, b: &LevelRun) -> bool {
    diagnostics::state(&a.session) == diagnostics::state(&b.session)
        && a.session.terrain() == b.session.terrain()
        && a.session.brick_mask() == b.session.brick_mask()
        && a.session.object_map_data() == b.session.object_map_data()
        && a.sound.diagnostic_json() == b.sound.diagnostic_json()
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        return Err("usage: hd_terrain_probe PACK ORIGINAL_ASSET_DIRECTORY".into());
    }
    let mut renderer = Renderer::new(&fs::read(&args[1])?)?;
    let mut bank = BTreeMap::new();
    for entry in fs::read_dir(Path::new(&args[2]))? {
        let path = entry?.path();
        if path.is_file() {
            bank.insert(
                path.file_name().unwrap().to_string_lossy().into_owned(),
                fs::read(path)?,
            );
        }
    }
    let mut selected = BTreeSet::new();
    for difficulty in [
        Difficulty::Fun,
        Difficulty::Tricky,
        Difficulty::Taxing,
        Difficulty::Mayhem,
    ] {
        for number in 1..=30 {
            let id = LevelId::new(difficulty, number)?;
            let entry = CatalogEntry::for_level(id);
            let file = DosLevelFile::parse(
                entry.file_index(),
                &bank[&format!("LEVEL{:03}.DAT", entry.file_index())],
            )?;
            let mut record = [0; LEVEL_RECORD_SIZE];
            let odd = entry
                .odd_index()
                .map(|_| OddTable::parse(&bank["ODDTABLE.DAT"]))
                .transpose()?;
            let level = entry.load(&file, odd, &mut record)?;
            let set = level.header().graphics;
            let special = level.header().special;
            let mut run = LevelRun::new_for_level(load(&bank, id)?, id, id.next(), Pacing::Dos)?;
            let mut plain = LevelRun::new_for_level(load(&bank, id)?, id, id.next(), Pacing::Dos)?;
            let left = run.assets.one_way_left.iter().any(|&v| v != 0);
            let right = run.assets.one_way_right.iter().any(|&v| v != 0);
            renderer.reset(&run.assets)?;
            renderer.frame(&run)?;
            let info = renderer.info();
            let expected = if special == 0 {
                set.to_string()
            } else {
                "null".into()
            };
            if !info.contains(&format!("\"terrain_material\":{expected},")) {
                return Err(format!("wrong material {id:?}: {info}").into());
            }
            let steps = if selected.insert((set, special, left, right)) {
                120
            } else {
                0
            };
            for step in 0..steps {
                run.tick()?;
                plain.tick()?;
                if step % 20 == 0 || step == steps - 1 {
                    renderer.frame(&run)?;
                }
                if !same(&run, &plain) {
                    return Err(format!("HD changed simulation/masks/audio {id:?}/{step}").into());
                }
            }
            if !same(&run, &plain) {
                return Err("initial rendering changed state".into());
            }
            println!(
                "{{\"rank\":\"{}\",\"number\":{number},\"set\":{set},\"special\":{special},\"left\":{left},\"right\":{right},\"paired_steps\":{steps},\"state\":{},\"info\":{}}}",
                difficulty.name(),
                diagnostics::state(&run.session),
                renderer.info()
            );
        }
    }
    Ok(())
}
