//! Host-side inspection only; file I/O stays outside the no_std library.
use lemmings_resources::DatSections;
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
        return Err("usage: inspect_dat FILE.DAT [FILE.DAT ...]".into());
    }

    let mut output = BufWriter::new(io::stdout().lock());
    writeln!(
        output,
        "file\tpart\toffset\tinitial_bits\tchecksum\tdecompressed_size\tsection_size\tpayload_size"
    )?;
    for path in paths {
        let path = Path::new(&path);
        let bytes = fs::read(path)?;
        for (part, section) in DatSections::new(&bytes).enumerate() {
            let section = section.map_err(|error| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("{}: {error}", path.display()),
                )
            })?;
            let header = section.header();
            writeln!(
                output,
                "{}\t{part}\t{}\t{}\t{}\t{}\t{}\t{}",
                path.display(),
                section.offset(),
                header.initial_bits,
                header.checksum,
                header.decompressed_size,
                header.section_size,
                section.payload().len(),
            )?;
        }
    }
    output.flush()?;
    Ok(())
}
