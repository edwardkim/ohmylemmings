//! DOS-original AdLib metadata. See issue-72_adlib_catalog.md for pinned sources.
//! This checks initial reads, not command-stream termination or OPL timing.
use crate::{DatError, DatSection, DatSections, DecompressError};
use core::fmt;
use lemmings_core::LevelId;

pub const ADLIB_IMAGE_SIZE: usize = 22125;
pub const ADLIB_MUSIC_COUNT: usize = 21;
pub const ADLIB_EFFECT_COUNT: usize = 18;
const MUSIC_TABLE: usize = 2926;
const MUSIC_START: usize = MUSIC_TABLE + ADLIB_MUSIC_COUNT * 2;
const SOUND_INSTRUMENTS: usize = 21731;
const EFFECT_TABLE: usize = 21989;
const EFFECT_START: usize = EFFECT_TABLE + ADLIB_EFFECT_COUNT * 2;
const MUSIC_CHANNELS: usize = 8; // DOS channel 8 is used for effects.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdlibError {
    Size {
        actual: usize,
    },
    Sections,
    Dat(DatError),
    Decompress(DecompressError),
    MusicIndex {
        index: usize,
    },
    EffectIndex {
        index: usize,
    },
    ChannelCount {
        track: usize,
        count: u8,
    },
    ZeroSampleRate {
        track: usize,
    },
    Range {
        offset: usize,
        count: usize,
        start: usize,
        end: usize,
    },
}
impl fmt::Display for AdlibError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid DOS AdLib resource: {self:?}")
    }
}
impl core::error::Error for AdlibError {}

/// A framed single-section ADLIB.DAT, decoded into caller-owned memory.
#[derive(Clone, Copy, Debug)]
pub struct AdlibFile<'a> {
    section: DatSection<'a>,
}
impl<'a> AdlibFile<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, AdlibError> {
        let mut sections = DatSections::new(bytes);
        let section = sections
            .next()
            .ok_or(AdlibError::Sections)?
            .map_err(AdlibError::Dat)?;
        if sections
            .next()
            .transpose()
            .map_err(AdlibError::Dat)?
            .is_some()
        {
            return Err(AdlibError::Sections);
        }
        if usize::from(section.header().decompressed_size) != ADLIB_IMAGE_SIZE {
            return Err(AdlibError::Size {
                actual: usize::from(section.header().decompressed_size),
            });
        }
        Ok(Self { section })
    }
    pub fn decode_into(self, output: &mut [u8]) -> Result<AdlibImage<'_>, AdlibError> {
        self.section
            .decompress_into(output)
            .map_err(AdlibError::Decompress)?;
        AdlibImage::parse(output)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdlibChannel {
    pub program_offset: usize,
    pub first_bar_offset: usize,
}
/// Raw header values; sample_rate_factor and wait_cycles are not wall time.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdlibMusic {
    pub header_offset: usize,
    pub sample_rate_factor: u16,
    pub instrument_offset: usize,
    pub wait_cycles: u8,
    channels: [AdlibChannel; MUSIC_CHANNELS],
    channel_count: usize,
}
impl AdlibMusic {
    pub fn channels(&self) -> &[AdlibChannel] {
        &self.channels[..self.channel_count]
    }
}

/// Borrowed DOS-original decoded image. No allocation, device, or game clock.
///
/// ```
/// use lemmings_resources::AdlibImage;
/// assert!(AdlibImage::parse(&[]).is_err());
/// ```
#[derive(Clone, Copy, Debug)]
pub struct AdlibImage<'a> {
    bytes: &'a [u8],
}
impl<'a> AdlibImage<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, AdlibError> {
        if bytes.len() != ADLIB_IMAGE_SIZE {
            return Err(AdlibError::Size {
                actual: bytes.len(),
            });
        }
        let image = Self { bytes };
        for index in 0..ADLIB_MUSIC_COUNT {
            image.music(index)?;
        }
        for index in 0..ADLIB_EFFECT_COUNT {
            image.effect_offset(index)?;
        }
        Ok(image)
    }
    pub const fn as_bytes(self) -> &'a [u8] {
        self.bytes
    }
    pub fn music(self, index: usize) -> Result<AdlibMusic, AdlibError> {
        if index >= ADLIB_MUSIC_COUNT {
            return Err(AdlibError::MusicIndex { index });
        }
        let h = self.word(MUSIC_TABLE + index * 2);
        music_range(h, 6)?;
        let count = self.bytes[h + 5];
        if count == 0 || usize::from(count) > MUSIC_CHANNELS {
            return Err(AdlibError::ChannelCount {
                track: index,
                count,
            });
        }
        music_range(h, 6 + usize::from(count) * 2)?;
        let factor = self.word(h) as u16;
        if factor == 0 {
            return Err(AdlibError::ZeroSampleRate { track: index });
        }
        let instrument_offset = MUSIC_TABLE + self.word(h + 2);
        music_range(instrument_offset, 16)?;
        let mut channels = [AdlibChannel {
            program_offset: 0,
            first_bar_offset: 0,
        }; MUSIC_CHANNELS];
        for (i, channel) in channels.iter_mut().take(usize::from(count)).enumerate() {
            let program_offset = MUSIC_TABLE + self.word(h + 6 + i * 2);
            music_range(program_offset, 2)?;
            let first_bar_offset = MUSIC_TABLE + self.word(program_offset);
            music_range(first_bar_offset, 1)?;
            *channel = AdlibChannel {
                program_offset,
                first_bar_offset,
            };
        }
        Ok(AdlibMusic {
            header_offset: h,
            sample_rate_factor: factor,
            instrument_offset,
            wait_cycles: self.bytes[h + 4],
            channels,
            channel_count: usize::from(count),
        })
    }
    /// Pinned Lemmix displayed catalog order; DOS playback verification is separate.
    pub fn music_for_level(self, level: LevelId) -> Result<AdlibMusic, AdlibError> {
        self.music(level.global_index() % ADLIB_MUSIC_COUNT)
    }
    pub fn effect_offset(self, index: usize) -> Result<usize, AdlibError> {
        if index >= ADLIB_EFFECT_COUNT {
            return Err(AdlibError::EffectIndex { index });
        }
        let offset = self.word(EFFECT_TABLE + index * 2);
        range(offset, 1, EFFECT_START, ADLIB_IMAGE_SIZE)?;
        Ok(offset)
    }
    // All callers establish a complete word range before reading.
    fn word(self, offset: usize) -> usize {
        usize::from(u16::from_le_bytes([
            self.bytes[offset],
            self.bytes[offset + 1],
        ]))
    }
}
fn music_range(offset: usize, count: usize) -> Result<(), AdlibError> {
    range(offset, count, MUSIC_START, SOUND_INSTRUMENTS)
}
fn range(offset: usize, count: usize, start: usize, end: usize) -> Result<(), AdlibError> {
    if offset < start || offset.checked_add(count).is_none_or(|n| n > end) {
        return Err(AdlibError::Range {
            offset,
            count,
            start,
            end,
        });
    }
    Ok(())
}
