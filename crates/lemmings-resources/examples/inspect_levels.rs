//! Inspect all base-level sections in one or more LEVELxxx.DAT files as JSONL.
use lemmings_resources::{DatSections, LEVEL_RECORD_SIZE, LevelRecord};
use std::{
    env,
    error::Error,
    fmt::Display,
    fs,
    io::{self, BufWriter, Write},
    path::Path,
};

fn context(path: &Path, part: usize, error: impl Display) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("{} section {part}: {error}", path.display()),
    )
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut paths = env::args_os().skip(1).peekable();
    if paths.peek().is_none() {
        return Err("usage: inspect_levels LEVELxxx.DAT [LEVELxxx.DAT ...]".into());
    }
    let mut output = BufWriter::new(io::stdout().lock());
    for (file_index, path) in paths.enumerate() {
        let path = Path::new(&path);
        let bytes = fs::read(path)?;
        for (part, section) in DatSections::new(&bytes).enumerate() {
            let section = section.map_err(|error| context(path, part, error))?;
            let mut decoded = [0; LEVEL_RECORD_SIZE];
            section
                .decompress_into(&mut decoded)
                .map_err(|error| context(path, part, error))?;
            let level = LevelRecord::parse(&decoded).map_err(|error| context(path, part, error))?;
            let h = level.header();
            write!(
                output,
                "{{\"file_index\":{file_index},\"part\":{part},\"dat_offset\":{},\"release_rate\":{},\"total\":{},\"required\":{},\"minutes\":{},\"skills\":{:?},\"screen_x\":{},\"graphics\":{},\"special\":{},\"superlemming\":{},\"title_hex\":\"",
                section.offset(),
                h.release_rate,
                h.total,
                h.required,
                h.minutes,
                h.skills,
                h.screen_x,
                h.graphics,
                h.special,
                h.superlemming,
            )?;
            for byte in level.title_bytes() {
                write!(output, "{byte:02x}")?;
            }
            write!(output, "\",\"objects\":[")?;
            for (index, object) in level.objects().enumerate() {
                if index != 0 {
                    write!(output, ",")?;
                }
                write!(
                    output,
                    "[{},{},{},{},{},{},{}]",
                    object.slot,
                    object.x,
                    object.y,
                    object.id,
                    object.flags.upside_down,
                    object.flags.no_overwrite,
                    object.flags.only_on_terrain
                )?;
            }
            write!(output, "],\"terrain\":[")?;
            for (index, terrain) in level.terrains().enumerate() {
                if index != 0 {
                    write!(output, ",")?;
                }
                write!(
                    output,
                    "[{},{},{},{},{},{},{}]",
                    terrain.slot,
                    terrain.x,
                    terrain.y,
                    terrain.id,
                    terrain.flags.upside_down,
                    terrain.flags.no_overwrite,
                    terrain.flags.erase
                )?;
            }
            write!(output, "],\"steel\":[")?;
            for (index, area) in level.steel().enumerate() {
                if index != 0 {
                    write!(output, ",")?;
                }
                write!(
                    output,
                    "[{},{},{},{},{}]",
                    area.slot, area.x, area.y, area.width, area.height
                )?;
            }
            writeln!(output, "]}}")?;
        }
    }
    output.flush()?;
    Ok(())
}
