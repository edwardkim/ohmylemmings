//! SDL copies the embedded RGBA surface; no runtime icon path is required.
use sdl2::{pixels::PixelFormatEnum, surface::Surface, video::Window};

pub fn apply(window: &mut Window) -> Result<(), String> {
    let mut pixels = include_bytes!("../resources/app-icon.rgba").to_vec();
    let surface = Surface::from_data(&mut pixels, 64, 64, 64 * 4, PixelFormatEnum::RGBA32)?;
    window.set_icon(&surface);
    Ok(())
}
