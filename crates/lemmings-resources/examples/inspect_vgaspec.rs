//! Inspect all four original VGASPEC backgrounds as JSONL, without level placement.
use lemmings_resources::{
    IndexedPixel, VGASPEC_HEIGHT, VGASPEC_PLANAR_SIZE, VGASPEC_WIDTH, VgaSpecFile,
};
use std::{
    env,
    error::Error,
    fs,
    io::{self, BufWriter, Write},
    path::Path,
};

fn hex(output: &mut impl Write, bytes: &[u8]) -> io::Result<()> {
    for byte in bytes {
        write!(output, "{byte:02x}")?;
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args_os().skip(1);
    let (Some(root), None) = (args.next(), args.next()) else {
        return Err("usage: inspect_vgaspec ORIGINAL_ASSET_DIRECTORY".into());
    };
    let mut output = BufWriter::new(io::stdout().lock());
    for set in 0..4 {
        let path = Path::new(&root).join(format!("VGASPEC{set}.DAT"));
        let contextualize = |error: &dyn Error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{}: {error}", path.display()),
            )
        };
        let bytes = fs::read(&path)
            .map_err(|e| io::Error::new(e.kind(), format!("{}: {e}", path.display())))?;
        let file = VgaSpecFile::parse(&bytes).map_err(|e| contextualize(&e))?;
        let mut inner = vec![0; file.decoded_size()];
        let record = file
            .decode_into(&mut inner)
            .map_err(|e| contextualize(&e))?;
        let mut planes = vec![0; VGASPEC_PLANAR_SIZE];
        let image = record
            .decode_into(&mut planes)
            .map_err(|e| contextualize(&e))?;
        let mut pixels = vec![IndexedPixel::default(); VGASPEC_WIDTH * VGASPEC_HEIGHT];
        image
            .decode_into(&mut pixels)
            .map_err(|e| contextualize(&e))?;
        write!(
            output,
            "{{\"set\":{set},\"width\":{VGASPEC_WIDTH},\"height\":{VGASPEC_HEIGHT},\"vga\":{:?},\"ega\":{:?},\"unknown\":{:?},\"transparent_pixels\":{},\"inner_hex\":\"",
            record.vga_palette().map(|c| c.components()),
            record.ega_palette(),
            record.unknown_palette_bytes(),
            pixels.iter().filter(|p| !p.opaque).count(),
        )?;
        hex(&mut output, record.as_bytes())?;
        write!(output, "\",\"planar_hex\":\"")?;
        hex(&mut output, image.planar_bytes())?;
        write!(output, "\",\"pixel_hex\":\"")?;
        for pixel in pixels {
            let byte = pixel.color | if pixel.opaque { 0 } else { 0x80 };
            write!(output, "{byte:02x}")?;
        }
        writeln!(output, "\"}}")?;
    }
    output.flush()?;
    Ok(())
}
