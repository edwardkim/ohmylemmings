//! Native diagnostic producer for the browser equivalence check. No original data.
use lemmings_web::WebGame;
use std::io::{self, Read, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 4 {
        return Err("expected scene, pacing, maximum steps; stdin: step command argument".into());
    }
    let mut game = WebGame::new(&args[1], &args[2])?;
    let steps: u32 = args[3].parse()?;
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let commands: Vec<_> = input
        .lines()
        .map(|line| {
            let words: Vec<_> = line.split_whitespace().collect();
            if words.len() != 3 {
                return Err("expected step command argument".into());
            }
            Ok((
                words[0].parse::<u32>()?,
                words[1].to_owned(),
                words[2].parse::<f64>()?,
            ))
        })
        .collect::<Result<_, Box<dyn std::error::Error>>>()?;
    let mut out = io::BufWriter::new(io::stdout().lock());
    for step in 0..=steps {
        for (_, name, value) in commands.iter().filter(|c| c.0 == step) {
            game.command(name, *value)?;
        }
        let state = game.state();
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
        if state.contains("\"phase\":\"Won\"")
            || state.contains("\"phase\":\"Lost\"")
            || step == steps
        {
            break;
        }
        game.step(1.0)?;
    }
    out.flush()?;
    Ok(())
}
