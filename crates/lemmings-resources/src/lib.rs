#![no_std]
//! Checked DOS resources and original level routing without allocation.
//!
//! ```
//! use lemmings_resources::ByteReader;
//!
//! let mut reader = ByteReader::new(&[0x12, 0x34, 0xff]);
//! assert_eq!(reader.read_u16_be()?, 0x1234);
//! assert!(reader.read_u16_be().is_err());
//! assert_eq!(reader.position(), 2);
//! assert_eq!(reader.read_bytes(1)?, &[0xff]);
//! # Ok::<(), lemmings_resources::ReadError>(())
//! ```

mod catalog;
mod dat;
mod decompress;
mod ground;
mod level;
mod odd;
mod reader;

pub use catalog::{CatalogEntry, CatalogError, DosLevelFile, ResolvedLevel};
pub use dat::{DatError, DatHeader, DatSection, DatSections};
pub use decompress::DecompressError;
pub use level::{
    LEVEL_RECORD_SIZE, LevelError, LevelField, LevelHeader, LevelProperties, LevelRecord,
    ObjectFlags, ObjectPlacement, SteelArea, TerrainFlags, TerrainPlacement,
};
pub use odd::{ODD_RECORD_SIZE, OddError, OddRecord, OddTable};
pub use reader::{ByteReader, ReadError};

pub use ground::{
    GROUND_OBJECT_COUNT, GROUND_RECORD_SIZE, GROUND_TERRAIN_COUNT, GroundError, GroundObject,
    GroundPalette, GroundRecord, GroundTerrain, ObjectTrigger, Rgb6,
};
