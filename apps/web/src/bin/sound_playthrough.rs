//! Device-free integrated peer; original inputs and PCM stay in ignored output.
use lemmings_web::{WebAssets, WebAudio, WebGame};
use std::{error::Error, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if !(1..=2).contains(&args.len()) {
        return Err("sound_playthrough NEW_OUTPUT [ORIGINAL_ASSETS]".into());
    }
    let root = PathBuf::from(&args[0]);
    fs::create_dir(&root)?;
    let mut files = WebAssets::new();
    if let Some(directory) = args.get(1) {
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if WebAssets::accepts(&name) {
                files.insert(&name, &fs::read(entry.path())?)?;
            }
        }
    }
    let mut a = WebGame::new("win", "dos")?;
    let mut b = WebGame::new("win", "dos")?;
    if args.len() == 2 {
        a.load_original(&files, 1.0)?;
        b.load_original(&files, 1.0)?;
    }
    let mut audio = WebAudio::new(44100.0)?;
    audio.set_volume(100.0)?;
    audio.prepare(&mut a, true)?;
    b.drain_sound_events();
    let mut rows = Vec::new();
    let mut pcm = Vec::new();
    for wall in 1_u64..=580 {
        for game in [&mut a, &mut b] {
            match wall {
                5 | 120 => game.command("Pause", 0.0)?,
                8 | 125 => game.command("Resume", 0.0)?,
                90 => {
                    game.key("KeyD", false)?;
                }
                106 => game.command("AssignDigger", 1.0)?,
                160 => game.command("Nuke", 0.0)?,
                360 => game.restart_level()?,
                _ => {}
            }
            game.step(1.0)?;
        }
        if wall == 100 || wall == 105 {
            audio.set_volume(if wall == 100 { 0.0 } else { 100.0 })?;
        }
        assert_eq!(a.state(), b.state());
        let state = a.state();
        let active = !(75..78).contains(&wall)
            && !state.contains("\"phase\":\"Won\"")
            && !state.contains("\"phase\":\"Lost\"");
        let flush = audio.prepare(&mut a, active)?;
        let end = wall * 44100 * 156250 / 2190197;
        let begin = (wall - 1) * 44100 * 156250 / 2190197;
        for sample in audio.render((end - begin) as f64)? {
            pcm.extend_from_slice(&sample.to_le_bytes());
        }
        let sounds = b.drain_sound_events();
        rows.push(format!("{{\"wall\":{wall},\"state\":{state},\"sounds\":{sounds},\"active\":{active},\"flush\":{flush},\"audio\":{},\"end\":{end}}}",audio.state()));
        if [55, 125, 359, 360, 418, 580].contains(&wall) {
            assert_eq!(a.frame()?, b.frame()?);
            assert_eq!(a.terrain(), b.terrain());
            assert_eq!(a.objects(), b.objects());
            for (ext, data) in [
                ("rgb", a.frame()?),
                ("terrain", a.terrain()),
                ("objects", a.objects()),
            ] {
                fs::write(root.join(format!("{wall}.{ext}")), data)?;
            }
        }
    }
    fs::write(root.join("native.pcm"), pcm)?;
    fs::write(root.join("trace.json"), format!("[{}]\n", rows.join(",")))?;
    Ok(())
}
