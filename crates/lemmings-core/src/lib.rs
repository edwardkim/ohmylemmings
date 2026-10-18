#![no_std]
//! Domain types shared by the future simulation and its platform adapters.
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

pub use level::{Difficulty, InvalidLevelNumber, LevelId};
