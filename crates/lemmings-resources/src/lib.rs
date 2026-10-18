#![no_std]
//! Borrowed, checked input for DOS Lemmings resources.
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
mod reader;

pub use dat::{DatError, DatHeader, DatSection, DatSections};
pub use reader::{ByteReader, ReadError};
