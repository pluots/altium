use std::{fmt, str};

use num_traits::CheckedMul;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AddContext, ErrorKind, Result, TruncBuf};
use crate::parse::{FromUtf8, ParseUtf8};

/// Separator in textlike streams
const SEP: u8 = b'|';
const KV_SEP: u8 = b'=';

/// Common coordinate type with x and y positions in nanometers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Location {
    // These are nonpublic because we might want to combine `Location` and `LocationFract`
    pub(crate) x: i32,
    pub(crate) y: i32,
}

/// Location with fraction
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LocationFract {
    pub x: i32,
    pub x_fract: i32,
    pub y: i32,
    pub y_fract: i32,
}

#[allow(clippy::cast_precision_loss)]
impl Location {
    #[inline]
    pub fn x(self) -> i32 {
        self.x
    }

    #[inline]
    pub fn y(self) -> i32 {
        self.y
    }

    #[inline]
    pub fn x_f32(self) -> f32 {
        self.x as f32
    }

    #[inline]
    pub fn y_f32(self) -> f32 {
        self.y as f32
    }

    #[must_use]
    pub(crate) fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub(crate) fn add_x(self, x: i32) -> Self {
        Self {
            x: self.x + x,
            y: self.y,
        }
    }

    #[must_use]
    pub(crate) fn add_y(self, y: i32) -> Self {
        Self {
            x: self.x,
            y: self.y + y,
        }
    }
}

impl LocationFract {
    /// Sometimes we don't want to deal with fractions
    #[inline]
    pub fn as_location(self) -> Location {
        Location {
            x: self.x,
            y: self.y,
        }
    }
}

impl From<(i32, i32)> for Location {
    fn from(value: (i32, i32)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum Visibility {
    Hidden,
    #[default]
    Visible,
}

impl FromUtf8<'_> for Visibility {
    fn from_utf8(buf: &[u8]) -> Result<Self, ErrorKind> {
        todo!("{}", String::from_utf8_lossy(buf))
    }
}

/// A unique ID that is assigned by Altium to each element in a design.
///
/// Every entity in Altium has a unique ID including files, library items, and records.
// TODO: figure out what file types use this exact format
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UniqueId {
    /// Altium's old style string unique ID
    #[serde(with = "unique_id_serde")]
    Simple([u8; 8]),
    /// UUID style, used by some newer files
    Uuid(Uuid),
}

impl UniqueId {
    #[allow(unused)]
    fn from_slice<S: AsRef<[u8]>>(buf: S) -> Option<Self> {
        buf.as_ref()
            .try_into()
            .ok()
            .map(Self::Simple)
            .or_else(|| Uuid::try_parse_ascii(buf.as_ref()).ok().map(Self::Uuid))
    }
}

impl fmt::Display for UniqueId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UniqueId::Simple(v) => str::from_utf8(v)
                .expect("unique IDs should always be ASCII")
                .fmt(f),
            UniqueId::Uuid(v) => v.as_hyphenated().fmt(f),
        }
    }
}

impl Default for UniqueId {
    fn default() -> Self {
        Self::Simple(*b"00000000")
    }
}

impl fmt::Debug for UniqueId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("UniqueId").field(&self.to_string()).finish()
    }
}

impl FromUtf8<'_> for UniqueId {
    fn from_utf8(buf: &[u8]) -> Result<Self, ErrorKind> {
        buf.as_ref()
            .try_into()
            .ok()
            .map(Self::Simple)
            .or_else(|| Uuid::try_parse_ascii(buf).ok().map(Self::Uuid))
            .ok_or(ErrorKind::InvalidUniqueId(TruncBuf::new(buf)))
    }
}

mod unique_id_serde {
    use serde::{de::Error, Deserialize, Deserializer, Serializer};

    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn serialize<S: Serializer>(val: &[u8; 8], serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(std::str::from_utf8(val).unwrap())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<[u8; 8], D::Error> {
        // let s = <String>::deserialize(deserializer);
        let s: &str = Deserialize::deserialize(deserializer)?;
        s.as_bytes()
            .try_into()
            .map_err(|_| D::Error::custom(format!("expected a 8-byte unique ID; got '{s}'")))
    }
}

/// Altium uses the format `Key1=Val1|Key2=Val2...`, this handles that
pub fn split_altium_map(buf: &[u8]) -> impl Iterator<Item = (&[u8], &[u8])> {
    buf.split(|b| *b == SEP).filter(|x| !x.is_empty()).map(|x| {
        split_once(x, KV_SEP).unwrap_or_else(|| panic!("couldn't find `=` in `{}`", buf2lstr(buf)))
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
pub fn buf2lstr(buf: &[u8]) -> String {
    String::from_utf8_lossy(buf).to_string()
}

/// `str::from_utf8` but with a context error
pub fn str_from_utf8(buf: &[u8]) -> Result<&str, ErrorKind> {
    str::from_utf8(buf).map_err(|e| ErrorKind::Utf8(e, String::from_utf8_lossy(buf).to_string()))
    // str::from_utf8(buf).map_err(|e| ErrorKind::Utf8(e, String::from_utf8_lossy(buf).to_string()))
}

/// RGB color
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub fn to_hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    pub const fn from_hex(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn as_float_rgba(self) -> [f32; 4] {
        [
            f32::from(self.r) / 255.0,
            f32::from(self.g) / 255.0,
            f32::from(self.b) / 255.0,
            1.0,
        ]
    }

    pub const fn black() -> Self {
        Self::from_hex(0x00, 0x00, 0x00)
    }

    pub const fn white() -> Self {
        Self::from_hex(0xff, 0xff, 0xff)
    }

    pub const fn red() -> Self {
        Self::from_hex(0xff, 0x00, 0x00)
    }

    pub const fn green() -> Self {
        Self::from_hex(0x00, 0xff, 0x00)
    }

    pub const fn blue() -> Self {
        Self::from_hex(0x00, 0x00, 0xff)
    }
}

impl FromUtf8<'_> for Rgb {
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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rotation90 {
    #[default]
    R0 = 0,
    R90 = 1,
    R180 = 2,
    R270 = 3,
}

impl Rotation90 {
    pub fn as_int(self) -> i16 {
        match self {
            Rotation90::R0 => 0,
            Rotation90::R90 => 90,
            Rotation90::R180 => 180,
            Rotation90::R270 => 270,
        }
    }
}

impl FromUtf8<'_> for Rotation90 {
    fn from_utf8(buf: &[u8]) -> Result<Self, ErrorKind> {
        todo!("{}", String::from_utf8_lossy(buf))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
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
            _ => return Err(ErrorKind::ReadOnlyState(value)),
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

/// Verify a number pattern matches, e.g. `X100`
pub fn is_number_pattern(s: &[u8], prefix: &[u8]) -> bool {
    if let Some(stripped) = s
        .strip_prefix(prefix)
        .map(|s| s.strip_prefix(&[b'-']).unwrap_or(s))
    {
        if stripped.iter().all(u8::is_ascii_digit) {
            return true;
        }
    }

    false
}

/// Infallible conversion
pub fn mils_to_nm<T>(mils: T) -> Result<T>
where
    T: CheckedMul,
    T: From<u16>,
    i64: From<T>,
{
    const FACTOR: u16 = 25400;
    mils.checked_mul(&FACTOR.into()).ok_or_else(|| {
        ErrorKind::Overflow(mils.into(), FACTOR.into(), '*').context("converting units")
    })
}
