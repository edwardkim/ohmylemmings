//! SDL-free bridge between logical screen coordinates and core selection.
use crate::{
    render::{SelectedSkill, View},
    session::Session,
};
use lemmings_core::AssignmentSkill;

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
pub use lemmings_session::command;
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
