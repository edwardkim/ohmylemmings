//! Presentation dependencies only. Plans never render or advance the simulation.
use crate::{Assets, assets::AssetSource};
use lemmings_core::LevelId;
use lemmings_resources::{
    CatalogEntry, DosLevelFile, GroundRecord, LEVEL_RECORD_SIZE, LevelHeader, OddTable,
};
use std::{collections::BTreeSet, error::Error};

/// Include every animation reachable through the level's initial skills and
/// objects. Nuke is always available, including levels with zero bombers.
pub fn requirements(
    header: LevelHeader,
    objects: &[(u8, u8)],
) -> Result<Vec<[u8; 2]>, Box<dyn Error>> {
    if header.graphics > 4 || header.special > 4 || objects.iter().any(|&(id, _)| id > 15) {
        return Err("invalid HD download selectors".into());
    }
    let mut keys = BTreeSet::new();
    for slot in [0, 1, 2, 3, 16, 17, 20, 21, 26, 27] {
        keys.insert([0, slot]);
    }
    let skills: [&[u8]; 8] = [
        &[5, 6, 8, 9],
        &[18, 19],
        &[],
        &[23],
        &[10, 11, 24, 25],
        &[12, 13],
        &[14, 15],
        &[4],
    ];
    for (&count, slots) in header.skills.iter().zip(skills) {
        if count != 0 {
            for &slot in slots {
                keys.insert([0, slot]);
            }
        }
    }
    keys.insert([1, header.graphics as u8]);
    for &(id, effect) in objects {
        keys.insert([2, header.graphics as u8 * 16 + id]);
        match effect {
            5 => {
                keys.insert([0, 7]);
            }
            6 => {
                keys.insert([0, 22]);
            }
            _ => {}
        }
    }
    for control in 0..13 {
        keys.insert([3, control]);
    }
    if let Some(special) = header.special_graphic_set() {
        keys.insert([4, special]);
    }
    Ok(keys.into_iter().collect())
}

pub fn for_assets(assets: &Assets) -> Result<Vec<[u8; 2]>, Box<dyn Error>> {
    let skills = [
        assets.climbers,
        assets.floaters,
        assets.bombers,
        assets.blockers,
        assets.builders,
        assets.bashers,
        assets.miners,
        assets.diggers,
    ];
    if let Some((set, level, ground, _)) = assets.hd_art()? {
        let mut header = level.header();
        header.graphics = set.into();
        // ODDTABLE overrides the base record's inventory in the live assets.
        header.skills = skills;
        let objects = level
            .objects()
            .map(|p| {
                ground
                    .object(p.id as usize)
                    .map(|o| (p.id, o.trigger_effect))
                    .ok_or("object metadata")
            })
            .collect::<Result<Vec<_>, _>>()?;
        requirements(header, &objects)
    } else {
        let header = LevelHeader {
            release_rate: 1,
            total: 1,
            required: 1,
            minutes: 1,
            skills,
            screen_x: 0,
            graphics: 0,
            special: 0,
            superlemming: false,
        };
        let mut keys = requirements(header, &[])?;
        // Invented courses do not carry original object records.
        for (code, slot) in [
            (lemmings_core::OBJECT_WATER, 7),
            (lemmings_core::OBJECT_FIRE, 22),
        ] {
            if assets.object_map.contains(&code) {
                keys.push([0, slot]);
            }
        }
        keys.sort_unstable();
        keys.dedup();
        Ok(keys)
    }
}

/// Resolve only the small level/ODD/GROUND metadata. No terrain composition,
/// sprite decoding, renderer or game construction is needed for prefetching.
pub fn from_source(
    source: &mut dyn AssetSource,
    level: LevelId,
) -> Result<Vec<[u8; 2]>, Box<dyn Error>> {
    fn read(source: &mut dyn AssetSource, name: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        source
            .read(name)?
            .ok_or_else(|| format!("{name}: missing download-plan metadata").into())
    }
    let entry = CatalogEntry::for_level(level);
    let bytes = read(source, &format!("LEVEL{:03}.DAT", entry.file_index()))?;
    let file = DosLevelFile::parse(entry.file_index(), &bytes)?;
    let odd = if entry.odd_index().is_some() {
        Some(read(source, "ODDTABLE.DAT")?)
    } else {
        None
    };
    let mut record = [0; LEVEL_RECORD_SIZE];
    let resolved = entry.load(
        &file,
        odd.as_deref().map(OddTable::parse).transpose()?,
        &mut record,
    )?;
    let bytes = read(
        source,
        &format!("GROUND{}O.DAT", resolved.header().graphics),
    )?;
    let ground = GroundRecord::parse(&bytes)?;
    let objects = resolved
        .objects()
        .map(|p| {
            ground
                .object(p.id as usize)
                .map(|o| (p.id, o.trigger_effect))
                .ok_or("object metadata")
        })
        .collect::<Result<Vec<_>, _>>()?;
    requirements(resolved.header(), &objects)
}
