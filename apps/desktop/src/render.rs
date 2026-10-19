use crate::{assets::Assets, session::Session};
use lemmings_core::{MotionAction, MotionDirection};
use lemmings_resources::{LemmingSpriteKind, LemmingSprites, SpriteDirection};
use std::error::Error;
pub const WIDTH: usize = 320;
pub const HEIGHT: usize = 200;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectedSkill {
    Digger,
    Basher,
    Miner,
    Builder,
}
pub struct View {
    pub camera: i32,
    pub replaying: bool,
    pub selected_skill: SelectedSkill,
    world: Vec<[u8; 4]>,
    iteration: Option<u64>,
}
impl View {
    pub fn new(camera: i32) -> Self {
        Self {
            camera,
            replaying: false,
            selected_skill: SelectedSkill::Digger,
            world: Vec::new(),
            iteration: None,
        }
    }
    pub fn scroll(&mut self, delta: i32, world_width: usize) {
        let max = world_width.saturating_sub(WIDTH).min(i32::MAX as usize) as i32;
        self.camera = self.camera.saturating_add(delta).clamp(0, max);
    }
    pub fn frame(&mut self, assets: &Assets, session: &Session) -> Result<Vec<u8>, Box<dyn Error>> {
        self.scroll(0, assets.width);
        let iteration = session.scheduler.step_state().iteration;
        if self.iteration != Some(iteration) {
            // Erase deleted terrain before object composition so objects behind
            // a freshly dug hole become visible with their original draw flags.
            self.world =
                assets.scene_with_terrain(iteration, session.terrain(), session.brick_mask())?;
            self.iteration = Some(iteration);
        }
        let mut output = vec![0; WIDTH * HEIGHT * 3];
        for y in 0..160.min(assets.height) {
            for x in 0..WIDTH {
                let wx = x + self.camera as usize;
                if wx < assets.width {
                    output[(y * WIDTH + x) * 3..(y * WIDTH + x) * 3 + 3]
                        .copy_from_slice(&self.world[y * assets.width + wx][..3]);
                }
            }
        }
        let sprites = LemmingSprites::parse(&assets.sprites)?;
        for actor in &session.actors {
            let a = actor.state();
            if a.removed.is_some() {
                continue;
            }
            let kind = match a.action {
                MotionAction::Walking => LemmingSpriteKind::Walking,
                MotionAction::Jumping => LemmingSpriteKind::Jumping,
                MotionAction::Falling => LemmingSpriteKind::Falling,
                MotionAction::Splatting => LemmingSpriteKind::Splatting,
                MotionAction::Digging => LemmingSpriteKind::Digging,
                MotionAction::Bashing => LemmingSpriteKind::Bashing,
                MotionAction::Mining => LemmingSpriteKind::Mining,
                MotionAction::Building => LemmingSpriteKind::Building,
                MotionAction::Shrugging => LemmingSpriteKind::Shrugging,
            };
            let direction = if a.direction == MotionDirection::Left {
                SpriteDirection::Left
            } else {
                SpriteDirection::Right
            };
            let animation = sprites.for_sprite(kind, direction);
            let info = animation.info();
            let frame = animation.frame(usize::from(a.frame))?;
            let mut pixels = Vec::with_capacity(usize::from(info.width) * usize::from(info.height));
            for y in 0..usize::from(info.height) {
                for x in 0..usize::from(info.width) {
                    let p = frame.pixel(x, y).unwrap();
                    pixels.push(if p.opaque { p.color } else { 128 });
                }
            }
            let x = i64::from(a.x) - i64::from(self.camera) - i64::from(info.foot_x);
            let y = i64::from(a.y) - i64::from(info.foot_y);
            if let (Ok(x), Ok(y)) = (i32::try_from(x), i32::try_from(y)) {
                blit(
                    &mut output,
                    &pixels,
                    (usize::from(info.width), usize::from(info.height)),
                    (x, y),
                    &assets.palette,
                );
            }
        }
        let step = session.scheduler.step_state();
        let state = session.scheduler.state();
        let alive = session
            .actors
            .iter()
            .filter(|a| a.state().removed.is_none())
            .count();
        let title = format!(
            "{}  OUT {alive}/{} RR {} TIME {}:{:02}",
            assets.title,
            session.scheduler.config().total_lemmings,
            state.release_rate,
            state.seconds_remaining / 60,
            state.seconds_remaining % 60
        );
        for pixel in output[160 * WIDTH * 3..].chunks_exact_mut(3) {
            pixel.copy_from_slice(&[20, 24, 32]);
        }
        text(&mut output, 4, 164, &title, [220, 228, 240]);
        let controls = if self.replaying {
            "REPLAYING RECORDED INPUT"
        } else if step.paused {
            "SPACE RESUME | UP/DOWN RATE"
        } else {
            "SPACE PAUSE | UP/DOWN RATE"
        };
        text(
            &mut output,
            4,
            176,
            controls,
            if step.paused {
                [255, 210, 80]
            } else {
                [140, 190, 170]
            },
        );
        text(
            &mut output,
            4,
            188,
            &match self.selected_skill {
                SelectedSkill::Digger => format!(
                    "DIG {} | CLICK LEMMING | ESC QUIT",
                    session.remaining_diggers
                ),
                SelectedSkill::Basher => {
                    format!("BASH {} | B/D SELECT | CLICK", session.remaining_bashers)
                }
                SelectedSkill::Miner => {
                    format!("MINE {} | M/B/D SELECT | CLICK", session.remaining_miners)
                }
                SelectedSkill::Builder => {
                    format!("BUILD {} | U/M/B/D | CLICK", session.remaining_builders)
                }
            },
            [140, 190, 170],
        );
        Ok(output)
    }
}
fn blit(
    output: &mut [u8],
    pixels: &[u8],
    size: (usize, usize),
    origin: (i32, i32),
    palette: &[[u8; 3]; 16],
) {
    for y in 0..size.1 {
        for x in 0..size.0 {
            let dx = i64::from(origin.0) + x as i64;
            let dy = i64::from(origin.1) + y as i64;
            if !(0..WIDTH as i64).contains(&dx) || !(0..160).contains(&dy) {
                continue;
            }
            let color = pixels[y * size.0 + x];
            if color == 128 {
                continue;
            }
            let offset = (dy as usize * WIDTH + dx as usize) * 3;
            output[offset..offset + 3].copy_from_slice(&palette[usize::from(color)]);
        }
    }
}
fn text(output: &mut [u8], x: usize, y: usize, value: &str, color: [u8; 3]) {
    for (index, c) in value.chars().enumerate() {
        let rows = glyph(c);
        for (dy, row) in rows.iter().enumerate() {
            for dx in 0..5 {
                let px = x + index * 6 + dx;
                let py = y + dy;
                if px < WIDTH && py < HEIGHT && (row & (1 << (4 - dx))) != 0 {
                    output[(py * WIDTH + px) * 3..(py * WIDTH + px) * 3 + 3]
                        .copy_from_slice(&color);
                }
            }
        }
    }
}
fn glyph(c: char) -> [u8; 7] {
    match c {
        'A' => [14, 17, 17, 31, 17, 17, 17],
        'B' => [30, 17, 17, 30, 17, 17, 30],
        'C' => [14, 17, 16, 16, 16, 17, 14],
        'D' => [30, 17, 17, 17, 17, 17, 30],
        'E' => [31, 16, 16, 30, 16, 16, 31],
        'F' => [31, 16, 16, 30, 16, 16, 16],
        'G' => [14, 17, 16, 23, 17, 17, 14],
        'H' => [17, 17, 17, 31, 17, 17, 17],
        'I' => [14, 4, 4, 4, 4, 4, 14],
        'J' => [7, 2, 2, 2, 18, 18, 12],
        'K' => [17, 18, 20, 24, 20, 18, 17],
        'L' => [16, 16, 16, 16, 16, 16, 31],
        'M' => [17, 27, 21, 21, 17, 17, 17],
        'N' => [17, 25, 21, 19, 17, 17, 17],
        'O' => [14, 17, 17, 17, 17, 17, 14],
        'P' => [30, 17, 17, 30, 16, 16, 16],
        'Q' => [14, 17, 17, 17, 21, 18, 13],
        'R' => [30, 17, 17, 30, 20, 18, 17],
        'S' => [15, 16, 16, 14, 1, 1, 30],
        'T' => [31, 4, 4, 4, 4, 4, 4],
        'U' => [17, 17, 17, 17, 17, 17, 14],
        'V' => [17, 17, 17, 17, 17, 10, 4],
        'W' => [17, 17, 17, 21, 21, 21, 10],
        'X' => [17, 17, 10, 4, 10, 17, 17],
        'Y' => [17, 17, 10, 4, 4, 4, 4],
        'Z' => [31, 1, 2, 4, 8, 16, 31],
        '0' => [14, 17, 19, 21, 25, 17, 14],
        '1' => [4, 12, 4, 4, 4, 4, 14],
        '2' => [14, 17, 1, 2, 4, 8, 31],
        '3' => [30, 1, 1, 14, 1, 1, 30],
        '4' => [2, 6, 10, 18, 31, 2, 2],
        '5' => [31, 16, 16, 30, 1, 1, 30],
        '6' => [14, 16, 16, 30, 17, 17, 14],
        '7' => [31, 1, 2, 4, 8, 8, 8],
        '8' => [14, 17, 17, 14, 17, 17, 14],
        '9' => [14, 17, 17, 15, 1, 1, 14],
        ':' => [0, 4, 4, 0, 4, 4, 0],
        '!' => [4, 4, 4, 4, 4, 0, 4],
        '/' => [1, 1, 2, 4, 8, 16, 16],
        '|' => [4; 7],
        _ => [0; 7],
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use lemmings_core::*;
    #[test]
    fn camera_clamps_at_both_edges_even_for_small_worlds() {
        let mut v = View::new(0);
        v.scroll(-100, 1584);
        assert_eq!(v.camera, 0);
        v.scroll(5000, 1584);
        assert_eq!(v.camera, 1264);
        v.scroll(i32::MAX, 1584);
        assert_eq!(v.camera, 1264);
        v.scroll(0, 100);
        assert_eq!(v.camera, 0);
    }
    #[test]
    fn transparent_and_offscreen_sprite_pixels_leave_background_intact() {
        let mut out = vec![10; WIDTH * HEIGHT * 3];
        let mut colors = [[0; 3]; 16];
        colors[3] = [1, 2, 3];
        blit(&mut out, &[3, 128, 3, 3], (2, 2), (-1, -1), &colors);
        assert_eq!(&out[..3], &[1, 2, 3]);
        assert_eq!(&out[3..6], &[10; 3]);
        blit(&mut out, &[128], (1, 1), (1, 0), &colors);
        assert_eq!(&out[3..6], &[10; 3]);
        blit(&mut out, &[3], (1, 1), (0, 160), &colors);
        assert_eq!(&out[160 * WIDTH * 3..160 * WIDTH * 3 + 3], &[10; 3]);
    }
    #[test]
    fn rendering_and_camera_changes_do_not_advance_or_modify_session() {
        let a = Assets::synthetic();
        let mut s = Session::new(
            a.config,
            a.entrances.clone(),
            a.width,
            a.height,
            a.terrain.clone(),
        )
        .unwrap();
        for _ in 0..60 {
            s.tick().unwrap();
        }
        let before = s.snapshot();
        let mut v = View::new(0);
        let first = v.frame(&a, &s).unwrap();
        assert_eq!(first.len(), WIDTH * HEIGHT * 3);
        assert_eq!(first, v.frame(&a, &s).unwrap());
        v.scroll(200, a.width);
        v.frame(&a, &s).unwrap();
        assert_eq!(s.snapshot(), before);
    }
    #[test]
    fn sprite_foot_anchor_and_removed_actor_visibility_are_preserved() {
        let a = Assets::synthetic();
        let mut s = Session::new(
            a.config,
            a.entrances.clone(),
            a.width,
            a.height,
            a.terrain.clone(),
        )
        .unwrap();
        let actor = Actor::from_state(
            ActorState {
                id: 0,
                born_iteration: 0,
                x: 20,
                y: 30,
                direction: MotionDirection::Right,
                action: MotionAction::Walking,
                frame: 0,
                end_of_animation: false,
                fallen: 0,
                new_digger: false,
                bricks_left: 0,
                removed: None,
            },
            MotionRules::default(),
        )
        .unwrap();
        s.actors.push(actor);
        let mut v = View::new(0);
        let frame = v.frame(&a, &s).unwrap();
        assert_eq!(
            &frame[((30 - 10) * WIDTH + (20 - 8)) * 3..((30 - 10) * WIDTH + (20 - 8)) * 3 + 3],
            &a.palette[3]
        );
        assert_eq!(
            &frame[(30 * WIDTH + 20) * 3..(30 * WIDTH + 20) * 3 + 3],
            &[0; 3]
        );
        let removed = Actor::from_state(
            ActorState {
                id: 1,
                born_iteration: 0,
                x: 20,
                y: 164,
                direction: MotionDirection::Right,
                action: MotionAction::Falling,
                frame: 0,
                end_of_animation: false,
                fallen: 0,
                new_digger: false,
                bricks_left: 0,
                removed: Some(RemovalReason::BottomBoundary),
            },
            MotionRules::default(),
        )
        .unwrap();
        s.actors.push(removed);
        assert_eq!(v.frame(&a, &s).unwrap(), frame);
    }
}
