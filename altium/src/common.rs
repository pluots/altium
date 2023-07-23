use crate::errors::{ErrorKind, TruncBuf};
use crate::parse::{FromUtf8, ParseUtf8};

/// Separator in textlike streams
const SEP: u8 = b'|';
const KV_SEP: u8 = b'=';

/// Common coordinate type
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Coords2 {
    pub x: u8,
    pub y: u8,
}

impl Coords2 {
    fn new(x: u8, y: u8) -> Self {
        Self { x, y }
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
/// TODO: figure out what file types use this exact format
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UniqueId([u8; 8]);

impl UniqueId {
    pub fn from_slice<S: AsRef<[u8]>>(buf: S) -> Option<Self> {
        buf.as_ref().try_into().ok().map(Self)
    }
}

impl FromUtf8 for UniqueId {
    fn from_utf8(buf: &[u8]) -> Result<Self, ErrorKind> {
        Ok(Self(buf.as_ref().try_into().map_err(|_| {
            ErrorKind::InvalidUniqueId(TruncBuf::truncate(buf))
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
}

impl FromUtf8 for Color {
    fn from_utf8(buf: &[u8]) -> Result<Self, ErrorKind> {
        const RMASK: u32 = 0x0000FF;
        const GMASK: u32 = 0x00FF00;
        const BMASK: u32 = 0xFF0000;
        let num: u32 = buf.parse_as_utf8()?;
        Ok(Self {
            r: (num & RMASK).try_into().unwrap(),
            g: ((num & GMASK) >> 8).try_into().unwrap(),
            b: ((num & BMASK) >> 16).try_into().unwrap(),
        })
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

impl FromUtf8 for ReadOnlyState {
    fn from_utf8(buf: &[u8]) -> Result<Self, ErrorKind> {
        let num: u8 = buf.parse_as_utf8()?;
        num.try_into()
    }
}
