use lemmings_game::{AssetSource, Assets, MissingAsset};
use std::error::Error;

#[test]
fn all_difficulties_request_the_selected_catalog_file_before_scene_assembly() {
    use lemmings_core::{Difficulty, LevelId};
    use lemmings_resources::CatalogEntry;
    for difficulty in [
        Difficulty::Fun,
        Difficulty::Tricky,
        Difficulty::Taxing,
        Difficulty::Mayhem,
    ] {
        for number in 1..=30 {
            let level = LevelId::new(difficulty, number).unwrap();
            let mut requested = Vec::new();
            let error = Assets::from_level_source(
                &mut |name: &str| {
                    requested.push(name.to_owned());
                    Ok(None)
                },
                level,
            )
            .err()
            .unwrap();
            let expected = format!(
                "LEVEL{:03}.DAT",
                CatalogEntry::for_level(level).file_index()
            );
            assert_eq!(requested.as_slice(), std::slice::from_ref(&expected));
            assert_eq!(error.downcast_ref::<MissingAsset>().unwrap().name, expected);
        }
    }
}

#[test]
fn missing_asset_retains_its_logical_name() {
    let mut source = |_: &str| -> Result<Option<Vec<u8>>, Box<dyn Error>> { Ok(None) };
    let error = Assets::from_source(&mut source as &mut dyn AssetSource, 1)
        .err()
        .unwrap();
    assert_eq!(
        error.downcast_ref::<MissingAsset>().unwrap().name,
        "LEVEL009.DAT"
    );
}

mod support;
use lemmings_game::{
    LevelRun, ResultAction,
    pacing::Pacing,
    render::{HEIGHT, WIDTH},
};

fn load(
    bank: &std::collections::BTreeMap<String, Vec<u8>>,
    number: u8,
) -> Result<Assets, Box<dyn Error>> {
    Assets::from_source(&mut |name: &str| Ok(bank.get(name).cloned()), number)
}

#[test]
fn named_bytes_build_the_full_scene_panel_and_owned_session() {
    let bank = support::bank(false);
    let mut names = Vec::new();
    let assets = Assets::from_source(
        &mut |name: &str| {
            names.push(name.to_owned());
            Ok(bank.get(name).cloned())
        },
        1,
    )
    .unwrap();
    assert_eq!(
        names,
        [
            "LEVEL009.DAT",
            "GROUND0O.DAT",
            "VGAGR0.DAT",
            "MAIN.DAT",
            "VGALEMMI.EXE"
        ]
    );
    assert_eq!(assets.title, "BYTE BANK");
    assert_eq!(
        (
            assets.climbers,
            assets.floaters,
            assets.bombers,
            assets.blockers,
            assets.builders,
            assets.bashers,
            assets.miners,
            assets.diggers
        ),
        (1, 2, 3, 4, 5, 6, 7, 8)
    );
    assert_eq!(assets.entrances[0].x, 48);
    assert_eq!(assets.terrain.iter().filter(|&&x| x != 0).count(), 8);
    assert!(assets.particles.is_none());
    let mut run = LevelRun::new(assets, 1, Some(2), Pacing::Reference).unwrap();
    let before = (
        run.session.snapshot(),
        run.session.completion_snapshot(),
        run.session.terrain().to_vec(),
    );
    let frame = run.view.frame(&run.assets, &run.session).unwrap();
    assert_eq!(frame.len(), WIDTH * HEIGHT * 3);
    assert_eq!(frame, run.view.frame(&run.assets, &run.session).unwrap());
    assert_eq!(
        before,
        (
            run.session.snapshot(),
            run.session.completion_snapshot(),
            run.session.terrain().to_vec()
        )
    );
}

#[test]
fn missing_and_truncated_dependencies_fail_instead_of_substituting_art() {
    let complete = support::bank(false);
    for name in ["LEVEL009.DAT", "GROUND0O.DAT", "VGAGR0.DAT", "MAIN.DAT"] {
        let mut bank = complete.clone();
        bank.remove(name);
        let error = load(&bank, 1).err().unwrap();
        assert_eq!(error.downcast_ref::<MissingAsset>().unwrap().name, name);
        bank.insert(name.into(), vec![0; 4]);
        assert!(load(&bank, 1).is_err(), "{name}");
    }
}

#[test]
fn odd_routes_request_the_table_and_apply_its_header_and_title() {
    let mut bank = support::bank(false);
    let record = &mut bank.get_mut("ODDTABLE.DAT").unwrap()[6 * 56..7 * 56];
    record[9] = 9;
    record[24..].fill(b' ');
    record[24..27].copy_from_slice(b"ODD");
    let assets = load(&bank, 8).unwrap();
    assert_eq!(assets.title, "ODD");
    assert_eq!(assets.climbers, 9);
    bank.remove("ODDTABLE.DAT");
    assert_eq!(
        load(&bank, 8)
            .err()
            .unwrap()
            .downcast_ref::<MissingAsset>()
            .unwrap()
            .name,
        "ODDTABLE.DAT"
    );
    bank.insert("ODDTABLE.DAT".into(), vec![0; 4]);
    assert!(load(&bank, 8).is_err());
}

#[test]
fn unsupported_numbers_and_missing_special_files_fail_explicitly() {
    for number in [0, 31, 255] {
        let error = Assets::from_source(
            &mut |_: &str| -> Result<Option<Vec<u8>>, Box<dyn Error>> {
                panic!("invalid level requested bytes")
            },
            number,
        );
        assert!(error.is_err());
    }
    let bank = support::bank(true);
    let mut names = Vec::new();
    let error = Assets::from_source(
        &mut |name: &str| {
            names.push(name.to_owned());
            Ok(bank.get(name).cloned())
        },
        1,
    )
    .err()
    .unwrap();
    assert_eq!(
        error.downcast_ref::<MissingAsset>().unwrap().name,
        "VGASPEC0.DAT"
    );
    assert!(names.contains(&"VGASPEC0.DAT".to_owned()));
}

#[test]
fn optional_particle_bytes_extract_the_pinned_table_and_propagate_host_errors() {
    let mut bank = support::bank(false);
    bank.insert("VGALEMMI.EXE".into(), b"MZ".to_vec());
    assert!(
        load(&bank, 1)
            .err()
            .unwrap()
            .to_string()
            .contains("VGALEMMI.EXE")
    );
    let mut exe = vec![0; 83194];
    exe[..2].copy_from_slice(b"MZ");
    exe[49976..49976 + lemmings_resources::EXPLOSION_PARTICLE_DATA_SIZE].fill(128);
    exe[49976] = 255;
    exe[49977] = 3;
    bank.insert("VGALEMMI.EXE".into(), exe);
    let assets = load(&bank, 1).unwrap();
    let table = assets.particles.unwrap();
    assert_eq!(
        lemmings_resources::ExplosionParticles::parse(&table)
            .unwrap()
            .offset(0, 0),
        Some((-1, 3))
    );
    for failed in ["LEVEL009.DAT", "GROUND0O.DAT", "MAIN.DAT", "VGALEMMI.EXE"] {
        let error = Assets::from_source(
            &mut |name: &str| {
                if name == failed {
                    Err(std::io::Error::other("host read failed").into())
                } else {
                    Ok(bank.get(name).cloned())
                }
            },
            1,
        )
        .err()
        .unwrap();
        assert_eq!(
            error.downcast_ref::<std::io::Error>().unwrap().to_string(),
            "host read failed"
        );
    }
}

#[test]
fn failed_byte_load_preserves_the_finished_scene_and_restart_uses_fresh_bytes() {
    let mut run = LevelRun::new(
        Assets::synthetic_completion("win"),
        1,
        Some(2),
        Pacing::Reference,
    )
    .unwrap();
    for _ in 0..1200 {
        run.session.tick().unwrap();
        if run.session.result().is_some() {
            break;
        }
    }
    assert!(run.session.result().unwrap().success);
    let frame = run.view.frame(&run.assets, &run.session).unwrap();
    let state = run.session.snapshot();
    let terrain = run.session.terrain().to_vec();
    for bank in [
        std::collections::BTreeMap::new(),
        std::collections::BTreeMap::from([("LEVEL009.DAT".into(), vec![0; 4])]),
    ] {
        assert!(
            run.prepare(ResultAction::Restart, Pacing::Reference, |n| Ok((
                load(&bank, n.number())?,
                lemmings_core::LevelId::new(lemmings_core::Difficulty::Fun, 2).ok()
            )))
            .is_err()
        );
        assert_eq!(state, run.session.snapshot());
        assert_eq!(terrain, run.session.terrain());
        assert_eq!(frame, run.view.frame(&run.assets, &run.session).unwrap());
    }
    let bank = support::bank(false);
    let restarted = run
        .prepare(ResultAction::Restart, Pacing::Reference, |n| {
            Ok((load(&bank, n.number())?, n.next()))
        })
        .unwrap()
        .unwrap();
    assert_eq!(restarted.assets.title, "BYTE BANK");
    assert!(restarted.session.result().is_none());
    assert_eq!(restarted.session.scheduler.step_state().control_step, 0);
}

#[test]
fn special_sources_keep_offsets_opacity_palette_edits_and_object_priority() {
    use lemmings_core::{Difficulty, LevelId};
    for (index, difficulty, number) in [
        (3, Difficulty::Fun, 22),
        (1, Difficulty::Tricky, 14),
        (0, Difficulty::Taxing, 15),
        (2, Difficulty::Mayhem, 22),
    ] {
        let bank = support::special_bank(index);
        let mut names = Vec::new();
        let a = Assets::from_level_source(
            &mut |name: &str| {
                names.push(name.to_owned());
                Ok(bank.get(name).cloned())
            },
            LevelId::new(difficulty, number).unwrap(),
        )
        .unwrap();
        assert_eq!(a.music_special, u16::from(index + 1));
        assert!(names.contains(&format!("VGASPEC{index}.DAT")));
        assert_eq!(a.terrain.iter().map(|&v| usize::from(v)).sum::<usize>(), 2);
        assert_eq!(&a.terrain[304..308], &[0, 1, 1, 0]);
        assert_eq!(a.terrain[100 * a.width + 48], 0);
        let mut mask = a.terrain.clone();
        let mut bricks = vec![0; mask.len()];
        let image = a.scene_with_traps(0, &mask, &bricks, &[None; 16]).unwrap();
        assert_eq!(image[305], [((10 + index) << 2), 80, 120, 255]);
        assert_eq!(image[306], [0, 0, 0, 255]); // black is solid
        assert_eq!(image[303], [0, 0, 0, 0]);
        // Entrance is color1 in the fixed object bank, independent of the picture.
        assert_eq!(image[4 * a.width + 304], [64, 64, 224, 255]);
        for x in [306, 310] {
            mask[x] = 1;
            bricks[x] = 1;
        }
        mask[305] = 0;
        let image = a.scene_with_traps(50, &mask, &bricks, &[None; 16]).unwrap();
        assert_eq!(image[305], [0, 0, 0, 0]);
        assert_eq!(image[306], [124, 124, 0, 255]); // special bridge color
        assert_eq!(image[310], [124, 124, 0, 255]); // new terrain
        mask[4 * a.width + 304] = 1;
        bricks[4 * a.width + 304] = 1;
        assert_eq!(
            a.scene_with_traps(50, &mask, &bricks, &[None; 16]).unwrap()[4 * a.width + 304],
            [64, 64, 224, 255]
        );
        mask[310] = 0;
        assert_eq!(
            a.scene_with_traps(50, &mask, &bricks, &[None; 16]).unwrap()[310],
            [0, 0, 0, 0]
        );
    }
}

#[test]
fn special_dependencies_reject_damaged_records_and_propagate_read_errors() {
    let complete = support::special_bank(0);
    for bytes in [vec![0; 4], {
        let mut b = complete["VGASPEC0.DAT"].clone();
        b[1] ^= 1;
        b
    }] {
        let mut bank = complete.clone();
        bank.insert("VGASPEC0.DAT".into(), bytes);
        assert!(load(&bank, 1).is_err());
    }
    let error = Assets::from_source(
        &mut |name: &str| {
            if name == "VGASPEC0.DAT" {
                Err(std::io::Error::other("special read failed").into())
            } else {
                Ok(complete.get(name).cloned())
            }
        },
        1,
    )
    .err()
    .unwrap();
    assert_eq!(error.to_string(), "special read failed");
}
