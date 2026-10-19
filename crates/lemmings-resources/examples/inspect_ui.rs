//! Export all supported original MAIN.DAT UI images; refuse to overwrite files.
use lemmings_resources::{MAIN_UI_IMAGE_KINDS, MainDatFile, MainUiImageKind, MainUiSection};
use std::{
    env,
    error::Error,
    fs::{self, OpenOptions},
    io::{self, BufWriter, Write},
    path::Path,
};
fn data_error(path: &Path, error: impl std::fmt::Display) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("{}: {error}", path.display()),
    )
}
fn write_new(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let result = (|| {
        let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
        file.write_all(bytes)
    })();
    result.map_err(|e: io::Error| io::Error::new(e.kind(), format!("{}: {e}", path.display())))
}
fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args_os().skip(1);
    let (Some(source), Some(destination), None) = (args.next(), args.next(), args.next()) else {
        return Err("usage: inspect_ui MAIN_DAT_FILE OUTPUT_DIRECTORY".into());
    };
    let source = Path::new(&source);
    let destination = Path::new(&destination);
    let bytes = fs::read(source)
        .map_err(|e| io::Error::new(e.kind(), format!("{}: {e}", source.display())))?;
    let file = MainDatFile::parse(&bytes).map_err(|e| data_error(source, e))?;
    let mut sections = MainUiSection::ALL.map(|section| vec![0; section.decoded_size()]);
    let mut records = Vec::new();
    for (section, output) in MainUiSection::ALL.into_iter().zip(&mut sections) {
        records.push(
            file.decode_ui_into(section, output)
                .map_err(|e| data_error(source, e))?,
        );
    }
    fs::create_dir_all(destination)?;
    for record in &records {
        write_new(
            &destination.join(format!("section-{:03}.bin", record.section().part_index())),
            record.as_bytes(),
        )?;
    }
    let mut stdout = BufWriter::new(io::stdout().lock());
    for kind in MAIN_UI_IMAGE_KINDS {
        let info = kind.info();
        let record = records
            .iter()
            .find(|r| r.section() == info.section)
            .expect("all UI sections");
        let group = record.images(kind)?;
        for index in 0..usize::from(info.image_count) {
            let image = group.image(index)?;
            let mut pixels = Vec::new();
            for y in 0..usize::from(image.height()) {
                for x in 0..usize::from(image.width()) {
                    let pixel = image.pixel(x, y).expect("bounded coordinates");
                    pixels.push(if pixel.opaque { pixel.color } else { 128 });
                }
            }
            let character = match kind {
                MainUiImageKind::MenuGlyphs => Some(33 + index),
                MainUiImageKind::PanelGlyphs | MainUiImageKind::VgaPanelGlyphs => Some(
                    usize::from(b"%0123456789-ABCDEFGHIJKLMNOPQRSTUVWXYZ"[index]),
                ),
                MainUiImageKind::DigitRight | MainUiImageKind::DigitLeft => Some(48 + index),
                _ => None,
            }
            .map_or_else(|| "null".to_owned(), |code| code.to_string());
            let name = format!("ui-{kind:?}-{index:03}.pixels.bin");
            write_new(&destination.join(&name), &pixels)?;
            writeln!(
                stdout,
                "{{\"kind\":\"{kind:?}\",\"section\":{},\"index\":{index},\"offset\":{},\"width\":{},\"height\":{},\"planes\":{},\"transparency\":\"{:?}\",\"palette\":\"{:?}\",\"glyph_codepoint\":{character},\"pixel_file\":\"{name}\",\"opaque_pixels\":{}}}",
                info.section.part_index(),
                info.offset + index * info.stride,
                info.width,
                info.height,
                info.planes,
                info.transparency,
                info.palette,
                pixels.iter().filter(|&&p| p != 128).count(),
            )?;
        }
    }
    stdout.flush()?;
    Ok(())
}
