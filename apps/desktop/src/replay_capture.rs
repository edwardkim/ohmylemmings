//! Opt-in bounded replay evidence. Normal interactive timing/captures are unchanged.
use std::{error::Error, io::Write};

pub fn elapsed(text: &str) -> Result<Vec<u64>, Box<dyn Error>> {
    let values: Vec<u64> = text
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::parse)
        .collect::<Result<_, _>>()?;
    if values.is_empty() || values.iter().any(|&v| v == 0 || v > 3_600_000_000) {
        return Err("elapsed replay requires positive microseconds <= one hour".into());
    }
    Ok(values)
}

pub fn write_frame(out: &mut impl Write, probe: u64, rgb: &[u8]) -> Result<(), Box<dyn Error>> {
    if (probe == u64::MAX) != rgb.is_empty() {
        return Err("invalid frame terminator".into());
    }
    out.write_all(&probe.to_le_bytes())?;
    out.write_all(&u32::try_from(rgb.len())?.to_le_bytes())?;
    out.write_all(rgb)?;
    Ok(())
}

pub fn verify_readback(
    size: (u32, u32),
    pixels: &[u8],
    logical: &[u8],
) -> Result<(), Box<dyn Error>> {
    let (w, h) = (size.0 as usize, size.1 as usize);
    let (lw, lh) = (super::WIDTH, super::HEIGHT);
    let scale = (w / lw).min(h / lh);
    if scale == 0
        || w.checked_mul(h).and_then(|n| n.checked_mul(3)) != Some(pixels.len())
        || logical.len() != lw * lh * 3
    {
        return Err("invalid SDL readback dimensions".into());
    }
    let (left, top) = ((w - lw * scale) / 2, (h - lh * scale) / 2);
    for y in 0..h {
        for x in 0..w {
            let expected =
                if (left..left + lw * scale).contains(&x) && (top..top + lh * scale).contains(&y) {
                    let at = ((y - top) / scale * lw + (x - left) / scale) * 3;
                    &logical[at..at + 3]
                } else {
                    &[0, 0, 0]
                };
            let at = (y * w + x) * 3;
            if &pixels[at..at + 3] != expected {
                return Err(format!("SDL readback differs at ({x},{y})").into());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn readback_checks_scaled_pixels_letterbox_and_lengths() {
        let rgb = vec![17; 320 * 200 * 3];
        let (w, h) = (642, 402);
        let mut pixels = vec![0; w * h * 3];
        for y in 1..401 {
            for x in 1..641 {
                pixels[(y * w + x) * 3..(y * w + x) * 3 + 3].fill(17);
            }
        }
        assert!(verify_readback((w as u32, h as u32), &pixels, &rgb).is_ok());
        for at in [0, (w + 1) * 3, (400 * w + 640) * 3] {
            pixels[at] ^= 1;
            assert!(verify_readback((w as u32, h as u32), &pixels, &rgb).is_err());
            pixels[at] ^= 1;
        }
        assert!(verify_readback((1, 1), &[0; 3], &rgb).is_err());
        assert!(verify_readback((640, 400), &pixels, &rgb).is_err());
    }
    #[test]
    fn elapsed_replay_rejects_empty_zero_negative_and_excessive_chunks() {
        assert_eq!(elapsed("1\n33333\n\n2").unwrap(), [1, 33333, 2]);
        for text in ["", "0", "-1", "1.5", "3600000001"] {
            assert!(elapsed(text).is_err(), "{text}");
        }
    }
    #[test]
    fn frame_stream_has_little_endian_header_and_explicit_end() {
        let mut bytes = Vec::new();
        write_frame(&mut bytes, 7, b"abc").unwrap();
        write_frame(&mut bytes, u64::MAX, &[]).unwrap();
        assert_eq!(&bytes[..12], &[7, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0]);
        assert_eq!(&bytes[12..15], b"abc");
        assert!(write_frame(&mut bytes, 0, &[]).is_err());
        assert!(write_frame(&mut bytes, u64::MAX, b"x").is_err());
    }
}
