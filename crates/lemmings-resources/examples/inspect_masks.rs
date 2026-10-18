//! Export all original MAIN section-1 bitmaps, refusing to overwrite files.
use lemmings_resources::{MAIN_MASK_KINDS, MAIN_MASK_RECORD_SIZE, MainDatFile, MainMaskKind};
use std::{
    env,
    error::Error,
    fs::{self, OpenOptions},
    io::{self, BufWriter, Write},
    path::Path,
};
fn write_new(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let result = (|| {
        let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
        file.write_all(bytes)
    })();
    result.map_err(|e: io::Error| io::Error::new(e.kind(), format!("{}: {e}", path.display())))
}
fn data_error(path: &Path, error: impl std::fmt::Display) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("{}: {error}", path.display()),
    )
}
fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args_os().skip(1);
    let (Some(source), Some(destination), None) = (args.next(), args.next(), args.next()) else {
        return Err("usage: inspect_masks MAIN_DAT_FILE OUTPUT_DIRECTORY".into());
    };
    let source = Path::new(&source);
    let destination = Path::new(&destination);
    let bytes = fs::read(source)
        .map_err(|e| io::Error::new(e.kind(), format!("{}: {e}", source.display())))?;
    let file = MainDatFile::parse(&bytes).map_err(|e| data_error(source, e))?;
    let mut output = [0; MAIN_MASK_RECORD_SIZE];
    let masks = file
        .decode_masks_into(&mut output)
        .map_err(|e| data_error(source, e))?;
    fs::create_dir_all(destination)?;
    write_new(&destination.join("section-001.bin"), masks.as_bytes())?;
    let mut stdout = BufWriter::new(io::stdout().lock());
    for kind in MAIN_MASK_KINDS {
        let frames = masks.frames(kind);
        let info = frames.info();
        for index in 0..usize::from(info.frame_count) {
            let image = frames.frame(index)?;
            let mut bits = Vec::new();
            for y in 0..usize::from(image.height()) {
                for x in 0..usize::from(image.width()) {
                    bits.push(u8::from(image.at(x, y).expect("bounded coordinates")));
                }
            }
            let digit = if kind == MainMaskKind::Countdown {
                (9 - index).to_string()
            } else {
                "null".to_owned()
            };
            let name = format!("mask-{kind:?}-{index:02}.bits.bin");
            write_new(&destination.join(&name), &bits)?;
            writeln!(
                stdout,
                "{{\"kind\":\"{kind:?}\",\"index\":{index},\"offset\":{},\"width\":{},\"height\":{},\"digit\":{digit},\"bit_file\":\"{name}\",\"selected_bits\":{}}}",
                info.offset + index * info.frame_size(),
                info.width,
                info.height,
                bits.iter().filter(|&&b| b != 0).count(),
            )?;
        }
    }
    stdout.flush()?;
    Ok(())
}
