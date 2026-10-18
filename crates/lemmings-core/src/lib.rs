#![no_std]
//! Domain types and checked terrain views shared by simulation and adapters.
//!
//! ```
//! use lemmings_core::{Difficulty, LevelId};
//!
//! let first = LevelId::new(Difficulty::Fun, 1)?;
//! assert_eq!(first.index(), 0);
//! assert_eq!(first.global_index(), 0);
//! assert!(LevelId::new(Difficulty::Fun, 0).is_err());
//! # Ok::<(), lemmings_core::InvalidLevelNumber>(())
//! ```

mod level;
mod terrain;

pub use level::{Difficulty, InvalidLevelNumber, LevelId};

pub use terrain::{TerrainMask, TerrainMaskError};
