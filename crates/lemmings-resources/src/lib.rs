#![no_std]
//! Checked DOS DAT and base-level parsing with allocation-free decompression.
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

mod dat;
mod decompress;
mod level;
mod reader;

pub use dat::{DatError, DatHeader, DatSection, DatSections};
pub use decompress::DecompressError;
pub use level::{
    LEVEL_RECORD_SIZE, LevelError, LevelField, LevelHeader, LevelRecord, ObjectFlags,
    ObjectPlacement, SteelArea, TerrainFlags, TerrainPlacement,
};
pub use reader::{ByteReader, ReadError};
