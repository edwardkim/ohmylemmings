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
mod masks;
mod objects;
mod odd;
mod reader;
mod sprites;
mod terrain;
mod ui;
mod vgagr;
mod vgaspec;

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

pub use vgagr::{
    ImageKind, IndexedPixel, ObjectFrames, PlanarImage, VgagrError, VgagrFile, VgagrImages,
};

pub use vgaspec::{
    VGASPEC_HEIGHT, VGASPEC_PLANAR_SIZE, VGASPEC_WIDTH, VgaSpecError, VgaSpecFile, VgaSpecImage,
    VgaSpecRecord,
};

pub use terrain::{
    LEVEL_TERRAIN_HEIGHT, LEVEL_TERRAIN_PIXEL_COUNT, LEVEL_TERRAIN_WIDTH, LevelTerrain,
    SpecialTerrain, TerrainComposeError, TerrainGraphics, TerrainSource, compose_terrain,
};

pub use sprites::{
    LEMMING_ANIMATION_COUNT, LEMMING_SPRITE_DATA_SIZE, LemmingAnimationInfo, LemmingFrames,
    LemmingSpriteError, LemmingSpriteKind, LemmingSprites, MAIN_DAT_SECTION_COUNT, MainDatFile,
    SpriteDirection, SpriteFrame,
};

pub use ui::{
    MAIN_UI_IMAGE_KINDS, MainUiError, MainUiImageKind, MainUiRecord, MainUiSection, UiImage,
    UiImageInfo, UiImages, UiPalette, UiTransparency,
};

pub use masks::{
    MAIN_MASK_KINDS, MAIN_MASK_RECORD_SIZE, MAIN_MASK_SECTION, MainMaskError, MainMaskKind,
    MainMasks, MaskBitmap, MaskFrames, MaskInfo,
};

pub use objects::{
    LevelScene, ObjectComposeError, ObjectFrameSelection, ObjectGraphics, ScenePixel,
    compose_objects,
};

mod particles;
pub use particles::{EXPLOSION_PARTICLE_DATA_SIZE, ExplosionParticles, ParticleDataError};

mod adlib;
pub use adlib::{
    ADLIB_EFFECT_COUNT, ADLIB_IMAGE_SIZE, ADLIB_MUSIC_COUNT, AdlibChannel, AdlibError, AdlibFile,
    AdlibImage, AdlibMusic,
};

mod adlib_stream;
pub use adlib_stream::{
    ADLIB_COMMAND_BUDGET, AdlibPlayer, AdlibState, AdlibStreamError, AdlibWrite,
};

mod sound;
pub use sound::{AdlibCommand, DosMusic, SoundMappingError, adlib_effect_for_cue};
