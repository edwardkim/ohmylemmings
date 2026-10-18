#![no_std]
//! Borrowed, checked input primitives for future DOS resource parsers.
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

mod reader;

pub use reader::{ByteReader, ReadError};
