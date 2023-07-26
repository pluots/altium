use std::{fmt, str};

use crate::error::{ErrorKind, TruncBuf};
use crate::parse::{FromUtf8, ParseUtf8};

/// Separator in textlike streams
const SEP: u8 = b'|';
const KV_SEP: u8 = b'=';

/// Common coordinate type
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Location {
    pub x: i32,
    pub y: i32,
}

impl Location {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn add_x(self, x: i32) -> Self {
        Self {
            x: self.x + x,
            y: self.y,
        }
    }

    pub fn add_y(self, y: i32) -> Self {
        Self {
            x: self.x,
            y: self.y + y,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Visibility {
    Hidden,
    #[default]
    Visible,
}

/// A unique ID
///
// TODO: figure out what file types use this exact format
#[derive(Clone, Copy, PartialEq)]
pub struct UniqueId([u8; 8]);

impl UniqueId {
    pub(crate) fn from_slice<S: AsRef<[u8]>>(buf: S) -> Option<Self> {
        buf.as_ref().try_into().ok().map(Self)
    }

    /// Get this `UniqueId` as a string
    pub fn as_str(&self) -> &str {
        str::from_utf8(&self.0).expect("unique IDs should always be ASCII")
    }
}

impl Default for UniqueId {
    fn default() -> Self {
        Self(*b"00000000")
    }
}

impl fmt::Debug for UniqueId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("UniqueId").field(&self.as_str()).finish()
    }
}

impl FromUtf8<'_> for UniqueId {
    fn from_utf8(buf: &[u8]) -> Result<Self, ErrorKind> {
        Ok(Self(buf.as_ref().try_into().map_err(|_| {
            ErrorKind::InvalidUniqueId(TruncBuf::new(buf))
        })?))
    }
}

/// Altium uses the format `Key1=Val1|Key2=Val2...`, this handles that
pub fn split_altium_map(buf: &[u8]) -> impl Iterator<Item = (&[u8], &[u8])> {
    buf.split(|b| *b == SEP).filter(|x| !x.is_empty()).map(|x| {
        split_once(x, KV_SEP)
            .unwrap_or_else(|| panic!("couldn't find `=` in `{}`", buf2lstring(buf)))
    })
}

/// Implement `str::split_once` for any buffer
///
/// Split at the first instance of a value
pub fn split_once<T>(buf: &[T], split: T) -> Option<(&[T], &[T])>
where
    T: PartialEq<T> + Copy,
{
    let pos = buf.iter().position(|x| *x == split)?;
    Some((&buf[..pos], &buf[pos + 1..]))
}

/// Quick helper method for a lossy string
pub fn buf2lstring(buf: &[u8]) -> String {
    String::from_utf8_lossy(buf).to_string()
}

/// `str::from_utf8` but with a context error
pub fn str_from_utf8(buf: &[u8]) -> Result<&str, ErrorKind> {
    str::from_utf8(buf).map_err(|e| ErrorKind::Utf8(e, String::from_utf8_lossy(buf).to_string()))
    // str::from_utf8(buf).map_err(|e| ErrorKind::Utf8(e, String::from_utf8_lossy(buf).to_string()))
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn to_hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    pub fn from_hex(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn black() -> Self {
        Self::from_hex(0x00, 0x00, 0x00)
    }

    pub fn white() -> Self {
        Self::from_hex(0xff, 0xff, 0xff)
    }

    pub fn red() -> Self {
        Self::from_hex(0xff, 0x00, 0x00)
    }

    pub fn green() -> Self {
        Self::from_hex(0x00, 0xff, 0x00)
    }

    pub fn blue() -> Self {
        Self::from_hex(0x00, 0x00, 0xff)
    }
}

impl FromUtf8<'_> for Color {
    fn from_utf8(buf: &[u8]) -> Result<Self, ErrorKind> {
        const RMASK: u32 = 0x0000ff;
        const GMASK: u32 = 0x00ff00;
        const BMASK: u32 = 0xff0000;
        let num: u32 = buf.parse_as_utf8()?;
        Ok(Self {
            r: (num & RMASK).try_into().unwrap(),
            g: ((num & GMASK) >> 8).try_into().unwrap(),
            b: ((num & BMASK) >> 16).try_into().unwrap(),
        })
    }
}

/// Rotation when only 4 values are allowed
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Rotation {
    #[default]
    R0 = 0,
    R90 = 1,
    R180 = 2,
    R270 = 3,
}

impl Rotation {
    pub fn as_int(self) -> i16 {
        match self {
            Rotation::R0 => 0,
            Rotation::R90 => 90,
            Rotation::R180 => 180,
            Rotation::R270 => 270,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ReadOnlyState {
    #[default]
    ReadWrite,
    ReadOnly,
}

impl TryFrom<u8> for ReadOnlyState {
    type Error = ErrorKind;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let res = match value {
            x if x == Self::ReadWrite as u8 => Self::ReadWrite,
            x if x == Self::ReadOnly as u8 => Self::ReadOnly,
            _ => return Err(ErrorKind::SheetStyle(value)),
        };

        Ok(res)
    }
}

impl FromUtf8<'_> for ReadOnlyState {
    fn from_utf8(buf: &[u8]) -> Result<Self, ErrorKind> {
        let num: u8 = buf.parse_as_utf8()?;
        num.try_into()
    }
}

/// Horizontal alignment
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum PosHoriz {
    #[default]
    Left,
    Center,
    Right,
}

/// Vertical alignment
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum PosVert {
    #[default]
    Top,
    Middle,
    Bottom,
}
