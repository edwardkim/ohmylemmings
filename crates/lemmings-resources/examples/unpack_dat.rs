//! Host-only example. Library decoding uses caller-owned memory and no I/O.
use lemmings_resources::DatSections;
use std::{
    env,
    error::Error,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::Path,
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args_os().skip(1);
    let (Some(input), Some(destination), None) = (args.next(), args.next(), args.next()) else {
        return Err("usage: unpack_dat INPUT.DAT OUTPUT_DIRECTORY".into());
    };
    let input = Path::new(&input);
    let bytes = fs::read(input)?;
    let destination = Path::new(&destination);
    fs::create_dir_all(destination)?;
    let contextualize = |error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{}: {error}", input.display()),
        )
    };
    let mut stdout = io::stdout().lock();
    for (index, section) in DatSections::new(&bytes).enumerate() {
        let section = section.map_err(contextualize)?;
        let mut output = vec![0; usize::from(section.header().decompressed_size)];
        section.decompress_into(&mut output).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{}: {error}", input.display()),
            )
        })?;
        let path = destination.join(format!("part-{index:03}.bin"));
        // Refuse to overwrite existing output, including a previous partial run.
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        file.write_all(&output)?;
        writeln!(stdout, "{index}\t{}\t{}", output.len(), path.display())?;
    }
    Ok(())
}
