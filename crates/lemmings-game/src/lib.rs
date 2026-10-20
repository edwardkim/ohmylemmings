//! Shared game asset assembly, CPU rendering and scene transitions.
//!
//! Hosts supply named bytes, tick the shared session, translate their input to
//! logical coordinates/commands, and present RGB frames. This crate performs no
//! filesystem, window, system-clock or browser access. Resource parsing remains
//! in `lemmings-resources`; simulation remains in `lemmings-session`/`lemmings-core`.
pub mod assets;
pub mod lifecycle;
pub mod panel;
pub mod pointer;
pub mod render;
pub mod sound;
pub use assets::{AssetSource, Assets, MissingAsset};
pub use lemmings_session as session;
pub use lemmings_session::pacing;
pub use lifecycle::{LevelRun, ResultAction};
pub use render::View;
