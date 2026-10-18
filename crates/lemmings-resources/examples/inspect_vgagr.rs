//! Inspect all five original VGAGR sets, including animation frames and previews.
use lemmings_resources::{GroundRecord, IndexedPixel, PlanarImage, VgagrFile};
use std::{
    env,
    error::Error,
    fs,
    io::{self, BufWriter, Write},
    path::Path,
};

fn emit(
    output: &mut impl Write,
    set: usize,
    kind: &str,
    slot: usize,
    frame: Option<usize>,
    image: PlanarImage<'_>,
) -> io::Result<()> {
    let mut pixels = vec![IndexedPixel::default(); image.pixel_count()];
    image.decode_into(&mut pixels).map_err(io::Error::other)?;
    write!(
        output,
        "{{\"set\":{set},\"kind\":\"{kind}\",\"slot\":{slot},\"frame\":"
    )?;
    match frame {
        Some(index) => write!(output, "{index}")?,
        None => write!(output, "null")?,
    }
    write!(
        output,
        ",\"width\":{},\"height\":{},\"pixel_hex\":\"",
        image.width(),
        image.height(),
    )?;
    for pixel in pixels {
        // Reference comparison encoding only: bit 7 marks a transparent pixel.
        let byte = pixel.color | if pixel.opaque { 0 } else { 0x80 };
        write!(output, "{byte:02x}")?;
    }
    writeln!(output, "\"}}")
}

fn read(path: &Path) -> io::Result<Vec<u8>> {
    fs::read(path)
        .map_err(|error| io::Error::new(error.kind(), format!("{}: {error}", path.display())))
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args_os().skip(1);
    let (Some(root), None) = (args.next(), args.next()) else {
        return Err("usage: inspect_vgagr ORIGINAL_ASSET_DIRECTORY".into());
    };
    let root = Path::new(&root);
    let mut output = BufWriter::new(io::stdout().lock());
    for set in 0..5 {
        let ground_path = root.join(format!("GROUND{set}O.DAT"));
        let vga_path = root.join(format!("VGAGR{set}.DAT"));
        let contextualize = |error: &dyn Error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "{} + {}: {error}",
                    ground_path.display(),
                    vga_path.display()
                ),
            )
        };
        let ground_bytes = read(&ground_path)?;
        let vga_bytes = read(&vga_path)?;
        let ground = GroundRecord::parse(&ground_bytes).map_err(|e| contextualize(&e))?;
        let file = VgagrFile::parse(&vga_bytes).map_err(|e| contextualize(&e))?;
        let [terrain_size, object_size] = file.decoded_sizes();
        let mut terrain = vec![0; terrain_size];
        let mut objects = vec![0; object_size];
        let images = file
            .decode_into(ground, &mut terrain, &mut objects)
            .map_err(|e| contextualize(&e))?;
        for slot in 0..64 {
            if let Some(image) = images.terrain(slot)? {
                emit(&mut output, set, "terrain", slot, None, image)?;
            }
        }
        for slot in 0..16 {
            if let Some(frames) = images.object(slot)? {
                for frame in 0..usize::from(frames.frame_count()) {
                    emit(
                        &mut output,
                        set,
                        "object",
                        slot,
                        Some(frame),
                        frames.frame(frame)?,
                    )?;
                }
                emit(&mut output, set, "preview", slot, None, frames.preview())?;
            }
        }
    }
    output.flush()?;
    Ok(())
}
