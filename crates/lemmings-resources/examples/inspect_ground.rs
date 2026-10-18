//! Inspect raw GROUND metadata and all palette banks as JSONL.
use lemmings_resources::{GroundPalette, GroundRecord};
use std::{
    env,
    error::Error,
    fs,
    io::{self, BufWriter, Write},
    path::Path,
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut paths = env::args_os().skip(1).peekable();
    if paths.peek().is_none() {
        return Err("usage: inspect_ground GROUNDxO.DAT [GROUNDxO.DAT ...]".into());
    }
    let mut output = BufWriter::new(io::stdout().lock());
    for (file_index, path) in paths.enumerate() {
        let path = Path::new(&path);
        let bytes = fs::read(path)?;
        let ground = GroundRecord::parse(&bytes).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{}: {error}", path.display()),
            )
        })?;
        write!(output, "{{\"file_index\":{file_index},\"objects\":[")?;
        for (i, o) in ground.objects().enumerate() {
            if i != 0 {
                write!(output, ",")?;
            }
            let t = o.trigger_rect();
            write!(
                output,
                "{{\"fields\":{:?},\"empty\":{},\"loops\":{},\"trigger\":[{},{},{},{}]}}",
                [
                    o.slot as u16,
                    o.animation_flags,
                    u16::from(o.first_frame),
                    u16::from(o.frame_count),
                    u16::from(o.width),
                    u16::from(o.height),
                    o.frame_size,
                    o.mask_offset,
                    o.unknown[0],
                    o.unknown[1],
                    o.trigger_left,
                    o.trigger_top,
                    u16::from(o.trigger_width),
                    u16::from(o.trigger_height),
                    u16::from(o.trigger_effect),
                    o.image_offset,
                    o.preview_offset,
                    o.unknown[2],
                    u16::from(o.sound_effect),
                ],
                o.is_empty(),
                o.animation_loops(),
                t.left,
                t.top,
                t.width,
                t.height,
            )?;
        }
        write!(output, "],\"terrain\":[")?;
        for (i, t) in ground.terrains().enumerate() {
            if i != 0 {
                write!(output, ",")?;
            }
            write!(
                output,
                "{{\"fields\":[{},{},{},{},{},{}],\"empty\":{}}}",
                t.slot,
                t.width,
                t.height,
                t.image_offset,
                t.mask_offset,
                t.unknown,
                t.is_empty(),
            )?;
        }
        write!(output, "],\"palettes\":[")?;
        for (i, bank) in [
            GroundPalette::Custom,
            GroundPalette::Standard,
            GroundPalette::Preview,
        ]
        .into_iter()
        .enumerate()
        {
            if i != 0 {
                write!(output, ",")?;
            }
            let vga = ground.vga_palette(bank);
            write!(
                output,
                "{{\"ega\":{:?},\"vga\":{:?},\"shifted\":{:?},\"full_range\":{:?}}}",
                ground.ega_palette(bank),
                vga.map(|c| c.components()),
                vga.map(|c| c.to_rgb8_shifted()),
                vga.map(|c| c.to_rgb8_full_range()),
            )?;
        }
        writeln!(output, "]}}")?;
    }
    output.flush()?;
    Ok(())
}
