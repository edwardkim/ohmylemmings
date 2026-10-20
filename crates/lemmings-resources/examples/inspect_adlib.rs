//! Host-only metadata export; original bytes remain in caller-controlled local files.
use lemmings_core::{Difficulty, LevelId};
use lemmings_resources::{
    ADLIB_EFFECT_COUNT, ADLIB_IMAGE_SIZE, ADLIB_MUSIC_COUNT, AdlibFile, AdlibImage,
};
use std::{env, error::Error, fs};
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    let (image_mode, path) = match args.as_slice() {
        [path] => (false, path),
        [mode, path] if mode == "--image" => (true, path),
        _ => return Err("usage: inspect_adlib [--image] INPUT_FILE".into()),
    };
    let bytes = fs::read(path)?;
    let mut decoded = [0; ADLIB_IMAGE_SIZE];
    let image = if image_mode {
        AdlibImage::parse(&bytes)?
    } else {
        AdlibFile::parse(&bytes)?.decode_into(&mut decoded)?
    };
    print!("{{\"music\":[");
    for i in 0..ADLIB_MUSIC_COUNT {
        if i != 0 {
            print!(",");
        }
        let m = image.music(i)?;
        print!(
            "{{\"index\":{i},\"header_offset\":{},\"sample_rate_factor\":{},\"instrument_offset\":{},\"wait_cycles\":{},\"channel_offsets\":[",
            m.header_offset, m.sample_rate_factor, m.instrument_offset, m.wait_cycles
        );
        for (n, ch) in m.channels().iter().enumerate() {
            if n != 0 {
                print!(",");
            }
            print!("{}", ch.program_offset);
        }
        print!("],\"first_bar_offsets\":[");
        for (n, ch) in m.channels().iter().enumerate() {
            if n != 0 {
                print!(",");
            }
            print!("{}", ch.first_bar_offset);
        }
        print!("]}}");
    }
    print!("],\"effects\":[");
    for i in 0..ADLIB_EFFECT_COUNT {
        if i != 0 {
            print!(",");
        }
        print!(
            "{{\"index\":{i},\"data_offset\":{}}}",
            image.effect_offset(i)?
        );
    }
    print!("],\"level_tracks\":[");
    for (g, group) in [
        Difficulty::Fun,
        Difficulty::Tricky,
        Difficulty::Taxing,
        Difficulty::Mayhem,
    ]
    .into_iter()
    .enumerate()
    {
        for n in 1..=30 {
            if g != 0 || n != 1 {
                print!(",");
            }
            print!(
                "{}",
                image
                    .music_for_level(LevelId::new(group, n)?)?
                    .header_offset
            );
        }
    }
    println!("]}}");
    Ok(())
}
