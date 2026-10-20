//! Owned deterministic sessions shared by native and future browser hosts.
//!
//! The host supplies decoded data and elapsed durations; this crate never opens
//! files, reads a system clock, renders, or calls SDL/browser APIs. The leaf
//! core remains allocation-free and `no_std`; this orchestration crate uses std.
//!
//! ```
//! use lemmings_core::{AssignmentSkill, StartupCommand};
//! use lemmings_session::command;
//! assert_eq!(command(AssignmentSkill::Digger, 7), StartupCommand::AssignDigger(7));
//! ```

pub mod pacing;
mod session;
use lemmings_core::{AssignmentSkill, StartupCommand};
pub use session::Session;

/// Turn a resolved skill/actor ID into the command consumed at a control boundary.
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

#[cfg(test)]
mod fixtures;
