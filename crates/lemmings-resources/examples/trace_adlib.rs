//! Host-only JSONL trace. Input is a decoded image; actions are m0:100,e0:20,s:1.
use lemmings_resources::{AdlibImage, AdlibPlayer, AdlibWrite};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        return Err("usage: trace_adlib IMAGE m0:100,e0:20,s:1".into());
    }
    let data = std::fs::read(&args[1])?;
    let mut p = AdlibPlayer::new(AdlibImage::parse(&data)?);
    let mut out = std::io::BufWriter::new(std::io::stdout().lock());
    let mut writes = [AdlibWrite::default(); 4096];
    for action in args[2].split(',') {
        let (command, steps) = action.split_once(':').ok_or("missing step count")?;
        match command.as_bytes().first().copied().ok_or("empty action")? {
            b'm' => p.request_music(command[1..].parse()?)?,
            b'e' => p.request_effect(command[1..].parse()?)?,
            b's' => p.request_stop(),
            b'n' => (),
            _ => return Err("unknown action".into()),
        }
        for _ in 0..steps.parse::<usize>()? {
            let n = p.step(&mut writes, 4096)?;
            let s = p.state();
            write!(
                out,
                "{{\"step\":{},\"channels\":{:?},\"globals\":[{},{},{},{},{}],\"flags\":[{},{},{},{}],\"registers\":{:?},\"writes\":[",
                s.audio_step - 1,
                s.channels,
                s.sample_rate_factor,
                s.instrument_offset,
                s.wait_cycles,
                s.cycle,
                s.music_channels,
                s.pending_music,
                s.playing_music,
                s.deferred_music,
                s.pending_effect,
                s.registers
            )?;
            for (i, w) in writes[..n].iter().enumerate() {
                if i != 0 {
                    write!(out, ",")?;
                }
                write!(out, "[{},{}]", w.register, w.value)?;
            }
            writeln!(out, "]}}")?;
        }
    }
    Ok(())
}
