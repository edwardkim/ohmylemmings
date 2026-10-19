//! SDL-free bridge between logical screen coordinates and core selection.
use crate::{
    render::{SelectedSkill, View},
    session::Session,
};
use lemmings_core::{AssignmentSkill, StartupCommand};

pub fn skill(skill: SelectedSkill) -> AssignmentSkill {
    match skill {
        SelectedSkill::Climber => AssignmentSkill::Climber,
        SelectedSkill::Floater => AssignmentSkill::Floater,
        SelectedSkill::Bomber => AssignmentSkill::Bomber,
        SelectedSkill::Blocker => AssignmentSkill::Blocker,
        SelectedSkill::Builder => AssignmentSkill::Builder,
        SelectedSkill::Basher => AssignmentSkill::Basher,
        SelectedSkill::Miner => AssignmentSkill::Miner,
        SelectedSkill::Digger => AssignmentSkill::Digger,
    }
}
pub fn command(skill: AssignmentSkill, id: u32) -> StartupCommand {
    match skill {
        AssignmentSkill::Climber => StartupCommand::AssignClimber(id),
        AssignmentSkill::Floater => StartupCommand::AssignFloater(id),
        AssignmentSkill::Bomber => StartupCommand::AssignBomber(id),
        AssignmentSkill::Blocker => StartupCommand::AssignBlocker(id),
        AssignmentSkill::Builder => StartupCommand::AssignBuilder(id),
        AssignmentSkill::Basher => StartupCommand::AssignBasher(id),
        AssignmentSkill::Miner => StartupCommand::AssignMiner(id),
        AssignmentSkill::Digger => StartupCommand::AssignDigger(id),
    }
}
pub fn refresh(session: &Session, view: &mut View) {
    if session.result().is_some() {
        view.selection = Default::default();
        view.right_mouse = false;
        return;
    }
    let point = view
        .panel
        .pointer
        .filter(|(x, y)| (0..320).contains(x) && (0..160).contains(y))
        .map(|(x, y)| (i64::from(x) + i64::from(view.camera), i64::from(y)));
    view.selection.update(
        session.actors.iter().map(|a| a.state()),
        point,
        view.right_mouse,
    );
}
pub fn caption(session: &Session, view: &View) -> String {
    let Some(a) = view.selection.hits.primary.and_then(|id| {
        session
            .actors
            .iter()
            .map(|a| a.state())
            .find(|a| a.id == id)
    }) else {
        return " ".repeat(14);
    };
    use lemmings_core::MotionAction::*;
    let label = if a.is_climber && a.is_floater {
        "ATHLETE"
    } else if a.is_climber {
        "CLIMBER"
    } else if a.is_floater {
        "FLOATER"
    } else {
        match a.action {
            Walking => "WALKER",
            Jumping => "JUMPER",
            Falling => "FALLER",
            Splatting => "SPLATTER",
            Digging => "DIGGER",
            Bashing => "BASHER",
            Mining => "MINER",
            Building => "BUILDER",
            Shrugging => "SHRUGGER",
            Blocking => "BLOCKER",
            OhNo => "OHNOER",
            Exploding => "BOMBER",
            Climbing => "CLIMBER",
            Hoisting => "HOISTER",
            Floating => "FLOATER",
            Exiting => "EXITER",
            Drowning => "DROWNER",
            Vaporizing => "FRIER",
        }
    };
    format!("{:<14}", format!("{label} {}", view.selection.hits.count))
        .chars()
        .take(14)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{assets::Assets, lifecycle::LevelRun, pacing::Pacing};
    use lemmings_core::*;
    use sdl2::{
        event::{Event, WindowEvent},
        mouse::{MouseButton, MouseState},
    };
    fn run() -> LevelRun {
        let mut assets = Assets::synthetic_builder();
        assets.config.total_lemmings = 1;
        let mut r = LevelRun::new(assets, 1, None, Pacing::Reference).unwrap();
        for _ in 0..80 {
            r.session.tick().unwrap();
        }
        let mut s = r.session.actors[0].state();
        s.action = MotionAction::Walking;
        s.x = 60;
        s.y = 100;
        s.fallen = 0;
        s.frame = 0;
        r.session.actors[0] = Actor::from_state(s, MotionRules::default()).unwrap();
        s.id = 1;
        s.x = 200;
        s.action = MotionAction::Building;
        s.bricks_left = 12;
        r.session
            .actors
            .push(Actor::from_state(s, MotionRules::default()).unwrap());
        r
    }
    fn motion(x: i32, right: bool) -> Event {
        Event::MouseMotion {
            timestamp: 0,
            window_id: 0,
            which: 0,
            mousestate: MouseState::from_sdl_state(if right { 4 } else { 0 }),
            x,
            y: 95,
            xrel: 0,
            yrel: 0,
        }
    }
    fn click(x: i32) -> Event {
        Event::MouseButtonDown {
            timestamp: 0,
            window_id: 0,
            which: 0,
            mouse_btn: MouseButton::Left,
            clicks: 1,
            x,
            y: 95,
        }
    }
    fn handle(r: &mut LevelRun, event: Event, replay: bool) {
        crate::input::handle(event, &mut r.session, &mut r.view, r.assets.width, replay).unwrap();
    }
    #[test]
    fn raw_right_motion_remotely_assigns_saved_id_and_focus_loss_clears_it() {
        let mut r = run();
        r.view.selected_skill = SelectedSkill::Bomber;
        handle(&mut r, motion(60, false), false);
        assert_eq!(caption(&r.session, &r.view), "WALKER 1      ");
        handle(&mut r, motion(200, true), false);
        assert_eq!(r.view.selection.hits.primary, None);
        handle(&mut r, click(200), true);
        assert!(r.session.inputs.is_empty());
        handle(&mut r, click(200), false);
        assert!(matches!(
            r.session.inputs[0].command,
            InputCommand::Custom(StartupCommand::AssignPointer(PointerAssignment {
                primary: 0,
                right_click: true,
                ..
            }))
        ));
        r.session.tick().unwrap();
        assert!(r.session.actors[0].state().explosion_timer > 0);
        assert_eq!(r.session.actors[1].state().explosion_timer, 0);
        handle(
            &mut r,
            Event::Window {
                timestamp: 0,
                window_id: 0,
                win_event: WindowEvent::FocusLost,
            },
            false,
        );
        handle(&mut r, motion(200, true), false);
        handle(&mut r, click(200), false);
        assert_eq!(r.session.inputs.len(), 1);
        assert!(crate::input::parse_replay("80 0 AssignPointer Bomber 0 - true", None).is_ok());
        for line in [
            "0 0 AssignPointer Bomber 0 - yes",
            "0 0 AssignPointer X 0 - true",
            "0 0 AssignPointer Bomber 0 bad true",
        ] {
            assert!(crate::input::parse_replay(line, None).is_err());
        }
    }
    #[test]
    fn overlap_caption_priority_and_fallback_are_resolved_at_command_boundary() {
        let mut r = run();
        let mut a = r.session.actors[1].state();
        a.x = 60;
        r.session.actors[1] = Actor::from_state(a, MotionRules::default()).unwrap();
        handle(&mut r, motion(60, false), false);
        assert_eq!(caption(&r.session, &r.view), "BUILDER 2     ");
        r.view.selected_skill = SelectedSkill::Builder;
        handle(&mut r, click(60), false);
        // The captured primary becomes a Basher before this ordered command runs.
        // It now accepts Builder, so the fallback must not be chosen at polling time.
        a.action = MotionAction::Bashing;
        a.bricks_left = 0;
        r.session.actors[1] = Actor::from_state(a, MotionRules::default()).unwrap();
        r.session.tick().unwrap();
        assert_eq!(r.session.actors[1].state().action, MotionAction::Building);
        assert_ne!(r.session.actors[0].state().action, MotionAction::Building);
    }
    #[test]
    fn trap_ghost_blocker_uses_inventory_and_map_but_remains_removed() {
        let mut r = run();
        r.view.selected_skill = SelectedSkill::Blocker;
        handle(&mut r, motion(60, false), false);
        let mut a = r.session.actors[0].state();
        a.removed = Some(RemovalReason::Trapped);
        a.object_below = 0;
        r.session.actors[0] = Actor::from_state(a, MotionRules::default()).unwrap();
        handle(&mut r, motion(200, true), false);
        handle(&mut r, click(200), false);
        let before = r.session.remaining_blockers;
        r.session.tick().unwrap();
        assert_eq!(r.session.remaining_blockers, before - 1);
        let a = r.session.actors[0].state();
        assert_eq!(a.action, MotionAction::Blocking);
        assert_eq!(a.removed, Some(RemovalReason::Trapped));
        let map = r.session.object_map_data().to_vec();
        for _ in 0..10 {
            r.session.tick().unwrap();
        }
        assert_eq!(r.session.actors[0].state(), a);
        assert_eq!(r.session.object_map_data(), map);
    }
}
