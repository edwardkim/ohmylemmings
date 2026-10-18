use crate::{DecompressError, IndexedPixel, MainDatFile};
use core::fmt;

/// Original DOS VGA UI sections. Tags select fixed layouts, not file provenance.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MainUiSection {
    PanelDigits,
    MenuImages,
    MenuExtras,
    SkillPanel,
}
impl MainUiSection {
    pub const ALL: [Self; 4] = [
        Self::PanelDigits,
        Self::MenuImages,
        Self::MenuExtras,
        Self::SkillPanel,
    ];
    pub const fn part_index(self) -> usize {
        match self {
            Self::PanelDigits => 2,
            Self::MenuImages => 3,
            Self::MenuExtras => 4,
            Self::SkillPanel => 6,
        }
    }
    pub const fn decoded_size(self) -> usize {
        match self {
            Self::PanelDigits => 8384,
            Self::MenuImages => 61968,
            Self::MenuExtras => 36080,
            Self::SkillPanel => 8224,
        }
    }
}

/// Disk image families. Scrolls have 16 frames, signs 4, menu glyphs 94,
/// panel glyphs 38 and each digit half 10. No generated spaces or blank counters.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum MainUiImageKind {
    Background,
    Logo,
    Play,
    LevelCode,
    Music,
    Rating,
    Exit,
    Navigation,
    MusicIcon,
    FxIcon,
    ScrollLeft,
    ScrollRight,
    Reel,
    RatingSigns,
    MenuGlyphs,
    SkillPanel,
    PanelGlyphs,
    DigitRight,
    DigitLeft,
}
pub const MAIN_UI_IMAGE_KINDS: [MainUiImageKind; 19] = [
    MainUiImageKind::Background,
    MainUiImageKind::Logo,
    MainUiImageKind::Play,
    MainUiImageKind::LevelCode,
    MainUiImageKind::Music,
    MainUiImageKind::Rating,
    MainUiImageKind::Exit,
    MainUiImageKind::Navigation,
    MainUiImageKind::MusicIcon,
    MainUiImageKind::FxIcon,
    MainUiImageKind::ScrollLeft,
    MainUiImageKind::ScrollRight,
    MainUiImageKind::Reel,
    MainUiImageKind::RatingSigns,
    MainUiImageKind::MenuGlyphs,
    MainUiImageKind::SkillPanel,
    MainUiImageKind::PanelGlyphs,
    MainUiImageKind::DigitRight,
    MainUiImageKind::DigitLeft,
];
impl MainUiImageKind {
    pub const fn info(self) -> UiImageInfo {
        LAYOUTS[self as usize]
    }
}

/// Opacity policy of the resource, independent of RGB or palette index remapping.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiTransparency {
    Opaque,
    ColorZero,
}
/// Caller palette role. No palette is embedded in these image sections.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiPalette {
    Menu,
    Level,
    Monochrome,
}

/// Fixed metadata; offsets/strides refer to the decompressed section. Digits
/// are interleaved right/left, so their stride exceeds the image byte size.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiImageInfo {
    pub kind: MainUiImageKind,
    pub section: MainUiSection,
    pub offset: usize,
    pub width: u16,
    pub height: u16,
    pub planes: u8,
    pub image_count: u8,
    pub stride: usize,
    pub transparency: UiTransparency,
    pub palette: UiPalette,
}
impl UiImageInfo {
    pub fn image_size(self) -> usize {
        usize::from(self.width) * usize::from(self.height) / 8 * usize::from(self.planes)
    }
}

// Pinned Lemmix GameScreen.Menu/Base, Game.SkillPanel and TS MainImageSprites/
// SkillPanelSprites. Policies/differences: mydocs/tech_insights/issue-17_main_ui.md.
const LAYOUTS: [UiImageInfo; 19] = [
    UiImageInfo {
        kind: MainUiImageKind::Background,
        section: MainUiSection::MenuImages,
        offset: 0x0,
        width: 320,
        height: 104,
        planes: 2,
        image_count: 1,
        stride: 8320,
        transparency: UiTransparency::Opaque,
        palette: UiPalette::Menu,
    },
    UiImageInfo {
        kind: MainUiImageKind::Logo,
        section: MainUiSection::MenuImages,
        offset: 0x2080,
        width: 632,
        height: 94,
        planes: 4,
        image_count: 1,
        stride: 29704,
        transparency: UiTransparency::ColorZero,
        palette: UiPalette::Menu,
    },
    UiImageInfo {
        kind: MainUiImageKind::Play,
        section: MainUiSection::MenuImages,
        offset: 0x9488,
        width: 120,
        height: 61,
        planes: 4,
        image_count: 1,
        stride: 3660,
        transparency: UiTransparency::ColorZero,
        palette: UiPalette::Menu,
    },
    UiImageInfo {
        kind: MainUiImageKind::LevelCode,
        section: MainUiSection::MenuImages,
        offset: 0xa2d4,
        width: 120,
        height: 61,
        planes: 4,
        image_count: 1,
        stride: 3660,
        transparency: UiTransparency::ColorZero,
        palette: UiPalette::Menu,
    },
    UiImageInfo {
        kind: MainUiImageKind::Music,
        section: MainUiSection::MenuImages,
        offset: 0xb120,
        width: 120,
        height: 61,
        planes: 4,
        image_count: 1,
        stride: 3660,
        transparency: UiTransparency::ColorZero,
        palette: UiPalette::Menu,
    },
    UiImageInfo {
        kind: MainUiImageKind::Rating,
        section: MainUiSection::MenuImages,
        offset: 0xbf6c,
        width: 120,
        height: 61,
        planes: 4,
        image_count: 1,
        stride: 3660,
        transparency: UiTransparency::ColorZero,
        palette: UiPalette::Menu,
    },
    UiImageInfo {
        kind: MainUiImageKind::Exit,
        section: MainUiSection::MenuImages,
        offset: 0xcdb8,
        width: 120,
        height: 61,
        planes: 4,
        image_count: 1,
        stride: 3660,
        transparency: UiTransparency::ColorZero,
        palette: UiPalette::Menu,
    },
    UiImageInfo {
        kind: MainUiImageKind::Navigation,
        section: MainUiSection::MenuImages,
        offset: 0xdc04,
        width: 120,
        height: 61,
        planes: 4,
        image_count: 1,
        stride: 3660,
        transparency: UiTransparency::ColorZero,
        palette: UiPalette::Menu,
    },
    UiImageInfo {
        kind: MainUiImageKind::MusicIcon,
        section: MainUiSection::MenuImages,
        offset: 0xea50,
        width: 64,
        height: 31,
        planes: 4,
        image_count: 1,
        stride: 992,
        transparency: UiTransparency::ColorZero,
        palette: UiPalette::Menu,
    },
    UiImageInfo {
        kind: MainUiImageKind::FxIcon,
        section: MainUiSection::MenuImages,
        offset: 0xee30,
        width: 64,
        height: 31,
        planes: 4,
        image_count: 1,
        stride: 992,
        transparency: UiTransparency::ColorZero,
        palette: UiPalette::Menu,
    },
    UiImageInfo {
        kind: MainUiImageKind::ScrollLeft,
        section: MainUiSection::MenuExtras,
        offset: 0x2a00,
        width: 48,
        height: 16,
        planes: 4,
        image_count: 16,
        stride: 384,
        transparency: UiTransparency::ColorZero,
        palette: UiPalette::Menu,
    },
    UiImageInfo {
        kind: MainUiImageKind::ScrollRight,
        section: MainUiSection::MenuExtras,
        offset: 0x4200,
        width: 48,
        height: 16,
        planes: 4,
        image_count: 16,
        stride: 384,
        transparency: UiTransparency::ColorZero,
        palette: UiPalette::Menu,
    },
    UiImageInfo {
        kind: MainUiImageKind::Reel,
        section: MainUiSection::MenuExtras,
        offset: 0x5a00,
        width: 16,
        height: 16,
        planes: 4,
        image_count: 1,
        stride: 128,
        transparency: UiTransparency::ColorZero,
        palette: UiPalette::Menu,
    },
    UiImageInfo {
        kind: MainUiImageKind::RatingSigns,
        section: MainUiSection::MenuExtras,
        offset: 0x5a80,
        width: 72,
        height: 27,
        planes: 4,
        image_count: 4,
        stride: 972,
        transparency: UiTransparency::ColorZero,
        palette: UiPalette::Menu,
    },
    UiImageInfo {
        kind: MainUiImageKind::MenuGlyphs,
        section: MainUiSection::MenuExtras,
        offset: 0x69b0,
        width: 16,
        height: 16,
        planes: 3,
        image_count: 94,
        stride: 96,
        transparency: UiTransparency::ColorZero,
        palette: UiPalette::Menu,
    },
    UiImageInfo {
        kind: MainUiImageKind::SkillPanel,
        section: MainUiSection::SkillPanel,
        offset: 0x0,
        width: 320,
        height: 40,
        planes: 4,
        image_count: 1,
        stride: 6400,
        transparency: UiTransparency::Opaque,
        palette: UiPalette::Level,
    },
    UiImageInfo {
        kind: MainUiImageKind::PanelGlyphs,
        section: MainUiSection::SkillPanel,
        offset: 0x1900,
        width: 8,
        height: 16,
        planes: 3,
        image_count: 38,
        stride: 48,
        transparency: UiTransparency::Opaque,
        palette: UiPalette::Level,
    },
    UiImageInfo {
        kind: MainUiImageKind::DigitRight,
        section: MainUiSection::PanelDigits,
        offset: 0x1900,
        width: 8,
        height: 8,
        planes: 1,
        image_count: 10,
        stride: 16,
        transparency: UiTransparency::ColorZero,
        palette: UiPalette::Monochrome,
    },
    UiImageInfo {
        kind: MainUiImageKind::DigitLeft,
        section: MainUiSection::PanelDigits,
        offset: 0x1908,
        width: 8,
        height: 8,
        planes: 1,
        image_count: 10,
        stride: 16,
        transparency: UiTransparency::ColorZero,
        palette: UiPalette::Monochrome,
    },
];

impl MainDatFile<'_> {
    /// Validate the selected section's declared size and exact caller output
    /// length before writing. Decompression failures may partially change output:
    /// discard it on error. A successful view borrows only output, not this file.
    pub fn decode_ui_into(
        self,
        section: MainUiSection,
        output: &mut [u8],
    ) -> Result<MainUiRecord<'_>, MainUiError> {
        let expected = section.decoded_size();
        let part = self.sections()[section.part_index()];
        let actual = usize::from(part.header().decompressed_size);
        if actual != expected {
            return Err(MainUiError::DeclaredSize {
                section,
                expected,
                actual,
            });
        }
        if output.len() != expected {
            return Err(MainUiError::BufferSize {
                section,
                expected,
                actual: output.len(),
            });
        }
        part.decompress_into(output)
            .map_err(|error| MainUiError::Decompress { section, error })?;
        MainUiRecord::parse(section, output)
    }
}

/// An exact-sized original UI section, preserving all bytes including unparsed
/// ranges. Only the documented families are interpreted. This is not a screen,
/// font layout engine, palette converter or authentication of an asset file.
///
/// ```
/// use lemmings_resources::{MainUiRecord, MainUiSection, IndexedPixel};
/// let mut bytes = [0; 36080];
/// bytes[0x69b0] = 0x80; // first menu glyph '!', plane 0
/// let record = MainUiRecord::parse(MainUiSection::MenuExtras, &bytes)?;
/// let glyph = record.menu_glyph('!')?;
/// assert_eq!((glyph.width(), glyph.height()), (16, 16));
/// assert_eq!(glyph.pixel(0, 0), Some(IndexedPixel { color: 1, opaque: true }));
/// assert!(!glyph.pixel(1, 0).unwrap().opaque);
/// # Ok::<(), lemmings_resources::MainUiError>(())
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MainUiRecord<'a> {
    bytes: &'a [u8],
    section: MainUiSection,
}
impl<'a> MainUiRecord<'a> {
    pub fn parse(section: MainUiSection, bytes: &'a [u8]) -> Result<Self, MainUiError> {
        let expected = section.decoded_size();
        if bytes.len() != expected {
            return Err(MainUiError::Size {
                section,
                expected,
                actual: bytes.len(),
            });
        }
        Ok(Self { bytes, section })
    }
    pub const fn as_bytes(self) -> &'a [u8] {
        self.bytes
    }
    pub const fn section(self) -> MainUiSection {
        self.section
    }
    pub fn images(self, kind: MainUiImageKind) -> Result<UiImages<'a>, MainUiError> {
        let info = kind.info();
        if info.section != self.section {
            return Err(MainUiError::SectionMismatch {
                expected: info.section,
                actual: self.section,
            });
        }
        Ok(UiImages {
            bytes: self.bytes,
            info,
        })
    }
    /// Stored ASCII '!'..'~'. Space/control/non-ASCII characters are errors:
    /// their layout or replacement is the caller's responsibility.
    pub fn menu_glyph(self, character: char) -> Result<UiImage<'a>, MainUiError> {
        let images = self.images(MainUiImageKind::MenuGlyphs)?;
        if !('!'..='~').contains(&character) {
            return Err(MainUiError::Glyph {
                section: self.section,
                character,
            });
        }
        images.image(character as usize - '!' as usize)
    }
    /// Disk order: percent, digits, dash, A..Z. ASCII lowercase aliases uppercase;
    /// spaces, unsupported punctuation and non-ASCII have no stored glyph.
    pub fn panel_glyph(self, character: char) -> Result<UiImage<'a>, MainUiError> {
        let images = self.images(MainUiImageKind::PanelGlyphs)?;
        let upper = character.to_ascii_uppercase();
        let index = "%0123456789-ABCDEFGHIJKLMNOPQRSTUVWXYZ"
            .chars()
            .position(|c| c == upper)
            .ok_or(MainUiError::Glyph {
                section: self.section,
                character,
            })?;
        images.image(index)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiImages<'a> {
    bytes: &'a [u8],
    info: UiImageInfo,
}
impl<'a> UiImages<'a> {
    pub const fn info(self) -> UiImageInfo {
        self.info
    }
    /// Disk order without wrapping or playback. Every metadata window lies
    /// inside its exact record. Index is bounded before multiplying by stride.
    pub fn image(self, index: usize) -> Result<UiImage<'a>, MainUiError> {
        if index >= usize::from(self.info.image_count) {
            return Err(MainUiError::ImageIndex {
                kind: self.info.kind,
                index,
                count: self.info.image_count,
            });
        }
        let start = self.info.offset + index * self.info.stride;
        Ok(UiImage {
            bytes: &self.bytes[start..start + self.info.image_size()],
            info: self.info,
        })
    }
}

/// Planar UI image: complete row-major planes, MSB left and plane 0 is bit 0.
/// Color indices are preserved even when zero is opaque (background/panel/font).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiImage<'a> {
    bytes: &'a [u8],
    info: UiImageInfo,
}
impl UiImage<'_> {
    pub const fn width(self) -> u16 {
        self.info.width
    }
    pub const fn height(self) -> u16 {
        self.info.height
    }
    pub fn pixel(self, x: usize, y: usize) -> Option<IndexedPixel> {
        if x >= usize::from(self.width()) || y >= usize::from(self.height()) {
            return None;
        }
        Some(self.pixel_at(y * usize::from(self.width()) + x))
    }
    /// Exact row-major caller output. A size error preserves every output pixel.
    pub fn decode_into(self, output: &mut [IndexedPixel]) -> Result<(), MainUiError> {
        let expected = usize::from(self.width()) * usize::from(self.height());
        if output.len() != expected {
            return Err(MainUiError::OutputSize {
                expected,
                actual: output.len(),
            });
        }
        for (index, pixel) in output.iter_mut().enumerate() {
            *pixel = self.pixel_at(index);
        }
        Ok(())
    }
    fn pixel_at(self, index: usize) -> IndexedPixel {
        let size = usize::from(self.width()) * usize::from(self.height()) / 8;
        let mut color = 0;
        for plane in 0..usize::from(self.info.planes) {
            if self.bytes[plane * size + index / 8] & (128 >> (index % 8)) != 0 {
                color |= 1 << plane;
            }
        }
        IndexedPixel {
            color,
            opaque: self.info.transparency == UiTransparency::Opaque || color != 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MainUiError {
    Size {
        section: MainUiSection,
        expected: usize,
        actual: usize,
    },
    DeclaredSize {
        section: MainUiSection,
        expected: usize,
        actual: usize,
    },
    BufferSize {
        section: MainUiSection,
        expected: usize,
        actual: usize,
    },
    SectionMismatch {
        expected: MainUiSection,
        actual: MainUiSection,
    },
    ImageIndex {
        kind: MainUiImageKind,
        index: usize,
        count: u8,
    },
    Glyph {
        section: MainUiSection,
        character: char,
    },
    OutputSize {
        expected: usize,
        actual: usize,
    },
    Decompress {
        section: MainUiSection,
        error: DecompressError,
    },
}
impl fmt::Display for MainUiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Size {
                section,
                expected,
                actual,
            } => write!(
                f,
                "MAIN UI {section:?} record length {actual}; expected {expected}"
            ),
            Self::DeclaredSize {
                section,
                expected,
                actual,
            } => write!(
                f,
                "MAIN UI {section:?} declared size {actual}; expected {expected}"
            ),
            Self::BufferSize {
                section,
                expected,
                actual,
            } => write!(
                f,
                "MAIN UI {section:?} decode output length {actual}; expected {expected}"
            ),
            Self::SectionMismatch { expected, actual } => {
                write!(f, "MAIN UI image needs {expected:?}; supplied {actual:?}")
            }
            Self::ImageIndex { kind, index, count } => {
                write!(f, "MAIN UI {kind:?} image {index}; expected 0..{count}")
            }
            Self::Glyph { section, character } => {
                write!(f, "MAIN UI {section:?} has no glyph for {character:?}")
            }
            Self::OutputSize { expected, actual } => write!(
                f,
                "MAIN UI pixel output length {actual}; expected {expected}"
            ),
            Self::Decompress { section, error } => write!(
                f,
                "MAIN UI {section:?} section {}: {error}",
                section.part_index()
            ),
        }
    }
}
impl core::error::Error for MainUiError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Decompress { error, .. } => Some(error),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use MainUiImageKind as K;
    use MainUiSection::*;
    use std::{format, string::String, vec, vec::Vec};
    fn record(section: MainUiSection) -> Vec<u8> {
        vec![0; section.decoded_size()]
    }
    fn p(color: u8, opaque: bool) -> IndexedPixel {
        IndexedPixel { color, opaque }
    }
    fn container(section: MainUiSection, payload: &[u8]) -> Vec<u8> {
        let mut result = packed(&vec![0; 21104]);
        for index in 1..7 {
            result.extend(packed(if index == section.part_index() {
                payload
            } else {
                &[0]
            }));
        }
        result
    }
    fn packed(data: &[u8]) -> Vec<u8> {
        let mut bits = String::new();
        for chunk in data.rchunks(264) {
            bits += &if chunk.len() <= 8 {
                format!("00{:03b}", chunk.len() - 1)
            } else {
                format!("111{:08b}", chunk.len() - 9)
            };
            for byte in chunk.iter().rev() {
                bits += &format!("{byte:08b}");
            }
        }
        let payload: Vec<u8> = bits
            .as_bytes()
            .rchunks(8)
            .map(|c| {
                c.iter()
                    .enumerate()
                    .fold(0, |a, (i, b)| a | ((b - b'0') << i))
            })
            .collect();
        let mut b = vec![
            ((bits.len() - 1) % 8 + 1) as u8,
            payload.iter().fold(0, |a, b| a ^ b),
            0,
            0,
        ];
        b.extend((data.len() as u16).to_be_bytes());
        b.extend([0, 0]);
        b.extend(((10 + payload.len()) as u16).to_be_bytes());
        b.extend(payload);
        b
    }

    #[test]
    fn every_short_prefix_and_extra_byte_is_rejected_for_all_four_sections() {
        for (section, size) in [
            (PanelDigits, 8384),
            (MenuImages, 61968),
            (MenuExtras, 36080),
            (SkillPanel, 8224),
        ] {
            assert_eq!(section.decoded_size(), size);
            let b = vec![0; size + 1];
            for len in 0..size {
                assert_eq!(
                    MainUiRecord::parse(section, &b[..len]),
                    Err(MainUiError::Size {
                        section,
                        expected: size,
                        actual: len
                    })
                );
            }
            assert!(MainUiRecord::parse(section, &b).is_err());
            let view = MainUiRecord::parse(section, &b[..size]).unwrap();
            assert_eq!(view.section(), section);
            assert_eq!(view.as_bytes().as_ptr(), b.as_ptr());
        }
    }
    #[test]
    fn nineteen_families_cover_two_hundred_images_and_fixed_plane_windows() {
        let mut images = 0;
        let mut pixels = 0;
        let mut kinds = 0;
        for section in MainUiSection::ALL {
            let b = record(section);
            let view = MainUiRecord::parse(section, &b).unwrap();
            for kind in MAIN_UI_IMAGE_KINDS {
                if let Ok(group) = view.images(kind) {
                    let info = group.info();
                    kinds += 1;
                    images += usize::from(info.image_count);
                    pixels += usize::from(info.width)
                        * usize::from(info.height)
                        * usize::from(info.image_count);
                    assert_eq!(info.kind, kind);
                    assert_eq!(info.section, section);
                    assert_eq!(
                        info.image_size(),
                        usize::from(info.width) * usize::from(info.height) / 8
                            * usize::from(info.planes)
                    );
                    assert!(
                        info.offset
                            + (usize::from(info.image_count) - 1) * info.stride
                            + info.image_size()
                            <= b.len()
                    );
                }
            }
        }
        assert_eq!((kinds, images, pixels), (19, 200, 216192));
    }
    #[test]
    fn image_families_are_bound_to_their_section() {
        let b = record(MenuImages);
        let view = MainUiRecord::parse(MenuImages, &b).unwrap();
        assert_eq!(
            view.images(K::SkillPanel),
            Err(MainUiError::SectionMismatch {
                expected: SkillPanel,
                actual: MenuImages
            })
        );
        assert_eq!(
            view.images(K::MenuGlyphs),
            Err(MainUiError::SectionMismatch {
                expected: MenuExtras,
                actual: MenuImages
            })
        );
        let b = record(SkillPanel);
        let view = MainUiRecord::parse(SkillPanel, &b).unwrap();
        assert_eq!(
            view.menu_glyph('!'),
            Err(MainUiError::SectionMismatch {
                expected: MenuExtras,
                actual: SkillPanel
            })
        );
    }
    #[test]
    fn background_two_planes_are_msb_first_with_opaque_zero() {
        let mut b = record(MenuImages);
        b[0] = 128;
        b[4160] = 64;
        b[4159] = 1;
        b[8319] = 1;
        let view = MainUiRecord::parse(MenuImages, &b).unwrap();
        let group = view.images(K::Background).unwrap();
        assert_eq!(group.info().transparency, UiTransparency::Opaque);
        let image = group.image(0).unwrap();
        assert_eq!(image.pixel(0, 0), Some(p(1, true)));
        assert_eq!(image.pixel(1, 0), Some(p(2, true)));
        assert_eq!(image.pixel(2, 0), Some(p(0, true)));
        assert_eq!(image.pixel(319, 103), Some(p(3, true)));
    }
    #[test]
    fn three_plane_fonts_have_distinct_opacity_policies_and_no_index_remap() {
        let mut b = record(MenuExtras);
        b[0x69b0 + 64] = 128;
        let view = MainUiRecord::parse(MenuExtras, &b).unwrap();
        let image = view.menu_glyph('!').unwrap();
        assert_eq!(image.pixel(0, 0), Some(p(4, true)));
        assert_eq!(image.pixel(1, 0), Some(p(0, false)));
        let mut b = record(SkillPanel);
        b[6400 + 32] = 128;
        let view = MainUiRecord::parse(SkillPanel, &b).unwrap();
        let image = view.panel_glyph('%').unwrap();
        assert_eq!(image.pixel(0, 0), Some(p(4, true)));
        assert_eq!(image.pixel(1, 0), Some(p(0, true)));
    }
    #[test]
    fn logo_width_632_reaches_the_last_row_and_preserves_four_bit_indices() {
        let mut b = record(MenuImages);
        let size = 632 * 94 / 8;
        for plane in 0..4 {
            b[0x2080 + plane * size + size - 1] = 1;
        }
        b[0x2080 + 3 * size] = 128;
        let view = MainUiRecord::parse(MenuImages, &b).unwrap();
        let group = view.images(K::Logo).unwrap();
        assert_eq!(group.info().palette, UiPalette::Menu);
        let image = group.image(0).unwrap();
        assert_eq!((image.width(), image.height()), (632, 94));
        assert_eq!(image.pixel(0, 0), Some(p(8, true)));
        assert_eq!(image.pixel(631, 93), Some(p(15, true)));
        assert_eq!(image.pixel(1, 0), Some(p(0, false)));
    }
    #[test]
    fn digit_halves_are_interleaved_right_then_left_with_a_sixteen_byte_stride() {
        let mut b = record(PanelDigits);
        b[6400] = 128;
        b[6408] = 64;
        b[6400 + 9 * 16 + 7] = 1;
        b[6408 + 9 * 16 + 7] = 2;
        let view = MainUiRecord::parse(PanelDigits, &b).unwrap();
        let right = view.images(K::DigitRight).unwrap();
        let left = view.images(K::DigitLeft).unwrap();
        assert_eq!(right.info().stride, 16);
        assert_eq!(right.info().image_size(), 8);
        assert_eq!(right.info().palette, UiPalette::Monochrome);
        assert_eq!(right.image(0).unwrap().pixel(0, 0), Some(p(1, true)));
        assert_eq!(left.image(0).unwrap().pixel(1, 0), Some(p(1, true)));
        assert_eq!(right.image(9).unwrap().pixel(7, 7), Some(p(1, true)));
        assert_eq!(left.image(9).unwrap().pixel(6, 7), Some(p(1, true)));
    }
    #[test]
    fn both_scroll_sequences_keep_all_sixteen_disk_frames() {
        let mut b = record(MenuExtras);
        for frame in 0..16 {
            b[0x2a00 + frame * 384] = 1 << (frame % 8);
            b[0x4200 + frame * 384 + 96] = 1 << (frame % 8);
        }
        let view = MainUiRecord::parse(MenuExtras, &b).unwrap();
        for frame in 0..16 {
            assert_eq!(
                view.images(K::ScrollLeft)
                    .unwrap()
                    .image(frame)
                    .unwrap()
                    .pixel(7 - frame % 8, 0),
                Some(p(1, true))
            );
            assert_eq!(
                view.images(K::ScrollRight)
                    .unwrap()
                    .image(frame)
                    .unwrap()
                    .pixel(7 - frame % 8, 0),
                Some(p(2, true))
            );
        }
    }
    #[test]
    fn four_rating_signs_end_exactly_at_the_menu_font_start() {
        let mut b = record(MenuExtras);
        b[0x5a80] = 128;
        b[0x69af] = 1;
        let view = MainUiRecord::parse(MenuExtras, &b).unwrap();
        let signs = view.images(K::RatingSigns).unwrap();
        assert_eq!(signs.info().image_count, 4);
        assert_eq!(signs.info().stride, 972);
        assert_eq!(signs.image(0).unwrap().pixel(0, 0), Some(p(1, true)));
        assert_eq!(signs.image(3).unwrap().pixel(71, 26), Some(p(8, true)));
        assert_eq!(view.menu_glyph('!').unwrap().pixel(0, 0), Some(p(0, false)));
    }
    #[test]
    fn menu_glyphs_map_printable_ascii_without_synthetic_space_or_unicode_aliases() {
        let mut b = record(MenuExtras);
        for index in 0..94 {
            b[0x69b0 + index * 96] = 1 << (index % 8);
        }
        let view = MainUiRecord::parse(MenuExtras, &b).unwrap();
        for (index, character) in ('!'..='~').enumerate() {
            assert_eq!(
                view.menu_glyph(character).unwrap().pixel(7 - index % 8, 0),
                Some(p(1, true))
            );
        }
        for character in [' ', '\0', '\n', '\u{7f}', '한', 'ß', '\u{10ffff}'] {
            assert_eq!(
                view.menu_glyph(character),
                Err(MainUiError::Glyph {
                    section: MenuExtras,
                    character
                })
            );
        }
    }
    #[test]
    fn panel_glyph_order_percent_digits_dash_letters_supports_ascii_lowercase_only() {
        let mut b = record(SkillPanel);
        let chars = "%0123456789-ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        for index in 0..38 {
            b[6400 + index * 48] = 1 << (index % 8);
        }
        let view = MainUiRecord::parse(SkillPanel, &b).unwrap();
        for (index, character) in chars.chars().enumerate() {
            assert_eq!(
                view.panel_glyph(character).unwrap().pixel(7 - index % 8, 0),
                Some(p(1, true))
            );
            assert_eq!(
                view.panel_glyph(character.to_ascii_lowercase()),
                view.panel_glyph(character)
            );
        }
        for character in [' ', '\0', '?', '!', '한', 'ß', '\u{10ffff}'] {
            assert_eq!(
                view.panel_glyph(character),
                Err(MainUiError::Glyph {
                    section: SkillPanel,
                    character
                })
            );
        }
    }
    #[test]
    fn huge_image_indices_and_coordinates_are_checked_before_multiplication() {
        for section in MainUiSection::ALL {
            let b = record(section);
            let view = MainUiRecord::parse(section, &b).unwrap();
            for kind in MAIN_UI_IMAGE_KINDS {
                if let Ok(group) = view.images(kind) {
                    let count = group.info().image_count;
                    for index in [usize::from(count), usize::MAX] {
                        assert_eq!(
                            group.image(index),
                            Err(MainUiError::ImageIndex { kind, index, count })
                        );
                    }
                    let image = group.image(0).unwrap();
                    for (x, y) in [
                        (usize::from(image.width()), 0),
                        (0, usize::from(image.height())),
                        (usize::MAX, 0),
                        (0, usize::MAX),
                    ] {
                        assert_eq!(image.pixel(x, y), None);
                    }
                }
            }
        }
    }
    #[test]
    fn bulk_decode_keeps_canaries_and_size_errors_preserve_every_output() {
        let sentinel = p(99, true);
        for section in MainUiSection::ALL {
            let mut b = record(section);
            for (i, v) in b.iter_mut().enumerate() {
                *v = (i * 73) as u8;
            }
            let view = MainUiRecord::parse(section, &b).unwrap();
            for kind in MAIN_UI_IMAGE_KINDS {
                if let Ok(group) = view.images(kind) {
                    for index in 0..usize::from(group.info().image_count) {
                        let image = group.image(index).unwrap();
                        let n = usize::from(image.width()) * usize::from(image.height());
                        let mut output = vec![sentinel; n + 2];
                        image.decode_into(&mut output[1..n + 1]).unwrap();
                        assert_eq!(output[0], sentinel);
                        assert_eq!(output[n + 1], sentinel);
                        for (i, pixel) in output[1..n + 1].iter().enumerate() {
                            assert_eq!(
                                Some(*pixel),
                                image.pixel(
                                    i % usize::from(image.width()),
                                    i / usize::from(image.width())
                                )
                            );
                        }
                    }
                    let image = group.image(0).unwrap();
                    let n = usize::from(image.width()) * usize::from(image.height());
                    for actual in [0, n - 1, n + 1] {
                        let mut output = vec![sentinel; actual];
                        assert_eq!(
                            image.decode_into(&mut output),
                            Err(MainUiError::OutputSize {
                                expected: n,
                                actual
                            })
                        );
                        assert!(output.iter().all(|&p| p == sentinel));
                    }
                }
            }
        }
    }
    #[test]
    fn decode_borrows_only_caller_output_and_preserves_uninterpreted_section_bytes() {
        for section in MainUiSection::ALL {
            let mut output = record(section);
            let ptr = output.as_ptr();
            let view = {
                let mut data = record(section);
                data.fill(0xa5);
                let b = container(section, &data);
                let main = MainDatFile::parse(&b).unwrap();
                main.decode_ui_into(section, &mut output).unwrap()
            };
            assert_eq!(view.as_bytes().as_ptr(), ptr);
            assert!(view.as_bytes().iter().all(|&v| v == 0xa5));
        }
    }
    #[test]
    fn declared_sizes_and_exact_decode_lengths_are_checked_before_any_writes() {
        for section in MainUiSection::ALL {
            let expected = section.decoded_size();
            let b = container(section, &[0]);
            let main = MainDatFile::parse(&b).unwrap();
            let mut output = vec![99; expected];
            assert_eq!(
                main.decode_ui_into(section, &mut output),
                Err(MainUiError::DeclaredSize {
                    section,
                    expected,
                    actual: 1
                })
            );
            assert!(output.iter().all(|&v| v == 99));
            let b = container(section, &record(section));
            let main = MainDatFile::parse(&b).unwrap();
            for actual in [0, expected - 1, expected + 1] {
                let mut output = vec![99; actual];
                assert_eq!(
                    main.decode_ui_into(section, &mut output),
                    Err(MainUiError::BufferSize {
                        section,
                        expected,
                        actual
                    })
                );
                assert!(output.iter().all(|&v| v == 99));
            }
        }
    }
    #[test]
    fn malformed_selected_commands_report_section_and_error_source() {
        let section = SkillPanel;
        let mut parts = vec![packed(&vec![0; 21104])];
        for index in 1..7 {
            if index == 6 {
                parts.push(vec![8, 0, 0, 0, 0x20, 0x20, 0, 0, 0, 11, 0]);
            } else {
                parts.push(packed(&[0]));
            }
        }
        let bytes: Vec<u8> = parts.concat();
        let file = MainDatFile::parse(&bytes).unwrap();
        let mut output = record(section);
        let error = file.decode_ui_into(section, &mut output).unwrap_err();
        assert!(matches!(
            error,
            MainUiError::Decompress {
                section: SkillPanel,
                ..
            }
        ));
        assert!(core::error::Error::source(&error).is_some());
        let mut unaffected = [99];
        file.sections()[1].decompress_into(&mut unaffected).unwrap();
        assert_eq!(unaffected, [0]);
    }
}
