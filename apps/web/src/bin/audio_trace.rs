//! Native peer for the actual-WASM device-output verifier; synthetic by default.
use lemmings_web::{WebAssets, WebAudio, WebGame};
use std::{error::Error, fs, path::PathBuf};
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if !(1..=2).contains(&args.len()) {
        return Err("audio_trace NEW_OUTPUT [ORIGINAL_ASSETS]".into());
    }
    let root = PathBuf::from(&args[0]);
    fs::create_dir_all(&root)?;
    let mut bank = WebAssets::new();
    if let Some(directory) = args.get(1) {
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if WebAssets::accepts(&name) {
                bank.insert(&name, &fs::read(entry.path())?)?;
            }
        }
    }
    let mut rows = Vec::new();
    for rate in [44100, 48000] {
        for volume in [0, 33, 100] {
            let name = format!("{rate}-{volume}");
            let mut a = WebGame::new("builder", "reference")?;
            let mut b = WebGame::new("builder", "reference")?;
            if args.len() == 2 {
                a.load_original(&bank, 1.0)?;
                b.load_original(&bank, 1.0)?;
            }
            let mut output = WebAudio::new(f64::from(rate))?;
            output.set_volume(f64::from(volume))?;
            let mut flushes = u32::from(output.prepare(&mut a, true)?);
            let mut pcm = Vec::new();
            for step in 1..=80 {
                for game in [&mut a, &mut b] {
                    match step {
                        5 => game.command("Pause", 0.0)?,
                        8 => game.command("Resume", 0.0)?,
                        16 => {
                            game.key("KeyD", false)?;
                        }
                        17 => {
                            game.key("KeyD", true)?;
                        }
                        78 => game.restart_level()?,
                        _ => {}
                    }
                    game.step(1.0)?;
                }
                flushes += u32::from(output.prepare(&mut a, !(65..68).contains(&step))?);
                let n = [1, 127, 512, 3, 1024, 7, 2048][(step - 1) % 7];
                for sample in output.render(n as f64)? {
                    pcm.extend_from_slice(&sample.to_le_bytes());
                }
                assert_eq!(a.state(), b.state());
                assert_eq!(a.terrain(), b.terrain());
            }
            assert_eq!(a.frame()?, b.frame()?);
            for (extension, bytes) in [
                ("pcm", pcm),
                ("rgb", a.frame()?),
                ("terrain", a.terrain()),
                ("objects", a.objects()),
            ] {
                fs::write(root.join(format!("{name}.{extension}")), bytes)?;
            }
            rows.push(format!("{{\"name\":{name:?},\"rate\":{rate},\"volume\":{volume},\"flushes\":{flushes},\"state\":{},\"audio\":{}}}",a.state(),output.state()));
        }
    }
    fs::write(
        root.join("native.json"),
        format!("{{\"cases\":[{}]}}\n", rows.join(",")),
    )?;
    Ok(())
}
