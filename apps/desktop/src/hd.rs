//! SDL adapter for shared HD textures and commands. No game rules live here.
use lemmings_game::{
    LevelRun,
    hd::scene::{Draw, Renderer},
};
use sdl2::{
    event::Event,
    mouse::MouseButton,
    pixels::{Color, PixelFormatEnum},
    rect::Rect,
    render::{BlendMode, Canvas, ScaleMode, Texture, TextureCreator},
    video::{Window, WindowContext},
};
use std::error::Error;
pub struct Host<'a> {
    pub renderer: Renderer,
    creator: &'a TextureCreator<WindowContext>,
    textures: Vec<Option<(u32, Texture<'a>)>>,
    dirty: bool,
}
impl<'a> Host<'a> {
    pub fn new(
        creator: &'a TextureCreator<WindowContext>,
        bytes: &[u8],
    ) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            renderer: Renderer::new(bytes)?,
            creator,
            textures: Vec::new(),
            dirty: true,
        })
    }
    pub fn invalidate(&mut self) {
        self.dirty = true;
        self.textures.clear();
    }
    pub fn draw(
        &mut self,
        canvas: &mut Canvas<Window>,
        run: &LevelRun,
    ) -> Result<(), Box<dyn Error>> {
        if self.dirty {
            self.renderer.reset(&run.assets)?;
            self.dirty = false;
        }
        let commands = self.renderer.frame(run)?;
        canvas.set_blend_mode(BlendMode::Blend);
        for command in commands {
            match command {
                Draw::Fill {
                    rect: [x, y, w, h],
                    color: [r, g, b, a],
                } => {
                    canvas.set_draw_color(Color::RGBA(r, g, b, a));
                    canvas.fill_rect(Rect::new(x, y, w as u32, h as u32))?;
                }
                Draw::GameClip(enabled) => canvas.set_clip_rect(if enabled {
                    Some(Rect::new(0, 0, 1920, 960))
                } else {
                    None
                }),
                Draw::Blit {
                    texture,
                    source: [sx, sy, sw, sh],
                    dest: [x, y, w, h],
                } => {
                    while self.textures.len() <= texture {
                        self.textures.push(None);
                    }
                    let data = &self.renderer.textures()[texture];
                    if self.textures[texture]
                        .as_ref()
                        .is_none_or(|(r, _)| *r != data.revision)
                    {
                        let mut tex = self.creator.create_texture_static(
                            PixelFormatEnum::RGBA32,
                            data.width,
                            data.height,
                        )?;
                        tex.set_blend_mode(BlendMode::Blend);
                        tex.set_scale_mode(ScaleMode::Linear);
                        tex.update(None, &data.rgba, data.width as usize * 4)?;
                        self.textures[texture] = Some((data.revision, tex));
                    }
                    canvas.copy(
                        &self.textures[texture]
                            .as_ref()
                            .ok_or("HD texture upload")?
                            .1,
                        Rect::new(sx as i32, sy as i32, sw, sh),
                        Rect::new(x, y, w as u32, h as u32),
                    )?;
                }
            }
        }
        canvas.set_clip_rect(None);
        Ok(())
    }
}
/// Read bounded catalog members from a directory, or a legacy single v1 pack.
pub fn read_pack(
    path: &std::path::Path,
    assets: &lemmings_game::Assets,
) -> Result<Vec<u8>, Box<dyn Error>> {
    use std::io::Read;
    fn bounded(path: &std::path::Path, limit: usize) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut data = Vec::new();
        std::fs::File::open(path)
            .map_err(|error| {
                std::io::Error::new(error.kind(), format!("{}: {error}", path.display()))
            })?
            .take((limit + 1) as u64)
            .read_to_end(&mut data)?;
        if data.len() > limit {
            return Err("HD file size limit".into());
        }
        Ok(data)
    }
    if !path.is_dir() {
        return bounded(path, 128 * 1024 * 1024);
    }
    let catalog = lemmings_game::hd::catalog::Catalog::parse(&bounded(
        &path.join("catalog.hdi"),
        12 + 128 * 12,
    )?)?;
    let scope = lemmings_game::hd::catalog::level_scope(assets)?;
    let entries = catalog.select(scope)?;
    let mut result = b"LHDSET01".to_vec();
    result.extend((entries.len() as u16).to_le_bytes());
    result.extend(0u16.to_le_bytes());
    for entry in entries {
        let data = bounded(
            &path.join(format!("atlas-{:03}.hdp", entry.index)),
            entry.bytes,
        )?;
        entry.verify(&data)?;
        result.extend((data.len() as u32).to_le_bytes());
        result.extend(data);
    }
    Ok(result)
}

pub enum Routed {
    Event(Event),
    Restart,
}
pub fn event(mut event: Event, run: &mut LevelRun) -> Routed {
    let point = match &mut event {
        Event::MouseMotion {
            x, y, mousestate, ..
        } => Some((
            x,
            y,
            "move",
            0,
            u8::from(mousestate.left()) | (u8::from(mousestate.right()) * 2),
        )),
        Event::MouseButtonDown {
            x, y, mouse_btn, ..
        } => Some((x, y, "down", button(*mouse_btn), 0)),
        Event::MouseButtonUp {
            x, y, mouse_btn, ..
        } => Some((x, y, "up", button(*mouse_btn), 0)),
        _ => None,
    };
    if let Some((x, y, kind, button, buttons)) = point {
        let Some((px, py)) =
            lemmings_game::hd::input::route(run, kind, (*x).into(), (*y).into(), button, buttons)
        else {
            return Routed::Restart;
        };
        *x = px;
        *y = py;
    }
    Routed::Event(event)
}
fn button(button: MouseButton) -> u8 {
    match button {
        MouseButton::Left => 0,
        MouseButton::Right => 2,
        _ => 1,
    }
}
