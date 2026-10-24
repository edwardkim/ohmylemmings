//! Offline, read-only dependency-plan checks against a supplied original bank.
use lemmings_web::{WebAssets, WebGame};
use std::{error::Error, path::Path};

fn main() -> Result<(), Box<dyn Error>> {
    let directory = std::env::args()
        .nth(1)
        .ok_or("usage: hd_download_probe GAME_DATA_DIRECTORY")?;
    let mut bank = WebAssets::new();
    for entry in std::fs::read_dir(Path::new(&directory))? {
        let path = entry?.path();
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if WebAssets::accepts(name) {
            bank.insert(name, &std::fs::read(&path)?)?;
        }
    }
    let mut game = WebGame::new("builder", "reference")?;
    let mut next_game = WebGame::new("builder", "reference")?;
    let ranks = ["fun", "tricky", "taxing", "mayhem"];
    for (rank_index, rank) in ranks.iter().enumerate() {
        for level in 1..=30 {
            game.load_original_rank(&bank, rank, f64::from(level))?;
            let before = (game.state(), game.sound_state(), game.input_log());
            let current = game.hd_download_plan(false)?;
            let next = game.hd_download_plan(true)?;
            assert_eq!((game.state(), game.sound_state(), game.input_log()), before);
            if rank_index == 3 && level == 30 {
                assert_eq!(next, "null");
            } else {
                let (next_rank, next_level) = if level == 30 {
                    (ranks[rank_index + 1], 1)
                } else {
                    (*rank, level + 1)
                };
                next_game.load_original_rank(&bank, next_rank, f64::from(next_level))?;
                assert_eq!(
                    next,
                    next_game.hd_download_plan(false)?,
                    "next plan {rank} {level}"
                );
            }
            println!(
                "{{\"rank\":\"{rank}\",\"level\":{level},\"scope\":{},\"plan\":{current}}}",
                game.hd_scope()?
            );
        }
    }
    Ok(())
}
