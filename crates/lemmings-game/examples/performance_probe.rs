//! Measurement adapter only. Hashing, dumps, initialization and I/O are outside hot windows.
use lemmings_audio::{AdlibPcm, AudioRate};
use lemmings_core::{Difficulty, InputCommand, LevelId, StartupCommand, TimedInput};
use lemmings_game::{Assets, LevelRun, diagnostics, pacing::Pacing};
use lemmings_resources::{AdlibImage, AdlibWrite};
use std::{collections::BTreeMap, error::Error, fs, hint::black_box, path::Path, time::Instant};
type Result<T> = std::result::Result<T, Box<dyn Error>>;
type Inputs = Vec<TimedInput<StartupCommand>>;
type Row = (u128, usize, String, u128);
fn hash(parts: &[&[u8]]) -> String {
    let mut h = 0xcbf29ce484222325u64;
    for p in parts {
        for &b in *p {
            h = (h ^ u64::from(b)).wrapping_mul(0x100000001b3);
        }
    }
    format!("{h:016x}")
}
fn parse_inputs(text: &str, end: u64) -> Result<Inputs> {
    let mut out = Vec::new();
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let w: Vec<_> = line.split_whitespace().collect();
        if w.len() < 3 {
            return Err("short input".into());
        }
        let step: u64 = w[0].parse()?;
        let seq: u32 = w[1].parse()?;
        if step >= end
            || out.last().is_some_and(|p: &TimedInput<StartupCommand>| {
                (p.control_step, p.sequence) >= (step, seq)
            })
        {
            return Err("input order/range".into());
        }
        let command = match w[2..] {
            ["Pause"] => InputCommand::Pause,
            ["Resume"] => InputCommand::Resume,
            ["Nuke"] => InputCommand::Custom(StartupCommand::Nuke),
            [name, value] => InputCommand::Custom(match name {
                "SetReleaseRate" => StartupCommand::SetReleaseRate(value.parse()?),
                "AssignDigger" => StartupCommand::AssignDigger(value.parse()?),
                "AssignBasher" => StartupCommand::AssignBasher(value.parse()?),
                "AssignMiner" => StartupCommand::AssignMiner(value.parse()?),
                "AssignBuilder" => StartupCommand::AssignBuilder(value.parse()?),
                "AssignBlocker" => StartupCommand::AssignBlocker(value.parse()?),
                "AssignBomber" => StartupCommand::AssignBomber(value.parse()?),
                "AssignClimber" => StartupCommand::AssignClimber(value.parse()?),
                "AssignFloater" => StartupCommand::AssignFloater(value.parse()?),
                _ => return Err("unknown command".into()),
            }),
            _ => return Err("invalid command arity".into()),
        };
        out.push(TimedInput {
            control_step: step,
            sequence: seq,
            command,
        });
    }
    Ok(out)
}
fn make_synthetic(kind: &str) -> Result<LevelRun> {
    let assets = if kind == "builder" {
        Assets::synthetic_builder()
    } else {
        Assets::synthetic_completion(kind)
    };
    LevelRun::new(assets, 1, None, Pacing::Dos)
}
fn advance(g: &mut LevelRun, inputs: &[TimedInput<StartupCommand>], count: u64) -> Result<()> {
    let start = g.session.scheduler.step_state().control_step;
    for _ in 0..count {
        if g.session.result().is_some() {
            return Err("frozen work is not a tick sample".into());
        }
        let step = g.session.scheduler.step_state().control_step;
        g.session
            .inputs
            .extend(
                inputs
                    .iter()
                    .filter(|i| i.control_step == step)
                    .map(|i| TimedInput {
                        control_step: i.control_step,
                        sequence: i.sequence,
                        command: match &i.command {
                            InputCommand::Pause => InputCommand::Pause,
                            InputCommand::Resume => InputCommand::Resume,
                            InputCommand::Custom(c) => InputCommand::Custom(*c),
                        },
                    }),
            );
        g.tick()?;
    }
    if g.session.scheduler.step_state().control_step != start + count {
        return Err("missing logical ticks".into());
    }
    Ok(())
}
fn rss() -> String {
    let s = fs::read_to_string("/proc/self/status").unwrap_or_default();
    let get = |name: &str| {
        s.lines()
            .find(|l| l.starts_with(name))
            .and_then(|l| l.split_whitespace().nth(1))
            .unwrap_or("0")
    };
    format!(
        "\"rss_kib\":{},\"process_peak_rss_kib\":{}",
        get("VmRSS:"),
        get("VmHWM:")
    )
}
fn emit(kind: &str, id: &str, ops: usize, rows: &[Row], extra: &str) {
    let rows = rows
        .iter()
        .map(|(n, o, h, i)| format!("[{n},{o},{h:?},{i}]"))
        .collect::<Vec<_>>()
        .join(",");
    println!(
        "{{\"kind\":{kind:?},\"id\":{id:?},\"operations\":{ops},\"samples\":[{rows}],{}, {extra}}}",
        rss()
    );
}
fn bank(path: &Path) -> Result<BTreeMap<String, Vec<u8>>> {
    let mut b = BTreeMap::new();
    for entry in fs::read_dir(path)? {
        let p = entry?.path();
        if p.is_file() {
            b.insert(
                p.file_name().unwrap().to_string_lossy().into(),
                fs::read(p)?,
            );
        }
    }
    Ok(b)
}
fn load(b: &BTreeMap<String, Vec<u8>>, rank: &str, n: u8) -> Result<Assets> {
    Assets::from_level_source(
        &mut |name: &str| Ok(b.get(name).cloned()),
        LevelId::new(rank.parse::<Difficulty>()?, n)?,
    )
}
fn state_hash(g: &LevelRun) -> String {
    hash(&[
        diagnostics::state(&g.session).as_bytes(),
        g.session.terrain(),
        g.session.brick_mask(),
        g.session.object_map_data(),
    ])
}
fn dump(g: &mut LevelRun, output: &Path) -> Result<()> {
    fs::create_dir(output)?;
    fs::write(output.join("state.json"), diagnostics::state(&g.session))?;
    fs::write(output.join("terrain.bin"), g.session.terrain())?;
    fs::write(output.join("bricks.bin"), g.session.brick_mask())?;
    fs::write(output.join("objects.bin"), g.session.object_map_data())?;
    fs::write(
        output.join("frame.rgb"),
        g.view.frame(&g.assets, &g.session)?,
    )?;
    Ok(())
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 4 {
        return Err(
            "performance_probe ASSET_DIR|synthetic CASES.tsv|read OUTPUT_DIR full|smoke".into(),
        );
    }
    let original = args[0] != "synthetic";
    let full = args[3] != "smoke";
    if !matches!(args[3].as_str(), "full" | "smoke" | "scene") {
        return Err("invalid mode".into());
    }
    let out = Path::new(&args[2]);
    fs::create_dir(out)?;
    if args[1] == "read" {
        let start = Instant::now();
        let bytes = bank(Path::new(&args[0]))?;
        let ns = start.elapsed().as_nanos();
        let size = bytes.values().map(Vec::len).sum::<usize>();
        let parts = bytes.values().map(Vec::as_slice).collect::<Vec<_>>();
        println!(
            "{{\"kind\":\"file_read\",\"ns\":{ns},\"bytes\":{size},\"files\":{},\"digest\":{:?},{}}}",
            bytes.len(),
            hash(&parts),
            rss()
        );
        return Ok(());
    }
    let b = if original {
        bank(Path::new(&args[0]))?
    } else {
        BTreeMap::new()
    };
    // Full memory load inventory. Owned byte clones are part of the public AssetSource API.
    let ids = if original {
        ["fun", "tricky", "taxing", "mayhem"]
            .into_iter()
            .flat_map(|r| (1..=30).map(move |n| (r, n)))
            .collect::<Vec<_>>()
    } else {
        vec![("synthetic", 1)]
    };
    for (rank, n) in ids.into_iter().filter(|_| args[3] != "scene") {
        let id = format!("{rank}-{n:02}");
        let mut rows = Vec::new();
        for _ in 0..33 {
            let start = Instant::now();
            let a = if original {
                load(&b, rank, n)?
            } else {
                Assets::synthetic_builder()
            };
            let ns = start.elapsed().as_nanos();
            black_box(&a);
            let info = diagnostics::asset_info(&a);
            let h = hash(&[info.as_bytes(), &a.terrain, &a.object_map, &a.steel]);
            rows.push((ns, 1, h, 0));
        }
        emit(
            "memory_load",
            &id,
            1,
            &rows,
            "\"timing\":\"owned-memory-bytes-to-assets; no I/O\"",
        );
    }
    let plan = fs::read_to_string(&args[1])?;
    for line in plan.lines().filter(|l| !l.trim().is_empty()) {
        let w: Vec<_> = line.split_whitespace().collect();
        if w.len() != 5 {
            return Err("case schema".into());
        }
        let start: u64 = w[2].parse()?;
        let count: u64 = w[3].parse()?;
        if count == 0 || start.checked_add(count).is_none_or(|end| end > 100000) {
            return Err("case range".into());
        }
        let text = if w[4] == "-" {
            String::new()
        } else {
            fs::read_to_string(w[4])?
        };
        let inputs = parse_inputs(&text, 100001)?;
        let id = format!("{}-{:02}", w[0], w[1].parse::<u8>()?);
        let mut tick_rows = Vec::new();
        let mut render_rows = Vec::new();
        let frames = if full { 32 } else { 4 };
        let mut final_game = None;
        for _ in 0..33 {
            let init = Instant::now();
            let mut g = if original {
                LevelRun::new_for_level(
                    load(&b, w[0], w[1].parse()?)?,
                    LevelId::new(w[0].parse()?, w[1].parse()?)?,
                    LevelId::new(w[0].parse()?, w[1].parse()?)?.next(),
                    Pacing::Dos,
                )?
            } else {
                make_synthetic("builder")?
            };
            advance(&mut g, &inputs, start)?;
            let init_ns = init.elapsed().as_nanos();
            let now = Instant::now();
            advance(&mut g, &inputs, count)?;
            let ns = now.elapsed().as_nanos();
            tick_rows.push((ns, count as usize, state_hash(&g), init_ns));
            let mut rgb = Vec::new();
            let now = Instant::now();
            for _ in 0..frames {
                rgb = g.view.frame(&g.assets, &g.session)?;
                black_box(&rgb);
            }
            let ns = now.elapsed().as_nanos();
            render_rows.push((ns, frames, hash(&[&rgb]), 0));
            final_game = Some(g);
        }
        emit(
            "tick",
            &id,
            count as usize,
            &tick_rows,
            "\"timing\":\"LevelRun ticks plus ordered input dispatch; no render/PCM/I/O\"",
        );
        emit(
            "cpu_frame",
            &id,
            frames,
            &render_rows,
            "\"width\":320,\"height\":200,\"format\":\"RGB24\"",
        );
        dump(&mut final_game.unwrap(), &out.join(&id))?;
    }
    if args[3] == "scene" {
        return Ok(());
    }
    let image = if original {
        lemmings_game::audio::decode_adlib(b.get("ADLIB.DAT").ok_or("ADLIB missing")?)?
    } else {
        lemmings_game::audio::demo_image()
    };
    fs::write(out.join("adlib-image.bin"), &image)?;
    let frames = if full { 44100 } else { 4096 };
    let mut sounds = vec![("combined".to_owned(), Some(0), (0..18).collect::<Vec<_>>())];
    for i in 0..if full { 21 } else { 1 } {
        sounds.push((format!("music-{i:02}"), Some(i), vec![]));
    }
    for i in 0..if full { 18 } else { 1 } {
        sounds.push((format!("effect-{i:02}"), None, vec![i]));
    }
    for (name, music, effects) in sounds {
        let mut rows = Vec::new();
        let mut pcm = vec![0i16; frames];
        let mut last = Vec::new();
        for _ in 0..33 {
            let now = Instant::now();
            let mut engine = AdlibPcm::new(AdlibImage::parse(&image)?, 44100, AudioRate::DOS_VGA)?;
            if let Some(i) = music {
                engine.request_music(i)?;
            }
            for &i in &effects {
                engine.request_effect(i)?;
            }
            let mut scratch = [AdlibWrite::default(); 4096];
            let init_ns = now.elapsed().as_nanos();
            let now = Instant::now();
            for chunk in pcm.chunks_mut(512) {
                engine.render(chunk, &mut scratch, 4096)?;
            }
            let ns = now.elapsed().as_nanos();
            last = pcm.iter().flat_map(|s| s.to_le_bytes()).collect();
            rows.push((ns, frames, hash(&[&last]), init_ns));
        }
        fs::write(out.join(format!("{name}.pcm")), last)?;
        emit(
            "pcm",
            &name,
            frames,
            &rows,
            "\"sample_rate\":44100,\"channels\":1,\"chunk\":512",
        );
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ordered_inputs_reject_duplicate_boundaries_unknown_skills_and_past_end() {
        assert!(parse_inputs("60 0 AssignDigger 0\n", 100).is_ok());
        for text in ["60 0 Pause\n60 0 Resume\n", "0 0 Cheat\n", "100 0 Nuke\n"] {
            assert!(parse_inputs(text, 100).is_err());
        }
    }
    #[test]
    fn completed_segments_cannot_be_counted_as_simulated_ticks() {
        let mut game = make_synthetic("zero").unwrap();
        assert!(advance(&mut game, &[], 100).is_err());
    }
}
