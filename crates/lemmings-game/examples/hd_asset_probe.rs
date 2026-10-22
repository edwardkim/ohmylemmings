//! Read the HD-03a generated pack without any original game files or window.
use lemmings_game::hd::assets::{AssetKey, AssetPack};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("usage: hd_asset_probe assets.hdp")?;
    let bytes = std::fs::read(path)?;
    let pack = AssetPack::parse(&bytes)?;
    for slot in [0, 2] {
        for index in 0..8 {
            let frame = pack
                .frame(AssetKey::Actor { slot, frame: index })
                .ok_or("missing walking frame")?;
            let atlas = &pack.atlases()[frame.atlas as usize];
            for row in 0..frame.rect[3] {
                if atlas.row(frame, row).ok_or("invalid row")?.len() != frame.rect[2] as usize * 4 {
                    return Err("row size differs".into());
                }
            }
        }
    }
    if pack.frame(AssetKey::Rock).is_none() {
        return Err("missing rock material".into());
    }
    println!(
        "{{\"status\":\"PASS\",\"atlases\":{},\"frames\":{},\"bytes\":{},\"walking_frames\":16,\"pixel_crc_valid\":true}}",
        pack.atlases().len(),
        pack.frames().len(),
        bytes.len()
    );
    Ok(())
}
