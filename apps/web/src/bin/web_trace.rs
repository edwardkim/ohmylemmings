//! Native diagnostic producer for browser equivalence. Optional local bytes stay local.
use lemmings_web::{WebAssets, WebGame};
use std::io::{self, Read, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if !(4..=6).contains(&args.len()) {
        return Err("expected scene, pacing, maximum steps [--interactive | asset directory --interactive]; stdin: step operation arguments".into());
    }
    let mut game = WebGame::new(
        if args[1].starts_with("original:") {
            "builder"
        } else {
            &args[1]
        },
        &args[2],
    )?;
    if let Some(level) = args[1].strip_prefix("original:") {
        let directory = std::path::Path::new(args.get(4).ok_or("asset directory required")?);
        let mut bank = WebAssets::new();
        for entry in std::fs::read_dir(directory)? {
            let entry = entry?;
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| "invalid filename")?;
            if WebAssets::accepts(&name) {
                bank.insert(&name, &std::fs::read(entry.path())?)?;
            }
        }
        game.load_original(&bank, level.parse()?)?;
    }
    let interactive = args.iter().any(|arg| arg == "--interactive");
    let steps: u32 = args[3].parse()?;
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let commands: Vec<_> = input
        .lines()
        .map(|line| {
            let words: Vec<_> = line.split_whitespace().collect();
            if words.len() < 2 {
                return Err("expected step operation".into());
            }
            Ok((words[0].parse::<u32>()?, words[1..].to_vec()))
        })
        .collect::<Result<_, Box<dyn std::error::Error>>>()?;
    let mut out = io::BufWriter::new(io::stdout().lock());
    for step in 0..=steps {
        for (_, words) in commands.iter().filter(|c| c.0 == step) {
            match words.as_slice() {
                ["Key", code, repeat] => {
                    game.key(code, *repeat == "1")?;
                }
                ["Pointer", kind, x, y, button, buttons, clicks] => {
                    game.pointer(
                        kind,
                        x.parse()?,
                        y.parse()?,
                        button.parse()?,
                        buttons.parse()?,
                        clicks.parse()?,
                    )?;
                }
                ["Release"] => game.release_input(),
                ["Restart"] => game.restart_level()?,
                ["Continue"] => {
                    game.transition("continue")?;
                }
                ["Level", number] => game.start_level(number.parse()?)?,
                [name, value] => game.command(name, value.parse()?)?,
                _ => return Err("invalid diagnostic operation".into()),
            }
        }
        let state = format!(
            "{{\"state\":{},\"view\":{}}}",
            game.state(),
            game.view_state()
        );
        for bytes in [
            state.as_bytes().to_vec(),
            game.terrain(),
            game.bricks(),
            game.objects(),
            game.frame()?,
        ] {
            out.write_all(&u32::try_from(bytes.len())?.to_le_bytes())?;
            out.write_all(&bytes)?;
        }
        if step == steps
            || (!interactive
                && (state.contains("\"phase\":\"Won\"") || state.contains("\"phase\":\"Lost\"")))
        {
            break;
        }
        game.step(1.0)?;
    }
    out.flush()?;
    Ok(())
}
