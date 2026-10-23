//! Host-only protocol adapter for the pinned C buffered-register oracle.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::{Read, Write};
    let mut bytes = Vec::new();
    std::io::stdin().take(6_000_013).read_to_end(&mut bytes)?;
    let pcm = lemmings_audio_probe::opl_pcm(&bytes).map_err(std::io::Error::other)?;
    let mut out = std::io::BufWriter::new(std::io::stdout().lock());
    for sample in pcm {
        out.write_all(&sample.to_le_bytes())?;
    }
    Ok(())
}
