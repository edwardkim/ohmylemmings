use core::fmt;

/// Immutable row-major binary occupancy, independent of colors and resources.
/// Dimensions are not fixed to a viewport. A view is valid only for an exact
/// width*height byte slice, with every byte either 0 (air) or 1 (terrain).
/// This type does not decide actor survival, walls, steel, or trigger behavior.
///
/// ```
/// use lemmings_core::TerrainMask;
/// let cells = [0, 1, 0, 1, 0, 1];
/// let terrain = TerrainMask::new(3, 2, &cells)?;
/// assert_eq!(terrain.pixel(1, 0), Some(true));
/// assert_eq!(terrain.pixel(0, 0), Some(false));
/// assert_eq!(terrain.pixel(3, 0), None);
/// assert!(terrain.is_solid(0, 1));
/// assert!(!terrain.is_solid(-1, 1));
/// assert_eq!(terrain.row(1), Some(&[1, 0, 1][..]));
/// # Ok::<(), lemmings_core::TerrainMaskError>(())
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TerrainMask<'a> {
    width: usize,
    height: usize,
    cells: &'a [u8],
}
impl<'a> TerrainMask<'a> {
    /// Validate nonzero dimensions, checked area, exact length, then every cell.
    /// Borrows the original bytes without allocation, padding, or normalization.
    pub fn new(width: usize, height: usize, cells: &'a [u8]) -> Result<Self, TerrainMaskError> {
        if width == 0 || height == 0 {
            return Err(TerrainMaskError::EmptyDimensions { width, height });
        }
        let expected = width
            .checked_mul(height)
            .ok_or(TerrainMaskError::AreaOverflow { width, height })?;
        if cells.len() != expected {
            return Err(TerrainMaskError::BufferSize {
                expected,
                actual: cells.len(),
            });
        }
        if let Some((index, &value)) = cells.iter().enumerate().find(|(_, value)| **value > 1) {
            return Err(TerrainMaskError::CellValue { index, value });
        }
        Ok(Self {
            width,
            height,
            cells,
        })
    }
    pub const fn width(self) -> usize {
        self.width
    }
    pub const fn height(self) -> usize {
        self.height
    }
    pub const fn pixel_count(self) -> usize {
        self.cells.len()
    }
    pub const fn as_bytes(self) -> &'a [u8] {
        self.cells
    }

    /// Distinguishes an in-bounds empty pixel (Some(false)) from outside (None).
    /// Checks both coordinates before multiplication or addition.
    pub fn pixel(self, x: usize, y: usize) -> Option<bool> {
        if x >= self.width || y >= self.height {
            return None;
        }
        Some(self.cells[y * self.width + x] == 1)
    }
    /// Returns one exact row without padding, or None outside the height.
    pub fn row(self, y: usize) -> Option<&'a [u8]> {
        if y >= self.height {
            return None;
        }
        let start = y * self.width;
        Some(&self.cells[start..start + self.width])
    }
    /// Signed geometric lookup: every outside coordinate has no terrain.
    /// Game-specific border, death, and actor movement rules belong to callers.
    pub fn is_solid(self, x: i32, y: i32) -> bool {
        match (usize::try_from(x), usize::try_from(y)) {
            (Ok(x), Ok(y)) => self.pixel(x, y).unwrap_or(false),
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TerrainMaskError {
    EmptyDimensions { width: usize, height: usize },
    AreaOverflow { width: usize, height: usize },
    BufferSize { expected: usize, actual: usize },
    CellValue { index: usize, value: u8 },
}
impl fmt::Display for TerrainMaskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyDimensions { width, height } => write!(
                f,
                "terrain dimensions {width}x{height}; both must be nonzero"
            ),
            Self::AreaOverflow { width, height } => {
                write!(f, "terrain area {width}x{height} overflows usize")
            }
            Self::BufferSize { expected, actual } => write!(
                f,
                "terrain buffer length {actual}; expected {expected} cells"
            ),
            Self::CellValue { index, value } => {
                write!(f, "terrain cell {index} has value {value}; expected 0 or 1")
            }
        }
    }
}
impl core::error::Error for TerrainMaskError {}
#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use std::{vec, vec::Vec};

    #[test]
    fn rejects_zero_sized_dimensions_before_other_validation() {
        for (width, height) in [(0, 0), (0, usize::MAX), (usize::MAX, 0)] {
            assert_eq!(
                TerrainMask::new(width, height, &[255]),
                Err(TerrainMaskError::EmptyDimensions { width, height })
            );
        }
    }
    #[test]
    fn detects_area_overflow_without_allocating_or_inspecting_a_buffer() {
        for (width, height) in [(usize::MAX, 2), (2, usize::MAX), (usize::MAX, usize::MAX)] {
            assert_eq!(
                TerrainMask::new(width, height, &[255]),
                Err(TerrainMaskError::AreaOverflow { width, height })
            );
        }
        assert_eq!(
            TerrainMask::new(usize::MAX, 1, &[]),
            Err(TerrainMaskError::BufferSize {
                expected: usize::MAX,
                actual: 0
            })
        );
    }
    #[test]
    fn validates_exact_buffer_length_before_cell_values_and_never_changes_input() {
        for actual in 0..8 {
            let cells = vec![255; actual];
            let result = TerrainMask::new(2, 2, &cells);
            let expected = if actual == 4 {
                TerrainMaskError::CellValue {
                    index: 0,
                    value: 255,
                }
            } else {
                TerrainMaskError::BufferSize {
                    expected: 4,
                    actual,
                }
            };
            assert_eq!(result, Err(expected));
            assert!(cells.iter().all(|v| *v == 255));
        }
    }
    #[test]
    fn rejects_every_nonbinary_byte_and_reports_the_first_invalid_cell() {
        for value in 2..=255 {
            let cells = [0, 1, value, 255];
            assert_eq!(
                TerrainMask::new(2, 2, &cells),
                Err(TerrainMaskError::CellValue { index: 2, value })
            );
        }
        assert_eq!(
            TerrainMask::new(2, 2, &[0, 1, 0, 2]),
            Err(TerrainMaskError::CellValue { index: 3, value: 2 })
        );
    }
    #[test]
    fn preserves_borrowed_row_major_bytes_and_metadata_without_allocation() {
        let cells = [0, 1, 0, 1, 1, 0];
        let mask = TerrainMask::new(3, 2, &cells).unwrap();
        assert_eq!((mask.width(), mask.height(), mask.pixel_count()), (3, 2, 6));
        assert_eq!(mask.as_bytes().as_ptr(), cells.as_ptr());
        assert_eq!(mask.as_bytes(), cells);
        let copy = mask;
        assert_eq!(copy.as_bytes().as_ptr(), cells.as_ptr());
    }
    #[test]
    fn reads_all_cells_in_an_asymmetric_map() {
        let cells = [0, 1, 0, 1, 1, 0, 0, 0, 0, 0, 1, 1];
        let mask = TerrainMask::new(4, 3, &cells).unwrap();
        let occupied = [(1, 0), (3, 0), (0, 1), (2, 2), (3, 2)];
        for y in 0..3 {
            for x in 0..4 {
                assert_eq!(mask.pixel(x, y), Some(occupied.contains(&(x, y))));
            }
        }
    }
    #[test]
    fn distinguishes_empty_inside_pixels_from_outside_coordinates() {
        let mask = TerrainMask::new(3, 2, &[0, 0, 0, 0, 0, 0]).unwrap();
        assert_eq!(mask.pixel(0, 0), Some(false));
        assert_eq!(mask.pixel(2, 1), Some(false));
        assert_eq!(mask.pixel(3, 0), None);
        assert_eq!(mask.pixel(0, 2), None);
        assert_eq!(mask.pixel(3, 2), None);
    }
    #[test]
    fn rejects_unsigned_extremes_before_index_arithmetic() {
        let mask = TerrainMask::new(3, 2, &[1; 6]).unwrap();
        for (x, y) in [
            (usize::MAX, 0),
            (0, usize::MAX),
            (usize::MAX, usize::MAX),
            (3, usize::MAX),
            (usize::MAX, 2),
        ] {
            assert_eq!(mask.pixel(x, y), None);
        }
        assert_eq!(mask.pixel(2, 1), Some(true));
    }
    #[test]
    fn signed_queries_treat_negative_and_extreme_outside_coordinates_as_empty() {
        let mask = TerrainMask::new(3, 2, &[0, 1, 0, 1, 0, 1]).unwrap();
        for (x, y) in [
            (-1, 0),
            (0, -1),
            (-1, -1),
            (3, 0),
            (0, 2),
            (i32::MIN, 0),
            (0, i32::MIN),
            (i32::MAX, 0),
            (0, i32::MAX),
            (i32::MAX, i32::MAX),
        ] {
            assert!(!mask.is_solid(x, y));
        }
        assert!(mask.is_solid(1, 0));
        assert!(mask.is_solid(2, 1));
        assert!(!mask.is_solid(0, 0));
    }
    #[test]
    fn rows_are_exact_disjoint_slices_and_do_not_read_neighboring_canaries() {
        let cells = [255, 0, 1, 1, 0, 254];
        let mask = TerrainMask::new(2, 2, &cells[1..5]).unwrap();
        assert_eq!(mask.row(0), Some(&[0, 1][..]));
        assert_eq!(mask.row(1), Some(&[1, 0][..]));
        assert_eq!(mask.row(0).unwrap().as_ptr(), cells[1..].as_ptr());
        assert_eq!(mask.row(1).unwrap().as_ptr(), cells[3..].as_ptr());
        assert_eq!(mask.row(2), None);
        assert_eq!(mask.row(usize::MAX), None);
        assert_eq!(cells[0], 255);
        assert_eq!(cells[5], 254);
    }
    #[test]
    fn supports_single_cell_single_row_single_column_and_non_viewport_dimensions() {
        for (w, h) in [(1, 1), (1, 5), (5, 1), (1648, 164)] {
            let mut cells = vec![0; w * h];
            cells[0] = 1;
            cells[w * h - 1] = 1;
            let mask = TerrainMask::new(w, h, &cells).unwrap();
            assert_eq!(mask.pixel(w - 1, h - 1), Some(true));
            assert!(mask.is_solid(0, 0));
            assert_eq!(mask.row(h - 1).unwrap().len(), w);
            assert_eq!(mask.pixel(w, h - 1), None);
            assert_eq!(mask.pixel(w - 1, h), None);
        }
    }
    #[test]
    fn exhaustive_small_masks_match_coordinate_sets_and_pinned_ts_digest() {
        let (mut masks, mut queries, mut hash) = (0, 0, 0xcbf29ce484222325u64);
        let mut mix = |value: u8| {
            hash = (hash ^ u64::from(value)).wrapping_mul(0x100000001b3);
        };
        for width in 1..=3 {
            for height in 1..=3 {
                for pattern in 0u16..(1 << (width * height)) {
                    let coords: Vec<_> = (0..height)
                        .flat_map(|y| (0..width).map(move |x| (x, y)))
                        .collect();
                    let points: Vec<_> = coords
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| pattern & (1 << i) != 0)
                        .map(|(_, p)| *p)
                        .collect();
                    let cells: Vec<_> = (0..coords.len())
                        .map(|i| ((pattern >> i) & 1) as u8)
                        .collect();
                    let mask = TerrainMask::new(width as usize, height as usize, &cells).unwrap();
                    for y in -1..=height {
                        for x in -1..=width {
                            let wanted = points.contains(&(x, y));
                            let inside = x >= 0 && x < width && y >= 0 && y < height;
                            let checked = if x < 0 || y < 0 {
                                None
                            } else {
                                mask.pixel(x as usize, y as usize)
                            };
                            assert_eq!(checked, inside.then_some(wanted));
                            assert_eq!(mask.is_solid(x, y), wanted);
                            mix(u8::from(mask.is_solid(x, y)));
                            mix(checked.map(u8::from).unwrap_or(2));
                            queries += 1;
                        }
                    }
                    for y in 0..height {
                        for &v in mask.row(y as usize).unwrap() {
                            mix(v);
                        }
                    }
                    assert_eq!(mask.as_bytes(), cells);
                    masks += 1;
                }
            }
        }
        assert_eq!((masks, queries), (682, 15970));
        // Actual pinned TS SolidLayer, same ordered corpus and coordinate-set oracle.
        assert_eq!(hash, 0x2eeedec0fe771b08);
        std::println!("682 masks / 15970 queries / fnv1a64={hash:016x}");
    }
}
