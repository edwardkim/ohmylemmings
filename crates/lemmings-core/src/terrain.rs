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

/// Exclusive binary terrain storage with checked, bool-only cell edits.
/// Construction shares TerrainMask validation and never changes the input.
/// Raw mutable bytes/rows are not exposed, preserving the binary invariant.
/// This view is neither Clone nor Copy; read-only views follow Rust borrowing.
///
/// ```
/// use lemmings_core::TerrainMaskMut;
/// let mut cells = [0, 1, 0, 0, 0, 1];
/// let mut terrain = TerrainMaskMut::new(3, 2, &mut cells)?;
/// assert_eq!(terrain.set_pixel(0, 0, true), Some(true)); // changed
/// assert_eq!(terrain.set_pixel(0, 0, true), Some(false)); // unchanged
/// assert_eq!(terrain.set_solid(-1, 0, false), None); // outside
/// assert!(terrain.as_read_only().is_solid(0, 0));
/// let terrain = terrain.into_read_only();
/// assert_eq!(terrain.row(0), Some(&[1, 1, 0][..]));
/// # Ok::<(), lemmings_core::TerrainMaskError>(())
/// ```
#[derive(Debug, Eq, PartialEq)]
pub struct TerrainMaskMut<'a> {
    width: usize,
    height: usize,
    cells: &'a mut [u8],
}
impl<'a> TerrainMaskMut<'a> {
    pub fn new(width: usize, height: usize, cells: &'a mut [u8]) -> Result<Self, TerrainMaskError> {
        TerrainMask::new(width, height, cells)?;
        Ok(Self {
            width,
            height,
            cells,
        })
    }
    /// Reborrow without copying or rescanning. Writes cannot coexist with a
    /// read-only view that is still in use.
    ///
    /// ```compile_fail,E0502
    /// use lemmings_core::TerrainMaskMut;
    /// let mut cells = [0];
    /// let mut terrain = TerrainMaskMut::new(1, 1, &mut cells).unwrap();
    /// let reading = terrain.as_read_only();
    /// let _ = terrain.set_pixel(0, 0, true);
    /// assert_eq!(reading.pixel(0, 0), Some(false));
    /// ```
    pub fn as_read_only(&self) -> TerrainMask<'_> {
        TerrainMask {
            width: self.width,
            height: self.height,
            cells: self.cells,
        }
    }
    /// Consume the exclusive view and retain the original buffer lifetime.
    pub fn into_read_only(self) -> TerrainMask<'a> {
        TerrainMask {
            width: self.width,
            height: self.height,
            cells: self.cells,
        }
    }
    /// Some(true) means changed, Some(false) unchanged, None outside. An outside
    /// write changes no cells. Bounds are checked before index arithmetic.
    pub fn set_pixel(&mut self, x: usize, y: usize, solid: bool) -> Option<bool> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let cell = &mut self.cells[y * self.width + x];
        let value = u8::from(solid);
        let changed = *cell != value;
        *cell = value;
        Some(changed)
    }
    /// Signed counterpart; negative or unrepresentable coordinates return None.
    pub fn set_solid(&mut self, x: i32, y: i32, solid: bool) -> Option<bool> {
        match (usize::try_from(x), usize::try_from(y)) {
            (Ok(x), Ok(y)) => self.set_pixel(x, y, solid),
            _ => None,
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
#[cfg(test)]
mod edit_tests {
    extern crate std;
    use super::*;
    use std::{collections::BTreeSet, vec, vec::Vec};

    #[test]
    fn mutable_construction_reuses_validation_and_preserves_rejected_buffers() {
        let cases = [
            (
                0,
                1,
                vec![255],
                TerrainMaskError::EmptyDimensions {
                    width: 0,
                    height: 1,
                },
            ),
            (
                usize::MAX,
                2,
                vec![255],
                TerrainMaskError::AreaOverflow {
                    width: usize::MAX,
                    height: 2,
                },
            ),
            (
                2,
                2,
                vec![255; 3],
                TerrainMaskError::BufferSize {
                    expected: 4,
                    actual: 3,
                },
            ),
            (
                2,
                2,
                vec![255; 5],
                TerrainMaskError::BufferSize {
                    expected: 4,
                    actual: 5,
                },
            ),
            (
                2,
                2,
                vec![0, 1, 2, 255],
                TerrainMaskError::CellValue { index: 2, value: 2 },
            ),
        ];
        for (w, h, mut cells, error) in cases {
            let before = cells.clone();
            assert_eq!(TerrainMaskMut::new(w, h, &mut cells), Err(error));
            assert_eq!(cells, before);
        }
        for value in 2..=255 {
            let mut cells = [0, 1, 0, value];
            assert_eq!(
                TerrainMaskMut::new(2, 2, &mut cells),
                Err(TerrainMaskError::CellValue { index: 3, value })
            );
            assert_eq!(cells, [0, 1, 0, value]);
        }
    }
    #[test]
    fn mutable_view_reborrows_the_same_storage_and_metadata() {
        let mut cells = [0, 1, 0, 1, 0, 1];
        let ptr = cells.as_ptr();
        let mask = TerrainMaskMut::new(3, 2, &mut cells).unwrap();
        let read = mask.as_read_only();
        assert_eq!(read.as_bytes().as_ptr(), ptr);
        assert_eq!((read.width(), read.height(), read.pixel_count()), (3, 2, 6));
        assert_eq!(read.row(1), Some(&[1, 0, 1][..]));
    }
    #[test]
    fn setting_and_clearing_preserve_every_neighbor_and_external_canary() {
        let mut cells = [255, 0, 1, 0, 1, 0, 1, 254];
        {
            let mut mask = TerrainMaskMut::new(3, 2, &mut cells[1..7]).unwrap();
            assert_eq!(mask.set_pixel(2, 0, true), Some(true));
            assert_eq!(mask.set_pixel(0, 1, false), Some(true));
            assert_eq!(mask.as_read_only().as_bytes(), [0, 1, 1, 0, 0, 1]);
        }
        assert_eq!(cells, [255, 0, 1, 1, 0, 0, 1, 254]);
    }
    #[test]
    fn repeated_writes_distinguish_changes_from_idempotent_values() {
        let mut cells = [0, 1];
        let mut mask = TerrainMaskMut::new(2, 1, &mut cells).unwrap();
        for (x, value, changed) in [
            (0, false, false),
            (1, true, false),
            (0, true, true),
            (0, true, false),
            (1, false, true),
            (1, false, false),
            (0, false, true),
            (1, true, true),
        ] {
            assert_eq!(mask.set_pixel(x, 0, value), Some(changed));
            assert_eq!(mask.as_read_only().pixel(x, 0), Some(value));
        }
        assert_eq!(mask.as_read_only().as_bytes(), [0, 1]);
    }
    #[test]
    fn unsigned_outside_writes_and_extremes_preserve_the_entire_buffer() {
        let mut cells = [0, 1, 0, 1, 0, 1];
        let before = cells;
        let mut mask = TerrainMaskMut::new(3, 2, &mut cells).unwrap();
        for (x, y) in [
            (3, 0),
            (0, 2),
            (3, 2),
            (usize::MAX, 0),
            (0, usize::MAX),
            (usize::MAX, usize::MAX),
        ] {
            for value in [false, true] {
                assert_eq!(mask.set_pixel(x, y, value), None);
                assert_eq!(mask.as_read_only().as_bytes(), before);
            }
        }
    }
    #[test]
    fn signed_outside_writes_reject_negative_row_aliases_and_extremes() {
        let mut cells = [0; 6];
        let mut mask = TerrainMaskMut::new(3, 2, &mut cells).unwrap();
        for (x, y) in [
            (-1, 1),
            (3, 0),
            (0, -1),
            (-1, -1),
            (i32::MIN, 0),
            (0, i32::MIN),
            (i32::MAX, 0),
            (0, i32::MAX),
        ] {
            assert_eq!(mask.set_solid(x, y, true), None);
            assert_eq!(mask.as_read_only().as_bytes(), [0; 6]);
        }
        assert_eq!(mask.set_solid(2, 1, true), Some(true));
        assert_eq!(mask.set_solid(2, 1, true), Some(false));
        assert!(mask.as_read_only().is_solid(2, 1));
    }
    #[test]
    fn read_only_reborrows_observe_edits_and_release_before_the_next_write() {
        let mut cells = [0; 4];
        let mut mask = TerrainMaskMut::new(2, 2, &mut cells).unwrap();
        let _ = mask.set_pixel(1, 0, true);
        {
            let read = mask.as_read_only();
            let copy = read;
            assert_eq!(read.row(0), Some(&[0, 1][..]));
            assert!(copy.is_solid(1, 0));
        }
        let _ = mask.set_solid(1, 0, false);
        let _ = mask.set_solid(0, 1, true);
        assert_eq!(mask.as_read_only().as_bytes(), [0, 0, 1, 0]);
    }
    #[test]
    fn consuming_the_mutable_view_retains_the_original_buffer_lifetime() {
        let mut cells = [0; 4];
        let ptr = cells.as_ptr();
        let read = {
            let mut mask = TerrainMaskMut::new(2, 2, &mut cells).unwrap();
            let _ = mask.set_pixel(1, 1, true);
            mask.into_read_only()
        };
        assert_eq!(read.as_bytes().as_ptr(), ptr);
        assert_eq!(read.pixel(1, 1), Some(true));
        assert_eq!(read.row(0), Some(&[0, 0][..]));
    }
    #[test]
    fn all_four_corners_and_adjacent_rows_use_exact_cell_addresses() {
        let mut cells = [0; 9];
        let mut mask = TerrainMaskMut::new(3, 3, &mut cells).unwrap();
        for (x, y) in [(0, 0), (2, 0), (0, 2), (2, 2)] {
            assert_eq!(mask.set_pixel(x, y, true), Some(true));
        }
        assert_eq!(mask.as_read_only().row(0), Some(&[1, 0, 1][..]));
        assert_eq!(mask.as_read_only().row(1), Some(&[0, 0, 0][..]));
        assert_eq!(mask.as_read_only().row(2), Some(&[1, 0, 1][..]));
        assert_eq!(mask.set_pixel(3, 0, true), None);
        assert_eq!(mask.set_pixel(0, 3, true), None);
        assert_eq!(mask.as_read_only().as_bytes(), [1, 0, 1, 0, 0, 0, 1, 0, 1]);
    }
    #[test]
    fn independent_mutable_subslices_do_not_share_edit_state() {
        let mut cells = [0; 8];
        {
            let (a, b) = cells.split_at_mut(4);
            let mut a = TerrainMaskMut::new(2, 2, a).unwrap();
            let mut b = TerrainMaskMut::new(1, 4, b).unwrap();
            let _ = a.set_pixel(1, 1, true);
            let _ = b.set_pixel(0, 0, true);
            let _ = b.set_solid(0, 3, true);
            let _ = a.set_solid(0, 0, true);
            assert_eq!(a.as_read_only().as_bytes(), [1, 0, 0, 1]);
            assert_eq!(b.as_read_only().as_bytes(), [1, 0, 0, 1]);
        }
        assert_eq!(cells, [1, 0, 0, 1, 1, 0, 0, 1]);
    }
    #[test]
    fn failed_construction_releases_the_buffer_for_later_valid_reuse() {
        let mut cells = [0, 1, 127, 0];
        assert_eq!(
            TerrainMaskMut::new(2, 2, &mut cells),
            Err(TerrainMaskError::CellValue {
                index: 2,
                value: 127
            })
        );
        cells[2] = 0;
        let mut mask = TerrainMaskMut::new(2, 2, &mut cells).unwrap();
        assert_eq!(mask.set_pixel(0, 1, true), Some(true));
        assert_eq!(mask.into_read_only().as_bytes(), [0, 1, 1, 0]);
    }
    #[test]
    fn exhaustive_cell_edits_match_coordinate_sets_and_pinned_ts_stream() {
        let (mut masks, mut attempts, mut valid_writes, mut hash) =
            (0, 0, 0, 0xcbf29ce484222325u64);
        let mut mix = |v: u8| {
            hash = (hash ^ u64::from(v)).wrapping_mul(0x100000001b3);
        };
        for width in 1..=3 {
            for height in 1..=3 {
                for pattern in 0u16..(1 << (width * height)) {
                    let coords: Vec<_> = (0..height)
                        .flat_map(|y| (0..width).map(move |x| (x, y)))
                        .collect();
                    let mut points: BTreeSet<_> = coords
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| pattern & (1 << i) != 0)
                        .map(|(_, p)| *p)
                        .collect();
                    let mut cells: Vec<_> = (0..coords.len())
                        .map(|i| ((pattern >> i) & 1) as u8)
                        .collect();
                    let original = cells.clone();
                    let mut mask =
                        TerrainMaskMut::new(width as usize, height as usize, &mut cells).unwrap();
                    for y in -1..=height {
                        for x in -1..=width {
                            let inside = x >= 0 && x < width && y >= 0 && y < height;
                            let restore = points.contains(&(x, y));
                            for (step, solid) in
                                [false, false, true, true, restore].into_iter().enumerate()
                            {
                                let expected = if inside {
                                    let previous = points.contains(&(x, y));
                                    if solid {
                                        points.insert((x, y));
                                    } else {
                                        points.remove(&(x, y));
                                    }
                                    valid_writes += 1;
                                    Some(previous != solid)
                                } else {
                                    None
                                };
                                let actual = if step % 2 == 0 {
                                    mask.set_solid(x, y, solid)
                                } else {
                                    mask.set_pixel(
                                        usize::try_from(x).unwrap_or(usize::MAX),
                                        usize::try_from(y).unwrap_or(usize::MAX),
                                        solid,
                                    )
                                };
                                assert_eq!(actual, expected);
                                mix(actual.map(u8::from).unwrap_or(2));
                                for (&p, &v) in coords.iter().zip(mask.as_read_only().as_bytes()) {
                                    assert_eq!(v, u8::from(points.contains(&p)));
                                    mix(v);
                                }
                                attempts += 1;
                            }
                        }
                    }
                    assert_eq!(mask.into_read_only().as_bytes(), original);
                    masks += 1;
                }
            }
        }
        assert_eq!((masks, attempts, valid_writes), (682, 79850, 27530));
        // Actual pinned TS valid-coordinate writes; explicit rejection elsewhere.
        assert_eq!(hash, 0x03ec6f2a34e44882);
        std::println!("682 masks / 79850 attempts / 27530 valid writes / fnv1a64={hash:016x}");
    }
}
