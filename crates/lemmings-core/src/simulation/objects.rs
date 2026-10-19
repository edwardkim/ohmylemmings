use core::fmt;

pub const OBJECT_NONE: u8 = 128;
pub const OBJECT_WATER: u8 = 133;
pub const OBJECT_FIRE: u8 = 134;
pub const OBJECT_EXIT: u8 = 129;
pub const OBJECT_FORCE_LEFT: u8 = 130;
pub const OBJECT_FORCE_RIGHT: u8 = 131;
pub const OBJECT_BLOCKER: u8 = 138;
// Game.pas InitializeObjectMap: (1647+16) div4 by (175+16) div4.
pub const DOS_OBJECT_MAP_WIDTH: usize = 415;
pub const DOS_OBJECT_MAP_HEIGHT: usize = 47;
pub const DOS_OBJECT_MAP_ORIGIN: i32 = -16;

/// Caller-owned raw object codes on the DOS four-pixel grid. Unknown trigger
/// bytes are preserved; reading them does not execute the trigger. The origin
/// is an aligned world coordinate, and negative coordinates round down.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObjectMap<'a> {
    width: usize,
    height: usize,
    origin_x: i32,
    origin_y: i32,
    cells: &'a [u8],
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObjectMapError {
    EmptyDimensions,
    AreaOverflow,
    BufferSize { expected: usize, actual: usize },
    UnalignedOrigin,
}
impl fmt::Display for ObjectMapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyDimensions => f.write_str("object map dimensions must be nonzero"),
            Self::AreaOverflow => f.write_str("object map area overflow"),
            Self::BufferSize { expected, actual } => {
                write!(f, "object map requires {expected} bytes, got {actual}")
            }
            Self::UnalignedOrigin => f.write_str("object map origin must be divisible by four"),
        }
    }
}
impl core::error::Error for ObjectMapError {}
impl<'a> ObjectMap<'a> {
    pub fn new(
        width: usize,
        height: usize,
        origin_x: i32,
        origin_y: i32,
        cells: &'a [u8],
    ) -> Result<Self, ObjectMapError> {
        if width == 0 || height == 0 {
            return Err(ObjectMapError::EmptyDimensions);
        }
        let expected = width
            .checked_mul(height)
            .ok_or(ObjectMapError::AreaOverflow)?;
        if cells.len() != expected {
            return Err(ObjectMapError::BufferSize {
                expected,
                actual: cells.len(),
            });
        }
        if origin_x % 4 != 0 || origin_y % 4 != 0 {
            return Err(ObjectMapError::UnalignedOrigin);
        }
        Ok(Self {
            width,
            height,
            origin_x,
            origin_y,
            cells,
        })
    }
    pub fn read(self, x: i64, y: i64) -> u8 {
        self.index(x, y).map_or(OBJECT_NONE, |i| self.cells[i])
    }
    fn index(self, x: i64, y: i64) -> Option<usize> {
        // Divide first: even the full i64 input domain cannot overflow when
        // subtracting the aligned i32 origin after division.
        let x = usize::try_from(x.div_euclid(4) - i64::from(self.origin_x / 4)).ok()?;
        let y = usize::try_from(y.div_euclid(4) - i64::from(self.origin_y / 4)).ok()?;
        (x < self.width && y < self.height).then(|| y * self.width + x)
    }
    pub const fn as_bytes(self) -> &'a [u8] {
        self.cells
    }
    pub const fn width(self) -> usize {
        self.width
    }
    pub const fn height(self) -> usize {
        self.height
    }
}
/// Exclusive raw object storage. Blocker transitions save/restore nine cells;
/// collision terrain remains separate and is never changed by these writes.
pub struct ObjectMapMut<'a> {
    width: usize,
    height: usize,
    origin_x: i32,
    origin_y: i32,
    cells: &'a mut [u8],
}
impl<'a> ObjectMapMut<'a> {
    pub fn new(
        width: usize,
        height: usize,
        origin_x: i32,
        origin_y: i32,
        cells: &'a mut [u8],
    ) -> Result<Self, ObjectMapError> {
        ObjectMap::new(width, height, origin_x, origin_y, cells)?;
        Ok(Self {
            width,
            height,
            origin_x,
            origin_y,
            cells,
        })
    }
    pub fn as_read_only(&self) -> ObjectMap<'_> {
        ObjectMap {
            width: self.width,
            height: self.height,
            origin_x: self.origin_x,
            origin_y: self.origin_y,
            cells: self.cells,
        }
    }
    pub fn write(&mut self, x: i64, y: i64, code: u8) -> bool {
        if let Some(i) = self.as_read_only().index(x, y) {
            self.cells[i] = code;
            true
        } else {
            false
        }
    }
}
