//! Export the 28 original MAIN.DAT sprite animations, refusing overwrites.
use lemmings_resources::{
    LEMMING_ANIMATION_COUNT, LEMMING_SPRITE_DATA_SIZE, MainDatFile, SpriteDirection,
};
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
fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args_os().skip(1);
    let (Some(source), Some(destination), None) = (args.next(), args.next(), args.next()) else {
        return Err("usage: inspect_sprites MAIN_DAT_FILE OUTPUT_DIRECTORY".into());
    };
    let source = Path::new(&source);
    let destination = Path::new(&destination);
    let bytes = fs::read(source)
        .map_err(|e| io::Error::new(e.kind(), format!("{}: {e}", source.display())))?;
    let parse_error = |error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{}: {error}", source.display()),
        )
    };
    let file = MainDatFile::parse(&bytes).map_err(parse_error)?;
    let mut inner = vec![0; LEMMING_SPRITE_DATA_SIZE];
    let sprites = file.decode_sprites_into(&mut inner).map_err(parse_error)?;
    fs::create_dir_all(destination)?;
    write_new(&destination.join("section-000.bin"), sprites.as_bytes())?;
    let mut stdout = BufWriter::new(io::stdout().lock());
    for slot in 0..LEMMING_ANIMATION_COUNT {
        let animation = sprites.animation(slot)?;
        let info = animation.info();
        let mut pixels = Vec::new();
        for frame in 0..usize::from(info.frame_count) {
            let image = animation.frame(frame)?;
            for y in 0..usize::from(info.height) {
                for x in 0..usize::from(info.width) {
                    let pixel = image.pixel(x, y).expect("bounded coordinates");
                    pixels.push(if pixel.opaque { pixel.color } else { 128 });
                }
            }
        }
        let name = format!("sprite-{slot:02}.pixels.bin");
        write_new(&destination.join(&name), &pixels)?;
        let direction = match info.direction {
            Some(SpriteDirection::Right) => "\"Right\"",
            Some(SpriteDirection::Left) => "\"Left\"",
            None => "null",
        };
        let (origin_x, origin_y) = info.origin_offset();
        writeln!(
            stdout,
            "{{\"slot\":{slot},\"kind\":\"{:?}\",\"direction\":{direction},\"offset\":{},\"frame_count\":{},\"width\":{},\"height\":{},\"planes\":{},\"foot_x\":{},\"foot_y\":{},\"origin_x\":{origin_x},\"origin_y\":{origin_y},\"pixel_file\":\"{name}\",\"opaque_pixels\":{}}}",
            info.kind,
            info.offset,
            info.frame_count,
            info.width,
            info.height,
            info.planes,
            info.foot_x,
            info.foot_y,
            pixels.iter().filter(|&&p| p != 128).count(),
        )?;
    }
    stdout.flush()?;
    Ok(())
}
