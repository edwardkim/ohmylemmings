//! Inspect a generated HD pack without original game files or a window.
use lemmings_game::hd::assets::{AssetKey, AssetPack, ICON_CONTROLS, object_frame_count};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("usage: hd_asset_probe assets.hdp [--hd03b|--all-actors|--all-portals]")?;
    let mode = std::env::args().nth(2);
    let expanded = matches!(
        mode.as_deref(),
        Some("--hd03b" | "--all-actors" | "--all-portals")
    );
    let bytes = std::fs::read(path)?;
    let pack = AssetPack::parse(&bytes)?;
    let required: &[(u8, u16)] = if expanded {
        &[
            (0, 8),
            (2, 8),
            (4, 16),
            (10, 16),
            (11, 16),
            (26, 16),
            (27, 1),
        ]
    } else {
        &[(0, 8), (2, 8)]
    };
    if matches!(mode.as_deref(), Some("--all-actors" | "--all-portals"))
        && !pack.missing_actor_frames().is_empty()
    {
        return Err("incomplete actor coverage".into());
    }
    for &(slot, count) in required {
        for frame in 0..count {
            pack.frame(AssetKey::Actor { slot, frame })
                .ok_or("missing required actor frame")?;
        }
    }
    if mode.as_deref() == Some("--all-portals") && !pack.missing_portal_frames().is_empty() {
        return Err("incomplete portal coverage".into());
    }
    if mode.as_deref() == Some("--all-portals") && !pack.missing_exit_effect_frames().is_empty() {
        return Err("incomplete exit effect coverage".into());
    }
    if pack.frame(AssetKey::Rock).is_none() {
        return Err("missing rock material".into());
    }
    if expanded {
        for id in [0, 1] {
            for frame in 0..object_frame_count(0, id).ok_or("missing object metadata")? {
                pack.frame(AssetKey::Object { set: 0, id, frame })
                    .ok_or("missing facility frame")?;
            }
        }
        for control in ICON_CONTROLS {
            pack.frame(AssetKey::Icon(control))
                .ok_or("missing toolbar icon")?;
        }
    }
    let (mut actors, mut objects, mut icons) = (0, 0, 0);
    for frame in pack.frames() {
        match frame.key {
            AssetKey::Actor { .. } => actors += 1,
            AssetKey::Object { .. } => objects += 1,
            AssetKey::Icon(_) => icons += 1,
            AssetKey::Rock | AssetKey::Material { .. } | AssetKey::Special { .. } => (),
        }
        let atlas = &pack.atlases()[frame.atlas as usize];
        for row in 0..frame.rect[3] {
            if atlas.row(frame, row).ok_or("invalid row")?.len() != frame.rect[2] as usize * 4 {
                return Err("row size differs".into());
            }
        }
    }
    println!(
        "{{\"status\":\"PASS\",\"atlases\":{},\"frames\":{},\"bytes\":{},\"actor_frames\":{actors},\"object_frames\":{objects},\"icons\":{icons},\"pixel_crc_valid\":true}}",
        pack.atlases().len(),
        pack.frames().len(),
        bytes.len()
    );
    Ok(())
}
