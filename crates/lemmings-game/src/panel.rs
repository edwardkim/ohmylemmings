//! Shared skill toolbar. Coordinates are logical 320×200 pixels.
use crate::{
    assets::Assets,
    render::{SelectedSkill, View},
    session::Session,
};
use lemmings_resources::{MainDatFile, MainUiImageKind, MainUiSection, UiImage};
use std::error::Error;

pub const SKILLS: [SelectedSkill; 8] = [
    SelectedSkill::Climber,
    SelectedSkill::Floater,
    SelectedSkill::Bomber,
    SelectedSkill::Blocker,
    SelectedSkill::Builder,
    SelectedSkill::Basher,
    SelectedSkill::Miner,
    SelectedSkill::Digger,
];
const GLYPHS: &str = "%0123456789-ABCDEFGHIJKLMNOPQRSTUVWXYZ";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Button {
    Slower,
    Faster,
    Skill(SelectedSkill),
    Pause,
    Nuke,
}

pub fn hit(x: i32, y: i32) -> Option<Button> {
    if !(176..198).contains(&y) || !(0..192).contains(&x) || !(1..15).contains(&(x % 16)) {
        return None;
    }
    Some(match x / 16 {
        0 => Button::Slower,
        1 => Button::Faster,
        2..=9 => Button::Skill(SKILLS[(x / 16 - 2) as usize]),
        10 => Button::Pause,
        _ => Button::Nuke,
    })
}

pub fn minimap_camera(x: i32, y: i32, width: usize) -> Option<i32> {
    if !(208..311).contains(&x) || !(178..197).contains(&y) {
        return None;
    }
    Some(((x - 208) * 16 - 176).clamp(0, width.saturating_sub(320).min(i32::MAX as usize) as i32))
}

/// Presentation input state belongs to the view and resets with each level.
#[derive(Default)]
pub struct Controls {
    pub held_rate: Option<i8>,
    pub dragging_map: bool,
    pub pointer: Option<(i32, i32)>,
    pub last_rate_step: Option<u64>,
    pub capture_pending: bool,
}
impl Controls {
    pub fn release(&mut self) {
        self.held_rate = None;
        self.dragging_map = false;
        self.last_rate_step = None;
    }
}

/// Selection affects presentation immediately; its optional logical cue is
/// recorded for the next control boundary, including while paused.
pub fn select_skill(session: &mut Session, view: &mut View, skill: SelectedSkill, emit_cue: bool) {
    if remaining(session, skill) == 0 {
        return;
    }
    view.selected_skill = skill;
    if emit_cue {
        session.notify_skill_selection();
    }
}

pub fn remaining(session: &Session, skill: SelectedSkill) -> u16 {
    match skill {
        SelectedSkill::Climber => session.remaining_climbers,
        SelectedSkill::Floater => session.remaining_floaters,
        SelectedSkill::Bomber => session.remaining_bombers,
        SelectedSkill::Blocker => session.remaining_blockers,
        SelectedSkill::Builder => session.remaining_builders,
        SelectedSkill::Basher => session.remaining_bashers,
        SelectedSkill::Miner => session.remaining_miners,
        SelectedSkill::Digger => session.remaining_diggers,
    }
}

/// Hold repeats once per control boundary, independently of polling/render frequency.
pub fn advance_input(
    session: &mut Session,
    view: &mut View,
    replaying: bool,
) -> Result<(), Box<dyn Error>> {
    if replaying || session.result().is_some() {
        view.panel.release();
        return Ok(());
    }
    let step = session.scheduler.step_state().control_step;
    if let Some(delta) = view.panel.held_rate
        && view.panel.last_rate_step != Some(step)
    {
        let rate = (i16::from(session.desired_rate()) + i16::from(delta)).clamp(
            i16::from(session.scheduler.config().initial_release_rate),
            99,
        ) as u8;
        if rate != session.desired_rate() {
            session.queue(lemmings_core::InputCommand::Custom(
                lemmings_core::StartupCommand::SetReleaseRate(rate),
            ))?;
        }
        view.panel.last_rate_step = Some(step);
    }
    Ok(())
}

/// Decoded indexed art; never stored in the simulation or no_std core.
pub struct Art {
    vga: bool,
    background: Vec<u8>,
    glyphs: Vec<Vec<u8>>,
    left: Vec<Vec<u8>>,
    right: Vec<Vec<u8>>,
}
fn pixels(image: UiImage<'_>) -> Vec<u8> {
    (0..usize::from(image.height()))
        .flat_map(|y| {
            (0..usize::from(image.width())).map(move |x| {
                let p = image.pixel(x, y).unwrap();
                if p.opaque { p.color } else { 128 }
            })
        })
        .collect()
}
impl Art {
    pub fn load(main: &MainDatFile<'_>) -> Result<Self, Box<dyn Error>> {
        let mut digits = vec![0; MainUiSection::PanelDigits.decoded_size()];
        let record = main.decode_ui_into(MainUiSection::PanelDigits, &mut digits)?;
        let background = pixels(record.images(MainUiImageKind::VgaSkillPanel)?.image(0)?);
        let images = record.images(MainUiImageKind::VgaPanelGlyphs)?;
        let glyphs = (0..38)
            .map(|n| images.image(n).map(pixels))
            .collect::<Result<_, _>>()?;
        let halves = |kind| -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
            let images = record.images(kind)?;
            (0..10).map(|n| Ok(pixels(images.image(n)?))).collect()
        };
        Ok(Self {
            vga: true,
            background,
            glyphs,
            left: halves(MainUiImageKind::DigitLeft)?,
            right: halves(MainUiImageKind::DigitRight)?,
        })
    }
    pub fn synthetic() -> Self {
        // Own simple button art/font; no DOS-derived pixels in offline fixtures.
        let mut background = vec![0; 320 * 40];
        for n in 0..12 {
            for y in 16..39 {
                for x in n * 16 + 1..n * 16 + 16 {
                    background[y * 320 + x] = 2;
                }
            }
            let c = "-+CFXKUBMDPN".chars().nth(n).unwrap_or('N');
            let rows = crate::render::glyph(c);
            for (y, row) in rows.iter().enumerate() {
                for x in 0..5 {
                    if row & (1 << (4 - x)) != 0 {
                        background[(y + 28) * 320 + n * 16 + 5 + x] = 3;
                    }
                }
            }
        }
        let glyphs = GLYPHS
            .chars()
            .map(|c| {
                let mut p = vec![0; 128];
                for (y, row) in crate::render::glyph(c).iter().enumerate() {
                    for x in 0..5 {
                        if row & (1 << (4 - x)) != 0 {
                            p[(y * 2 + 1) * 8 + x + 1] = 1;
                            p[(y * 2 + 2) * 8 + x + 1] = 1;
                        }
                    }
                }
                p
            })
            .collect();
        let digits: Vec<_> = (0..10)
            .map(|n| {
                let mut p = vec![128; 64];
                // Compact own 3-pixel decimal digit, duplicated into both halves.
                for (y, row) in crate::render::glyph(char::from(b'0' + n))
                    .iter()
                    .enumerate()
                {
                    for x in 0..3 {
                        if row & (1 << (4 - x * 2)) != 0 {
                            p[y * 8 + x] = 1;
                            p[y * 8 + x + 4] = 1;
                        }
                    }
                }
                p
            })
            .collect();
        Self {
            vga: false,
            background,
            glyphs,
            left: digits.clone(),
            right: digits,
        }
    }
}

fn put(frame: &mut [u8], x: usize, y: usize, color: [u8; 3]) {
    if x < 320 && y < 200 {
        frame[(y * 320 + x) * 3..(y * 320 + x) * 3 + 3].copy_from_slice(&color);
    }
}
fn border(frame: &mut [u8], x: usize, y: usize, w: usize, h: usize, c: [u8; 3]) {
    for dx in 0..w {
        put(frame, x + dx, y, c);
        put(frame, x + dx, y + h - 1, c);
    }
    for dy in 0..h {
        put(frame, x, y + dy, c);
        put(frame, x + w - 1, y + dy, c);
    }
}

pub fn info(session: &Session) -> String {
    let mut chars = [' '; 40];
    let alive = session
        .actors
        .iter()
        .filter(|a| a.state().removed.is_none() || a.state().particle_timer > 0)
        .count();
    let percent = lemmings_core::percentage(
        session.saved() as u32,
        session.scheduler.nuke_state().max_lemmings,
    );
    let seconds = session.scheduler.state().seconds_remaining;
    for (at, s) in [
        (14, "OUT ".to_owned()),
        (18, alive.to_string()),
        (23, "IN ".to_owned()),
        (26, format!("{percent:2}%")),
        (31, "TIME".to_owned()),
        (35, format!("{:2}-{:02}", seconds / 60, seconds % 60)),
    ] {
        for (i, c) in s.chars().enumerate().take(40 - at) {
            chars[at + i] = c;
        }
    }
    chars.into_iter().collect()
}

pub fn draw(frame: &mut [u8], assets: &Assets, session: &Session, view: &View) {
    let art = &assets.panel;
    let mut palette = assets.palette;
    if art.vga {
        // Original EXE c1a0..c1b7, VGA DAC values. These UI colors are
        // separate from the level's upper eight terrain colors.
        let high = [
            [0, 42, 0],
            [21, 63, 21],
            [21, 21, 21],
            [42, 0, 0],
            [42, 21, 0],
            [0, 42, 42],
            [63, 21, 63],
            [42, 0, 42],
        ];
        for (dst, raw) in palette[8..].iter_mut().zip(high) {
            *dst = raw.map(|v| v * 4);
        }
    }
    // Pinned SkillPanel.ReadBitmapFromStyle: panel color 7 aliases terrain color 8.
    palette[7] = assets.palette[8];
    for y in 0..40 {
        for x in 0..320 {
            put(
                frame,
                x,
                y + 160,
                palette[art.background[y * 320 + x] as usize],
            );
        }
    }
    let mut caption = info(session);
    caption.replace_range(..14, &crate::pointer::caption(session, view));
    for (i, c) in caption.chars().enumerate() {
        if let Some(n) = GLYPHS.chars().position(|g| g == c) {
            for y in 0..16 {
                for x in 0..8 {
                    put(
                        frame,
                        i * 8 + x,
                        160 + y,
                        palette[{
                            let c = usize::from(art.glyphs[n][y * 8 + x]);
                            if art.vga { (c & 3) | ((c & 4) << 1) } else { c }
                        }],
                    );
                }
            }
        }
    }
    let counts = [
        u16::from(assets.config.initial_release_rate),
        u16::from(session.scheduler.state().release_rate),
        session.remaining_climbers,
        session.remaining_floaters,
        session.remaining_bombers,
        session.remaining_blockers,
        session.remaining_builders,
        session.remaining_bashers,
        session.remaining_miners,
        session.remaining_diggers,
    ];
    for (i, count) in counts.into_iter().enumerate() {
        let n = usize::from(count.min(99));
        for y in 0..8 {
            for x in 0..8 {
                let on = n != 0
                    && if x < 4 {
                        art.left[n / 10][y * 8 + x] != 128
                    } else {
                        art.right[n % 10][y * 8 + x] != 128
                    };
                put(
                    frame,
                    i * 16 + 4 + x,
                    177 + y,
                    if on { palette[3] } else { [0; 3] },
                );
            }
        }
        if n == 0 {
            for y in 0..8 {
                for x in 0..8 {
                    put(frame, i * 16 + 4 + x, 177 + y, [240, 208, 208]);
                }
            }
        }
    }
    let selected = SKILLS
        .iter()
        .position(|s| *s == view.selected_skill)
        .unwrap()
        + 2;
    border(frame, selected * 16, 176, 16, 24, palette[3]);
    if session.scheduler.step_state().paused {
        border(frame, 161, 176, 15, 24, palette[3]);
    }
    // DOS EXE 7b59..7b75 counts >8 occupied bits in a horizontal 16px group.
    // Canonical world omits DOS horizontal16/vertical8 padding.
    // 7a6c clears the top row; refresh every column from current occupancy.
    for my in 0..20 {
        for mx in 0..104 {
            let solid = minimap_solid(
                session.render_terrain(),
                assets.width,
                assets.height,
                mx,
                my,
            );
            put(
                frame,
                208 + mx,
                178 + my,
                if solid { assets.palette[7] } else { [0; 3] },
            );
        }
    }
    for actor in &session.actors {
        let a = actor.state();
        if (a.removed.is_none() || a.particle_timer > 0)
            && a.x >= 0
            && a.y >= 8
            && a.x / 16 + 1 < 104
            && a.y / 8 - 1 < 20
        {
            put(
                frame,
                209 + a.x as usize / 16,
                177 + a.y as usize / 8,
                palette[2],
            );
        }
    }
    // DOS 7b7a..7cd7 draws 24×20; pinned Lemmix uses 25×20.
    let left = (209 + view.camera.max(0) as usize / 16).min(311);
    border(frame, left, 178, 24.min(312 - left), 20, palette[3]);
}

fn minimap_solid(terrain: &[u8], width: usize, height: usize, mx: usize, my: usize) -> bool {
    if mx == 0 || my == 0 {
        return false;
    }
    let x = (mx - 1) * 16;
    let y = (my + 1) * 8;
    y < height
        && x < width
        && terrain[y * width + x..y * width + (x + 16).min(width)]
            .iter()
            .filter(|p| **p != 0)
            .count()
            > 8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimap_uses_majority_of_sixteen_pixels_and_reflects_edits() {
        let mut terrain = vec![0; 64 * 160];
        // Map (2,1) samples native x16..32 at y16. Eight pixels stay empty.
        terrain[16 * 64 + 16..16 * 64 + 24].fill(1);
        assert!(!minimap_solid(&terrain, 64, 160, 2, 1));
        terrain[16 * 64 + 24] = 1;
        assert!(minimap_solid(&terrain, 64, 160, 2, 1));
        terrain[16 * 64 + 16] = 0;
        assert!(!minimap_solid(&terrain, 64, 160, 2, 1));
        assert!(!minimap_solid(&terrain, 64, 160, 0, 1));
        assert!(!minimap_solid(&terrain, 64, 160, 2, 0));
        assert!(!minimap_solid(&terrain, 64, 160, 2, 19));
        assert!(!minimap_solid(&terrain, 64, 160, 104, 1));
    }

    #[test]
    fn counters_clamp_clear_and_selection_does_not_leave_a_stale_frame() {
        let mut assets = Assets::synthetic();
        assets.palette[2] = [0, 176, 0];
        let mut run =
            crate::lifecycle::LevelRun::new(assets, 1, None, crate::pacing::Pacing::Dos).unwrap();
        run.session.remaining_climbers = 99;
        let a = run.view.frame(&run.assets, &run.session).unwrap();
        run.session.remaining_climbers = 500;
        assert_eq!(run.view.frame(&run.assets, &run.session).unwrap(), a);
        run.session.remaining_climbers = 0;
        let empty = run.view.frame(&run.assets, &run.session).unwrap();
        for y in 177..185 {
            for x in 36..44 {
                assert_eq!(
                    &empty[(y * 320 + x) * 3..(y * 320 + x) * 3 + 3],
                    &[240, 208, 208]
                );
            }
        }
        run.view.selected_skill = SelectedSkill::Climber;
        let selected = run.view.frame(&run.assets, &run.session).unwrap();
        assert_eq!(
            &selected[(176 * 320 + 32) * 3..(176 * 320 + 32) * 3 + 3],
            &run.assets.palette[3]
        );
        assert_ne!(
            &empty[(176 * 320 + 32) * 3..(176 * 320 + 32) * 3 + 3],
            &run.assets.palette[3]
        );
        assert_ne!(
            &selected[(176 * 320 + 144) * 3..(176 * 320 + 144) * 3 + 3],
            &run.assets.palette[3]
        );
        let state = run.session.snapshot();
        run.view.scroll(16, run.assets.width);
        run.view.frame(&run.assets, &run.session).unwrap();
        assert_eq!(run.session.snapshot(), state);
    }

    #[test]
    fn buttons_have_half_open_bounds_and_gaps_never_assign() {
        assert_eq!(hit(1, 176), Some(Button::Slower));
        assert_eq!(hit(14, 197), Some(Button::Slower));
        assert_eq!(hit(17, 176), Some(Button::Faster));
        assert_eq!(hit(33, 176), Some(Button::Skill(SelectedSkill::Climber)));
        assert_eq!(hit(145, 197), Some(Button::Skill(SelectedSkill::Digger)));
        assert_eq!(hit(161, 176), Some(Button::Pause));
        assert_eq!(hit(177, 176), Some(Button::Nuke));
        for (x, y) in [
            (0, 176),
            (15, 176),
            (16, 176),
            (1, 175),
            (1, 198),
            (-1, 180),
            (192, 180),
            (320, 180),
        ] {
            assert_eq!(hit(x, y), None, "{x},{y}");
        }
    }

    #[test]
    fn minimap_click_centers_and_clamps_with_exclusive_edges() {
        assert_eq!(minimap_camera(208, 178, 1584), Some(0));
        assert_eq!(minimap_camera(250, 180, 1584), Some(496));
        assert_eq!(minimap_camera(310, 196, 1584), Some(1264));
        assert_eq!(minimap_camera(250, 180, 100), Some(0));
        for (x, y) in [
            (207, 180),
            (311, 180),
            (250, 177),
            (250, 197),
            (i32::MAX, 180),
        ] {
            assert_eq!(minimap_camera(x, y, 1584), None);
        }
    }
}

#[cfg(test)]
mod availability_tests {
    use super::*;
    #[test]
    fn empty_skill_cannot_replace_an_available_selection() {
        let mut run = crate::LevelRun::new(
            Assets::synthetic_builder(),
            1,
            None,
            crate::pacing::Pacing::Reference,
        )
        .unwrap();
        run.session.remaining_floaters = 10;
        run.session.remaining_diggers = 0;
        select_skill(
            &mut run.session,
            &mut run.view,
            SelectedSkill::Floater,
            true,
        );
        select_skill(&mut run.session, &mut run.view, SelectedSkill::Digger, true);
        assert_eq!(run.view.selected_skill, SelectedSkill::Floater);
        assert_eq!(remaining(&run.session, SelectedSkill::Digger), 0);
        assert_eq!(run.session.remaining_floaters, 10);
    }
}
