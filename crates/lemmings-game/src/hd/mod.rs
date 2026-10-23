//! Shared HD layout, read-only presentation and host input routing. No platform
//! I/O, wall clock or simulation mutation.
//!
//! Design coordinates are 1920×1080: the 320×160 game viewport at 6× and a
//! separately laid out 120-pixel toolbar. Hosts use the same unit for viewport
//! sizes and pointer coordinates (CSS pixels or drawable pixels, never mixed).
//! Pointer coordinates are local to the output surface's top-left. Subtract the
//! browser/client offset first; do not scale already-logical SDL events twice.
//! See `mydocs/plans/issue-140_hd_presentation.md` and issue #138's art contract.
pub mod assets;
pub mod input;
mod layout;
pub mod scene;
mod snapshot;

pub use layout::{
    Control, ControlRegion, GAME_HEIGHT, HEIGHT, Hit, Rect, SCALE, SpriteRect, Viewport, WIDTH,
    control_regions, sprite_rect,
};
pub use snapshot::{
    ActorSnapshot, HudSnapshot, Snapshot, SnapshotError, TerrainSnapshot, snapshot,
};

pub mod catalog;
