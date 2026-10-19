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
    Blocker,
    Bomber,
    Climber,
    Floater,
}
pub struct View {
    pub camera: i32,
    pub replaying: bool,
    pub selected_skill: SelectedSkill,
    pub next_level: Option<u8>,
    pub result_error: bool,
    world: Vec<[u8; 4]>,
    iteration: Option<u64>,
}
impl View {
    pub fn new(camera: i32) -> Self {
        Self {
            camera,
            replaying: false,
            selected_skill: SelectedSkill::Digger,
            next_level: None,
            result_error: false,
            world: Vec::new(),
            iteration: None,
        }
    }
    fn result_frame(
        &self,
        assets: &Assets,
        session: &Session,
        result: lemmings_core::LevelResult,
    ) -> Vec<u8> {
        let mut frame = [12, 18, 32].repeat(WIDTH * HEIGHT);
        let white = [225, 235, 240];
        let purple = [190, 165, 255];
        let heading = if result.success {
            "LEVEL COMPLETE"
        } else {
            "TRY AGAIN"
        };
        centered(
            &mut frame,
            18,
            heading,
            if result.success {
                [100, 235, 155]
            } else {
                [255, 160, 120]
            },
        );
        centered(&mut frame, 36, &assets.title.to_ascii_uppercase(), white);
        centered(
            &mut frame,
            57,
            match result.reason {
                lemmings_core::EndReason::TimeExpired => "TIME IS UP",
                lemmings_core::EndReason::PopulationExhausted => "ALL LEMMINGS ACCOUNTED FOR",
            },
            purple,
        );
        centered(
            &mut frame,
            76,
            &format!(
                "YOU RESCUED {}%   NEEDED {}%",
                result.rescued_percent, result.target_percent
            ),
            white,
        );
        centered(
            &mut frame,
            91,
            &format!("SAVED {} / {}", result.rescued, result.lemming_count),
            white,
        );
        let seconds = session.scheduler.state().seconds_remaining;
        centered(
            &mut frame,
            106,
            &format!("TIME LEFT {}:{:02}", seconds / 60, seconds % 60),
            white,
        );
        let prompt = if !result.success {
            "ENTER / LEFT CLICK: RETRY".to_owned()
        } else if let Some(next) = self.next_level {
            format!("ENTER / LEFT CLICK: LEVEL {next}")
        } else {
            "END OF SUPPORTED LEVELS".to_owned()
        };
        centered(&mut frame, 132, &prompt, purple);
        centered(&mut frame, 149, "R: RESTART THIS LEVEL", white);
        centered(&mut frame, 164, "ESC / RIGHT CLICK: QUIT", white);
        if self.result_error {
            centered(
                &mut frame,
                184,
                "LEVEL LOAD FAILED - TRY R OR ENTER",
                [255, 160, 120],
            );
        }
        frame
    }
    pub fn scroll(&mut self, delta: i32, world_width: usize) {
        let max = world_width.saturating_sub(WIDTH).min(i32::MAX as usize) as i32;
        self.camera = self.camera.saturating_add(delta).clamp(0, max);
    }
    pub fn frame(&mut self, assets: &Assets, session: &Session) -> Result<Vec<u8>, Box<dyn Error>> {
        if let Some(result) = session.result() {
            return Ok(self.result_frame(assets, session, result));
        }
        self.scroll(0, assets.width);
        let iteration = session.scheduler.step_state().iteration;
        if self.iteration != Some(iteration) {
            // Erase deleted terrain before object composition so objects behind
            // a freshly dug hole become visible with their original draw flags.
            self.world = assets.scene_with_traps(
                iteration,
                session.render_terrain(),
                session.render_bricks(),
                session.traps(),
            )?;
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
                if a.particle_timer > 0
                    && let Some(bytes) = &assets.particles
                {
                    let particles = lemmings_resources::ExplosionParticles::parse(bytes)?;
                    let colors = [4, 15, 14, 13, 12, 11, 10, 9, 8, 11, 10, 9, 8, 7, 6, 2];
                    for n in 0..80 {
                        if let Some((dx, dy)) = particles.offset(usize::from(a.particle_frame), n) {
                            let x = i64::from(a.x) + i64::from(dx) - i64::from(self.camera);
                            let y = i64::from(a.y) + i64::from(dy);
                            if (0..WIDTH as i64).contains(&x) && (0..160).contains(&y) {
                                let at = (y as usize * WIDTH + x as usize) * 3;
                                output[at..at + 3].copy_from_slice(&assets.palette[colors[n % 16]]);
                            }
                        }
                    }
                }
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
                MotionAction::Blocking => LemmingSpriteKind::Blocking,
                MotionAction::Climbing => LemmingSpriteKind::Climbing,
                MotionAction::Floating => LemmingSpriteKind::Umbrella,
                MotionAction::Drowning => LemmingSpriteKind::Drowning,
                MotionAction::Vaporizing => LemmingSpriteKind::Vaporizing,
                MotionAction::Exiting => LemmingSpriteKind::Exiting,
                MotionAction::Hoisting => LemmingSpriteKind::Hoisting,
                MotionAction::OhNo => LemmingSpriteKind::OhNo,
                MotionAction::Exploding => LemmingSpriteKind::Explosion,
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
            if let Some(digit) = a.countdown_digit() {
                let mut glyph = [128; 64];
                for gy in 0..8 {
                    for gx in 0..8 {
                        if assets.countdown_masks[usize::from(9 - digit) * 8 + gy] & (128 >> gx)
                            != 0
                        {
                            glyph[gy * 8 + gx] = 3;
                        }
                    }
                }
                let x = i64::from(a.x) - i64::from(self.camera);
                let y = (i64::from(a.y) - i64::from(info.foot_y)).max(0) - 8;
                if let (Ok(x), Ok(y)) = (i32::try_from(x), i32::try_from(y)) {
                    blit(&mut output, &glyph, (8, 8), (x, y), &assets.palette);
                }
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
            session.scheduler.nuke_state().max_lemmings,
            state.release_rate,
            state.seconds_remaining / 60,
            state.seconds_remaining % 60
        );
        for pixel in output[160 * WIDTH * 3..].chunks_exact_mut(3) {
            pixel.copy_from_slice(&[20, 24, 32]);
        }
        text(&mut output, 4, 164, &title, [220, 228, 240]);
        let controls = if let Some(result) = session.result() {
            if result.success {
                "WON | ESC QUIT"
            } else {
                "LOST | ESC QUIT"
            }
        } else if session.scheduler.nuke_state().is_nuked {
            if step.paused {
                "NUKE ACTIVE | SPACE RESUME"
            } else {
                "NUKE ACTIVE | SPACE PAUSE"
            }
        } else if self.replaying {
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
        if session.has_exit() || session.completion_rules().is_some() {
            let caption = if let Some(rules) = session.completion_rules() {
                format!(
                    "{}% / {}%",
                    lemmings_core::percentage(
                        session.saved() as u32,
                        session.scheduler.nuke_state().max_lemmings
                    ),
                    rules.target_percent()
                )
            } else {
                format!("SAVED {}", session.saved())
            };
            text(
                &mut output,
                WIDTH - caption.len() * 6 - 4,
                176,
                &caption,
                [220, 228, 240],
            );
        }
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
                SelectedSkill::Floater => {
                    format!(
                        "FLOAT {} | F/C/E/K/U/M/B/D | CLICK",
                        session.remaining_floaters
                    )
                }
                SelectedSkill::Climber => {
                    format!(
                        "CLIMB {} | C/E/K/U/M/B/D | CLICK",
                        session.remaining_climbers
                    )
                }
                SelectedSkill::Bomber => {
                    format!("BOMB {} | E/K/U/M/B/D | CLICK", session.remaining_bombers)
                }
                SelectedSkill::Blocker => {
                    format!("BLOCK {} | K/U/M/B/D | CLICK", session.remaining_blockers)
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
fn centered(output: &mut [u8], y: usize, value: &str, color: [u8; 3]) {
    // Long level names stay centered without overflowing or silently blank glyphs.
    let value: String = value.chars().take(WIDTH / 6).collect();
    text(
        output,
        (WIDTH - value.chars().count() * 6) / 2,
        y,
        &value,
        color,
    );
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
        '-' => [0, 0, 0, 31, 0, 0, 0],
        ':' => [0, 4, 4, 0, 4, 4, 0],
        '!' => [4, 4, 4, 4, 4, 0, 4],
        '/' => [1, 1, 2, 4, 8, 16, 16],
        '%' => [25, 25, 2, 4, 8, 19, 19],
        '|' => [4; 7],
        _ => [0; 7],
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn result_scene_is_frozen_distinct_and_contains_outcome_color() {
        for (kind, color) in [
            ("win", [100, 235, 155]),
            ("loss", [255, 160, 120]),
            ("zero", [255, 160, 120]),
        ] {
            let mut run = crate::lifecycle::LevelRun::new(
                Assets::synthetic_completion(kind),
                1,
                Some(2),
                crate::pacing::Pacing::Reference,
            )
            .unwrap();
            let playing = run.view.frame(&run.assets, &run.session).unwrap();
            for _ in 0..1200 {
                run.session.tick().unwrap();
                if run.session.result().is_some() {
                    break;
                }
            }
            assert!(run.session.result().is_some());
            let before = run.session.snapshot();
            let result = run.view.frame(&run.assets, &run.session).unwrap();
            assert_ne!(result, playing);
            assert!(
                result[18 * WIDTH * 3..25 * WIDTH * 3]
                    .chunks_exact(3)
                    .any(|p| p == color)
            );
            assert_eq!(run.view.frame(&run.assets, &run.session).unwrap(), result);
            assert_eq!(run.session.snapshot(), before);
        }
    }
    #[test]
    fn removed_explosion_particles_use_signed_offsets_palette_frame_and_playfield_clip() {
        let mut assets = Assets::synthetic();
        assets.sprites.fill(0);
        assets.palette[4] = [11, 22, 33];
        assets.palette[15] = [44, 55, 66];
        let mut table = vec![128; lemmings_resources::EXPLOSION_PARTICLE_DATA_SIZE];
        let at = 3 * 160;
        table[at..at + 12].copy_from_slice(&[255, 254, 0, 0, 128, 1, 129, 0, 0, 127, 0, 128]);
        assets.particles = Some(table);
        let mut session = Session::new(
            assets.config,
            assets.entrances.clone(),
            assets.width,
            assets.height,
            assets.terrain.clone(),
        )
        .unwrap();
        let mut a = Actor::from_spawn(
            SpawnRequest {
                actor_id: 0,
                entrance_index: 0,
                iteration: 54,
            },
            &[EntrancePoint { x: 0, y: 0 }],
            MotionRules::default(),
        )
        .unwrap()
        .state();
        a.x = 32;
        a.y = 100;
        a.action = MotionAction::Exploding;
        a.frame = 0;
        a.end_of_animation = true;
        a.fallen = 0;
        a.removed = Some(RemovalReason::Exploded);
        a.particle_timer = 49;
        a.particle_frame = 3;
        session
            .actors
            .push(Actor::from_state(a, MotionRules::default()).unwrap());
        let frame = View::new(0).frame(&assets, &session).unwrap();
        let p = (98 * WIDTH + 31) * 3;
        assert_eq!(&frame[p..p + 3], &[11, 22, 33]);
        let p = (100 * WIDTH + 32) * 3;
        assert_eq!(&frame[p..p + 3], &[44, 55, 66]);
        a.particle_timer = 0;
        session.actors[0] = Actor::from_state(a, MotionRules::default()).unwrap();
        let frame = View::new(0).frame(&assets, &session).unwrap();
        let p = (98 * WIDTH + 31) * 3;
        assert_eq!(&frame[p..p + 3], &[0, 0, 0]);
    }
    #[test]
    fn countdown_glyphs_follow_sprite_foot_anchor_without_mirroring() {
        for (action, foot) in [
            (MotionAction::Walking, 10),
            (MotionAction::Digging, 12),
            (MotionAction::Building, 13),
            (MotionAction::Mining, 13),
            (MotionAction::Climbing, 12),
            (MotionAction::Hoisting, 12),
            (MotionAction::Floating, 16),
            (MotionAction::Exiting, 13),
            (MotionAction::Drowning, 10),
            (MotionAction::Vaporizing, 14),
        ] {
            for (timer, digit) in [(79, 5u8), (64, 4), (48, 3), (32, 2), (16, 1)] {
                let mut assets = Assets::synthetic();
                assets.sprites.fill(0);
                assets.countdown_masks.fill(0);
                assets.countdown_masks[usize::from(9 - digit) * 8] = 128 >> digit;
                let mut session = Session::new(
                    assets.config,
                    assets.entrances.clone(),
                    assets.width,
                    assets.height,
                    assets.terrain.clone(),
                )
                .unwrap();
                let mut a = Actor::from_spawn(
                    SpawnRequest {
                        actor_id: 0,
                        entrance_index: 0,
                        iteration: 54,
                    },
                    &[EntrancePoint { x: 0, y: 0 }],
                    MotionRules::default(),
                )
                .unwrap()
                .state();
                a.action = action;
                a.is_floater = action == MotionAction::Floating;
                a.is_climber = matches!(action, MotionAction::Climbing | MotionAction::Hoisting);
                a.x = 32;
                a.y = 100;
                a.fallen = 0;
                a.explosion_timer = timer;
                a.direction = MotionDirection::Left;
                a.bricks_left = if action == MotionAction::Building {
                    12
                } else {
                    0
                };
                session
                    .actors
                    .push(Actor::from_state(a, MotionRules::default()).unwrap());
                let pixels = View::new(0).frame(&assets, &session).unwrap();
                let at = ((100 - foot - 8) * WIDTH + 32 + usize::from(digit)) * 3;
                assert_eq!(&pixels[at..at + 3], &assets.palette[3]);
                let empty = ((100 - foot - 8) * WIDTH + 32) * 3;
                assert_eq!(&pixels[empty..empty + 3], &[0, 0, 0]);
            }
        }
    }
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
                blocker_saved: None,
                is_climber: false,
                is_floater: false,
                float_index: 0,
                walking_once: false,
                explosion_timer: 0,
                particle_timer: 0,
                particle_frame: 0,
                object_below: 0,
                object_in_front: 0,
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
                blocker_saved: None,
                is_climber: false,
                is_floater: false,
                float_index: 0,
                walking_once: false,
                explosion_timer: 0,
                particle_timer: 0,
                particle_frame: 0,
                object_below: 0,
                object_in_front: 0,
                removed: Some(RemovalReason::BottomBoundary),
            },
            MotionRules::default(),
        )
        .unwrap();
        s.actors.push(removed);
        assert_eq!(v.frame(&a, &s).unwrap(), frame);
    }
    #[test]
    fn rescue_count_remains_visible_with_a_long_title_and_updates_after_exit() {
        let mut assets = Assets::synthetic_exit(false);
        assets.title = "A".repeat(120);
        assets.config.total_lemmings = 1;
        let mut s = Session::new(
            assets.config,
            assets.entrances.clone(),
            assets.width,
            assets.height,
            assets.terrain.clone(),
        )
        .unwrap();
        s.configure_blocker(0, assets.object_map.clone()).unwrap();
        let mut view = View::new(0);
        let before = view.frame(&assets, &s).unwrap();
        let x = WIDTH - "SAVED 0".len() * 6 - 4;
        let at = (176 * WIDTH + x + 1) * 3;
        assert_eq!(&before[at..at + 3], &[220, 228, 240]);
        for _ in 0..150 {
            s.tick().unwrap();
        }
        assert_eq!(s.saved(), 1);
        let after = view.frame(&assets, &s).unwrap();
        let digit = (176 * WIDTH + x + 6 * 6 + 1) * 3;
        assert_ne!(&before[digit..digit + 3], &after[digit..digit + 3]);
    }
}
