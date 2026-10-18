//! Host-side original-catalog inspection; output includes raw base bytes as hex.
use lemmings_core::{Difficulty, LevelId};
use lemmings_resources::{CatalogEntry, DosLevelFile, LEVEL_RECORD_SIZE, OddTable};
use std::{
    env,
    error::Error,
    fs,
    io::{self, BufWriter, Write},
    path::Path,
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args_os().skip(1);
    let (Some(directory), None) = (args.next(), args.next()) else {
        return Err("usage: inspect_catalog DOS_DATA_DIRECTORY".into());
    };
    let directory = Path::new(&directory);
    let odd_bytes = fs::read(directory.join("ODDTABLE.DAT"))?;
    let odd = OddTable::parse(&odd_bytes)?;
    let dat_bytes: Vec<_> = (0..10)
        .map(|index| fs::read(directory.join(format!("LEVEL{index:03}.DAT"))))
        .collect::<Result<_, _>>()?;
    let files: Vec<_> = dat_bytes
        .iter()
        .enumerate()
        .map(|(index, bytes)| DosLevelFile::parse(index as u8, bytes))
        .collect::<Result<_, _>>()?;
    let mut output = BufWriter::new(io::stdout().lock());
    for difficulty in [
        Difficulty::Fun,
        Difficulty::Tricky,
        Difficulty::Taxing,
        Difficulty::Mayhem,
    ] {
        for number in 1..=30 {
            let id = LevelId::new(difficulty, number)?;
            let entry = CatalogEntry::for_level(id);
            let mut bytes = [0; LEVEL_RECORD_SIZE];
            let selected = entry.load(
                &files[usize::from(entry.file_index())],
                Some(odd),
                &mut bytes,
            )?;
            let header = selected.header();
            write!(
                output,
                "{{\"global_index\":{},\"group\":\"{difficulty:?}\",\"number\":{number},\"file\":{},\"part\":{},\"odd_index\":",
                id.global_index(),
                entry.file_index(),
                entry.part_index()
            )?;
            match entry.odd_index() {
                Some(index) => write!(output, "{index}")?,
                None => write!(output, "null")?,
            }
            write!(
                output,
                ",\"release_rate\":{},\"total\":{},\"required\":{},\"minutes\":{},\"skills\":{:?},\"screen_x\":{},\"graphics\":{},\"special\":{},\"superlemming\":{},\"title_hex\":\"",
                header.release_rate,
                header.total,
                header.required,
                header.minutes,
                header.skills,
                header.screen_x,
                header.graphics,
                header.special,
                header.superlemming
            )?;
            for byte in selected.title_bytes() {
                write!(output, "{byte:02x}")?;
            }
            write!(output, "\",\"base_hex\":\"")?;
            for byte in selected.base_bytes() {
                write!(output, "{byte:02x}")?;
            }
            writeln!(
                output,
                "\",\"objects_count\":{},\"terrain_count\":{},\"steel_count\":{}}}",
                selected.objects().count(),
                selected.terrains().count(),
                selected.steel().count()
            )?;
        }
    }
    output.flush()?;
    Ok(())
}
