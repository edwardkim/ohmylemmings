fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;
    let a: Vec<_> = std::env::args().collect();
    if a.len() != 8 {
        return Err("usage: audio-probe IMAGE RATE NUM DEN ACTIONS PARTITIONS pcm|direct".into());
    }
    let data = std::fs::read(&a[1])?;
    let rate = a[2].parse()?;
    let num = a[3].parse()?;
    let den = a[4].parse()?;
    let partitions: Vec<u32> = a[6].split(',').map(str::parse).collect::<Result<_, _>>()?;
    let pcm = match a[7].as_str() {
        "pcm" => lemmings_audio_probe::render(&data, rate, num, den, &a[5], &partitions),
        "direct" => lemmings_audio_probe::direct(&data, rate, num, den, &a[5]),
        _ => Err("unknown renderer".into()),
    }
    .map_err(std::io::Error::other)?;
    let mut out = std::io::BufWriter::new(std::io::stdout().lock());
    for s in pcm {
        out.write_all(&s.to_le_bytes())?;
    }
    Ok(())
}
