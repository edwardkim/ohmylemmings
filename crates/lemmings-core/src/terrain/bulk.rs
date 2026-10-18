use super::{TerrainMask, TerrainMaskMut};

/// Half-open destination bounds of actual changed cells, not the whole request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TerrainRect {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

/// Changed pixels and their minimal bounding rectangle. A no-op has 0/None.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TerrainChange {
    pub changed_pixels: usize,
    pub bounds: Option<TerrainRect>,
}
impl TerrainChange {
    fn include(&mut self, x: usize, y: usize) {
        self.changed_pixels += 1;
        self.bounds = Some(match self.bounds {
            None => TerrainRect {
                x,
                y,
                width: 1,
                height: 1,
            },
            Some(old) => {
                let left = old.x.min(x);
                let top = old.y.min(y);
                let right = (old.x + old.width).max(x + 1);
                let bottom = (old.y + old.height).max(y + 1);
                TerrainRect {
                    x: left,
                    y: top,
                    width: right - left,
                    height: bottom - top,
                }
            }
        });
    }
}

impl TerrainMaskMut<'_> {
    /// Set or clear the clipped rectangle. Zero extents and fully outside areas
    /// are no-ops. Huge extents never require origin+length or area arithmetic.
    /// Count and bounds describe only cells whose value actually changed.
    ///
    /// ```
    /// use lemmings_core::{TerrainChange, TerrainMaskMut, TerrainRect};
    /// let mut cells = [0; 9];
    /// let mut terrain = TerrainMaskMut::new(3, 3, &mut cells)?;
    /// assert_eq!(terrain.fill_rect(-1, 1, 3, 3, true), TerrainChange {
    ///     changed_pixels: 4,
    ///     bounds: Some(TerrainRect { x: 0, y: 1, width: 2, height: 2 }),
    /// });
    /// assert_eq!(terrain.fill_rect(0, 1, 2, 2, true), TerrainChange::default());
    /// # Ok::<(), lemmings_core::TerrainMaskError>(())
    /// ```
    pub fn fill_rect(
        &mut self,
        x: i32,
        y: i32,
        width: usize,
        height: usize,
        solid: bool,
    ) -> TerrainChange {
        self.edit_region(x, y, width, height, solid, |_, _| true)
    }

    /// Apply a validated binary selection: 1 sets/clears, 0 preserves. The mask
    /// top-left is placed at (x,y), without rotation, mirroring, or anchor bias.
    /// Clipping preserves the source row/column alignment. Mask data is immutable.
    ///
    /// ```
    /// use lemmings_core::{TerrainMask, TerrainMaskMut};
    /// let selection = [1, 0, 1, 0, 1, 0];
    /// let mask = TerrainMask::new(3, 2, &selection)?;
    /// let mut cells = [1; 6];
    /// let mut terrain = TerrainMaskMut::new(3, 2, &mut cells)?;
    /// assert_eq!(terrain.apply_mask(-1, 0, mask, false).changed_pixels, 2);
    /// assert_eq!(terrain.as_read_only().row(0), Some(&[1, 0, 1][..]));
    /// assert_eq!(terrain.as_read_only().row(1), Some(&[0, 1, 1][..]));
    /// assert_eq!(mask.as_bytes(), selection);
    /// # Ok::<(), lemmings_core::TerrainMaskError>(())
    /// ```
    ///
    /// Source and destination cannot share a still-borrowed view:
    ///
    /// ```compile_fail,E0502
    /// use lemmings_core::TerrainMaskMut;
    /// let mut cells = [1; 4];
    /// let mut terrain = TerrainMaskMut::new(2, 2, &mut cells).unwrap();
    /// let source = terrain.as_read_only();
    /// let _ = terrain.apply_mask(0, 0, source, false);
    /// ```
    pub fn apply_mask(
        &mut self,
        x: i32,
        y: i32,
        mask: TerrainMask<'_>,
        solid: bool,
    ) -> TerrainChange {
        self.edit_region(x, y, mask.width(), mask.height(), solid, |sx, sy| {
            mask.pixel(sx, sy).expect("clipped source coordinates")
        })
    }

    fn edit_region(
        &mut self,
        x: i32,
        y: i32,
        width: usize,
        height: usize,
        solid: bool,
        mut selected: impl FnMut(usize, usize) -> bool,
    ) -> TerrainChange {
        let Some((dx, sx, width)) = clip_axis(x, width, self.width) else {
            return TerrainChange::default();
        };
        let Some((dy, sy, height)) = clip_axis(y, height, self.height) else {
            return TerrainChange::default();
        };
        let mut result = TerrainChange::default();
        let value = u8::from(solid);
        for row in 0..height {
            for column in 0..width {
                if !selected(sx + column, sy + row) {
                    continue;
                }
                let (x, y) = (dx + column, dy + row);
                let cell = &mut self.cells[y * self.width + x];
                if *cell != value {
                    *cell = value;
                    result.include(x, y);
                }
            }
        }
        result
    }
}

// Returns destination start, source start, count; never sums signed origins and
// caller extents. unsigned_abs handles i32::MIN. Every sum in the loops is bounded
// by a validated source/destination size, including on wasm32.
fn clip_axis(origin: i32, length: usize, limit: usize) -> Option<(usize, usize, usize)> {
    let (destination, source, count) = if origin >= 0 {
        let destination = usize::try_from(origin).ok()?;
        (destination, 0, length.min(limit.checked_sub(destination)?))
    } else {
        let source = usize::try_from(origin.unsigned_abs()).ok()?;
        (0, source, length.checked_sub(source)?.min(limit))
    };
    (count != 0).then_some((destination, source, count))
}
#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use std::{collections::BTreeSet, vec, vec::Vec};
    fn change(n: usize, x: usize, y: usize, width: usize, height: usize) -> TerrainChange {
        TerrainChange {
            changed_pixels: n,
            bounds: Some(TerrainRect {
                x,
                y,
                width,
                height,
            }),
        }
    }
    #[test]
    fn rectangle_add_changes_only_requested_cells() {
        let mut cells = [0; 20];
        let mut map = TerrainMaskMut::new(5, 4, &mut cells).unwrap();
        assert_eq!(map.fill_rect(1, 1, 3, 2, true), change(6, 1, 1, 3, 2));
        assert_eq!(
            map.as_read_only().as_bytes(),
            [0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0]
        );
    }
    #[test]
    fn rectangle_erase_reports_actual_changed_cells_and_minimal_bounds() {
        let mut cells = [0; 20];
        cells[7] = 1;
        cells[12] = 1;
        let mut map = TerrainMaskMut::new(5, 4, &mut cells).unwrap();
        assert_eq!(map.fill_rect(0, 0, 5, 4, false), change(2, 2, 1, 1, 2));
        assert_eq!(map.as_read_only().as_bytes(), [0; 20]);
    }
    #[test]
    fn zero_extents_fully_outside_and_repeated_rectangles_are_noops() {
        let mut cells = [0; 4];
        let mut map = TerrainMaskMut::new(2, 2, &mut cells).unwrap();
        for (x, y, w, h) in [
            (0, 0, 0, 2),
            (0, 0, 2, 0),
            (2, 0, 3, 3),
            (0, 2, 3, 3),
            (-2, 0, 2, 2),
            (0, -2, 2, 2),
        ] {
            assert_eq!(map.fill_rect(x, y, w, h, true), TerrainChange::default());
        }
        assert_eq!(map.fill_rect(0, 0, 2, 2, false), TerrainChange::default());
        assert_eq!(map.fill_rect(0, 0, 2, 2, true), change(4, 0, 0, 2, 2));
        assert_eq!(map.fill_rect(0, 0, 2, 2, true), TerrainChange::default());
    }
    #[test]
    fn rectangle_clips_all_four_edges_and_keeps_external_canaries() {
        let mut cells = [255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 254];
        {
            let mut map = TerrainMaskMut::new(3, 3, &mut cells[1..10]).unwrap();
            assert_eq!(map.fill_rect(-1, -1, 3, 3, true), change(4, 0, 0, 2, 2));
            assert_eq!(map.fill_rect(2, 2, 3, 3, true), change(1, 2, 2, 1, 1));
            assert_eq!(map.as_read_only().as_bytes(), [1, 1, 0, 1, 1, 0, 0, 0, 1]);
        }
        assert_eq!(cells[0], 255);
        assert_eq!(cells[10], 254);
    }
    #[test]
    fn mask_ones_select_add_and_erase_while_zeroes_preserve() {
        let source = [1, 0, 1, 0, 1, 0];
        let mask = TerrainMask::new(3, 2, &source).unwrap();
        for solid in [false, true] {
            let mut cells = vec![u8::from(!solid); 6];
            let mut map = TerrainMaskMut::new(3, 2, &mut cells).unwrap();
            assert_eq!(map.apply_mask(0, 0, mask, solid), change(3, 0, 0, 3, 2));
            let expected: Vec<_> = source
                .iter()
                .map(|&v| {
                    if v == 1 {
                        u8::from(solid)
                    } else {
                        u8::from(!solid)
                    }
                })
                .collect();
            assert_eq!(map.as_read_only().as_bytes(), expected);
        }
        assert_eq!(source, [1, 0, 1, 0, 1, 0]);
    }
    #[test]
    fn negative_mask_placement_preserves_source_rows_and_columns_without_flipping() {
        let mask = TerrainMask::new(3, 3, &[1, 0, 0, 0, 1, 0, 0, 0, 1]).unwrap();
        let mut cells = [0; 6];
        let mut map = TerrainMaskMut::new(3, 2, &mut cells).unwrap();
        assert_eq!(map.apply_mask(-1, -1, mask, true), change(2, 0, 0, 2, 2));
        assert_eq!(map.as_read_only().as_bytes(), [1, 0, 0, 0, 1, 0]);
    }
    #[test]
    fn mask_selection_outside_zero_mask_and_repeated_edits_have_no_changes() {
        let source = [1, 0, 0, 0];
        let mask = TerrainMask::new(2, 2, &source).unwrap();
        let zero = TerrainMask::new(2, 2, &[0; 4]).unwrap();
        let mut cells = [0; 4];
        let mut map = TerrainMaskMut::new(2, 2, &mut cells).unwrap();
        assert_eq!(map.apply_mask(-1, 0, mask, true), TerrainChange::default());
        assert_eq!(map.apply_mask(0, 0, zero, true), TerrainChange::default());
        assert_eq!(map.apply_mask(0, 0, mask, true), change(1, 0, 0, 1, 1));
        assert_eq!(map.apply_mask(0, 0, mask, true), TerrainChange::default());
    }
    #[test]
    fn mask_change_bounds_exclude_selected_but_already_equal_cells() {
        let source = [1; 9];
        let mask = TerrainMask::new(3, 3, &source).unwrap();
        let mut cells = [1; 9];
        cells[4] = 0;
        let mut map = TerrainMaskMut::new(3, 3, &mut cells).unwrap();
        assert_eq!(map.apply_mask(0, 0, mask, true), change(1, 1, 1, 1, 1));
        assert_eq!(map.as_read_only().as_bytes(), [1; 9]);
    }
    #[test]
    fn enormous_rectangle_extents_clip_without_area_or_endpoint_overflow() {
        let mut cells = [0; 6];
        let mut map = TerrainMaskMut::new(3, 2, &mut cells).unwrap();
        assert_eq!(
            map.fill_rect(0, 0, usize::MAX, usize::MAX, true),
            change(6, 0, 0, 3, 2)
        );
        assert_eq!(
            map.fill_rect(-1, -1, usize::MAX, usize::MAX, false),
            change(6, 0, 0, 3, 2)
        );
        assert_eq!(map.as_read_only().as_bytes(), [0; 6]);
    }
    #[test]
    fn minimum_signed_origins_skip_exactly_without_negation_overflow() {
        let skip = usize::try_from(i32::MIN.unsigned_abs()).unwrap();
        let mut cells = [0; 4];
        let mut map = TerrainMaskMut::new(2, 2, &mut cells).unwrap();
        assert_eq!(
            map.fill_rect(i32::MIN, 0, skip, 2, true),
            TerrainChange::default()
        );
        assert_eq!(
            map.fill_rect(i32::MIN, i32::MIN, skip + 1, skip + 1, true),
            change(1, 0, 0, 1, 1)
        );
        assert_eq!(
            map.fill_rect(i32::MIN, i32::MIN, usize::MAX, usize::MAX, true),
            change(3, 0, 0, 2, 2)
        );
        assert_eq!(
            map.fill_rect(i32::MAX, i32::MAX, usize::MAX, usize::MAX, false),
            TerrainChange::default()
        );
        assert_eq!(map.as_read_only().as_bytes(), [1; 4]);
    }
    #[test]
    fn extreme_mask_origins_are_outside_and_preserve_both_buffers() {
        let source = [1, 0, 0, 1];
        let mask = TerrainMask::new(2, 2, &source).unwrap();
        let mut cells = [0; 4];
        let mut map = TerrainMaskMut::new(2, 2, &mut cells).unwrap();
        for (x, y) in [
            (i32::MIN, 0),
            (0, i32::MIN),
            (i32::MAX, 0),
            (0, i32::MAX),
            (i32::MIN, i32::MAX),
        ] {
            assert_eq!(map.apply_mask(x, y, mask, true), TerrainChange::default());
            assert_eq!(map.as_read_only().as_bytes(), [0; 4]);
        }
        assert_eq!(mask.as_bytes(), source);
    }
    #[test]
    fn mixed_add_remove_order_is_immediate_and_deterministic() {
        let mask = TerrainMask::new(2, 2, &[1, 0, 0, 1]).unwrap();
        let mut cells = [0; 4];
        let mut map = TerrainMaskMut::new(2, 2, &mut cells).unwrap();
        assert_eq!(map.fill_rect(0, 0, 2, 2, true), change(4, 0, 0, 2, 2));
        assert_eq!(map.apply_mask(0, 0, mask, false), change(2, 0, 0, 2, 2));
        assert_eq!(map.as_read_only().as_bytes(), [0, 1, 1, 0]);
        assert_eq!(map.apply_mask(0, 0, mask, true), change(2, 0, 0, 2, 2));
        assert_eq!(map.fill_rect(0, 0, 2, 2, false), change(4, 0, 0, 2, 2));
        assert_eq!(map.as_read_only().as_bytes(), [0; 4]);
    }
    #[test]
    fn distinct_source_and_destination_subslices_keep_all_source_bytes_and_canaries() {
        let mut storage = [255, 1, 0, 0, 1, 254, 253, 0, 0, 0, 0, 252];
        {
            let (a, b) = storage.split_at_mut(6);
            let source = TerrainMask::new(2, 2, &a[1..5]).unwrap();
            let mut dest = TerrainMaskMut::new(2, 2, &mut b[1..5]).unwrap();
            assert_eq!(dest.apply_mask(0, 0, source, true), change(2, 0, 0, 2, 2));
            assert_eq!(source.as_bytes(), [1, 0, 0, 1]);
        }
        assert_eq!(storage, [255, 1, 0, 0, 1, 254, 253, 1, 0, 0, 1, 252]);
    }
    fn verify_case(
        dest: (i32, i32, u16),
        origin: (i32, i32),
        source: (i32, i32, Option<u16>),
        solid: bool,
    ) -> (TerrainChange, Vec<u8>) {
        let (dw, dh, pattern) = dest;
        let (x, y) = origin;
        let (sw, sh, bits) = source;
        let coords: Vec<_> = (0..dh).flat_map(|y| (0..dw).map(move |x| (x, y))).collect();
        let before: BTreeSet<_> = coords
            .iter()
            .enumerate()
            .filter(|(i, _)| pattern & (1 << i) != 0)
            .map(|(_, p)| *p)
            .collect();
        let mut wanted = before.clone();
        for sy in 0..sh {
            for sx in 0..sw {
                if bits.is_some_and(|bits| bits & (1 << (sy * sw + sx)) == 0) {
                    continue;
                }
                let p = (x + sx, y + sy);
                if p.0 >= 0 && p.0 < dw && p.1 >= 0 && p.1 < dh {
                    if solid {
                        wanted.insert(p);
                    } else {
                        wanted.remove(&p);
                    }
                }
            }
        }
        let changed: Vec<_> = before.symmetric_difference(&wanted).copied().collect();
        let expected = if changed.is_empty() {
            TerrainChange::default()
        } else {
            let left = changed.iter().map(|p| p.0).min().unwrap() as usize;
            let top = changed.iter().map(|p| p.1).min().unwrap() as usize;
            let right = changed.iter().map(|p| p.0).max().unwrap() as usize + 1;
            let bottom = changed.iter().map(|p| p.1).max().unwrap() as usize + 1;
            change(changed.len(), left, top, right - left, bottom - top)
        };
        let mut cells: Vec<_> = (0..coords.len())
            .map(|i| ((pattern >> i) & 1) as u8)
            .collect();
        let actual = {
            let mut map = TerrainMaskMut::new(dw as usize, dh as usize, &mut cells).unwrap();
            if let Some(bits) = bits {
                let source: Vec<_> = (0..sw * sh).map(|i| ((bits >> i) & 1) as u8).collect();
                let mask = TerrainMask::new(sw as usize, sh as usize, &source).unwrap();
                let result = map.apply_mask(x, y, mask, solid);
                assert_eq!(mask.as_bytes(), source);
                result
            } else {
                map.fill_rect(x, y, sw as usize, sh as usize, solid)
            }
        };
        assert_eq!(actual, expected);
        for (&p, &v) in coords.iter().zip(&cells) {
            assert_eq!(v, u8::from(wanted.contains(&p)));
        }
        (actual, cells)
    }
    #[test]
    fn exhaustive_region_edits_match_coordinate_sets_and_pinned_ts_stream() {
        let (mut operations, mut rectangles, mut masked, mut hash) =
            (0, 0, 0, 0xcbf29ce484222325u64);
        let mut mix = |v: u8| {
            hash = (hash ^ u64::from(v)).wrapping_mul(0x100000001b3);
        };
        let mut run = |dest, origin, source: (i32, i32, Option<u16>), solid| {
            let (change, cells) = verify_case(dest, origin, source, solid);
            mix(change.changed_pixels as u8);
            mix(u8::from(change.bounds.is_some()));
            if let Some(b) = change.bounds {
                for v in [b.x, b.y, b.width, b.height] {
                    mix(v as u8);
                }
            }
            for v in cells {
                mix(v);
            }
            operations += 1;
            if source.2.is_some() {
                masked += 1;
            } else {
                rectangles += 1;
            }
        };
        for dw in 1..=2 {
            for dh in 1..=2 {
                for pattern in 0u16..(1 << (dw * dh)) {
                    for y in -2..=dh {
                        for x in -2..=dw {
                            for rw in 0..=3 {
                                for rh in 0..=3 {
                                    for solid in [false, true] {
                                        run((dw, dh, pattern), (x, y), (rw, rh, None), solid);
                                    }
                                }
                            }
                            for sw in 1..=2 {
                                for sh in 1..=2 {
                                    for bits in 0u16..(1 << (sw * sh)) {
                                        for solid in [false, true] {
                                            run(
                                                (dw, dh, pattern),
                                                (x, y),
                                                (sw, sh, Some(bits)),
                                                solid,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        assert_eq!((operations, rectangles, masked), (49728, 18944, 30784));
        assert_eq!(hash, 0xf4207a057fb5986b);
        std::println!("49728 operations / 18944 rectangles / 30784 masked / fnv1a64={hash:016x}");
    }
}
