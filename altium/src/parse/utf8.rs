//! Traits related to parsing things encoded as utf8 (i.e., string "1234" and
//! not 0x1234)
//!
//! Usually these are things where we have a binary file but part of it is utf8
//! encoded.

use crate::{common::buf2lstring, ErrorKind};
use std::str;

/// Extension trait for `&[u8]` that will parse a string as utf8/ASCII for
/// anything implementing `FromUtf8`
pub trait ParseUtf8 {
    /// Parse this as utf8 to whatever the target type is
    fn parse_as_utf8<T: FromUtf8>(self) -> Result<T, ErrorKind>;
}

/// Trait saying that a type can be created from a utf8/ASCII string.
///
/// Implement this on anything we want to `buf.parse_as_utf8()` from
pub trait FromUtf8: Sized {
    /// Parse an entire buffer as this type
    fn from_utf8(buf: &[u8]) -> Result<Self, ErrorKind>;
}

impl<'a> ParseUtf8 for &'a [u8] {
    fn parse_as_utf8<T: FromUtf8>(self: &'a [u8]) -> Result<T, ErrorKind> {
        T::from_utf8(self)
    }
}

impl FromUtf8 for String {
    fn from_utf8(buf: &[u8]) -> Result<Self, ErrorKind> {
        Ok(str::from_utf8(buf)?.to_owned())
    }
}

impl FromUtf8 for Box<str> {
    fn from_utf8(buf: &[u8]) -> Result<Self, ErrorKind> {
        Ok(str::from_utf8(buf)?.into())
    }
}

impl FromUtf8 for bool {
    fn from_utf8(buf: &[u8]) -> Result<Self, ErrorKind> {
        if buf == b"T" {
            Ok(true)
        } else if buf == b"F" {
            Ok(false)
        } else {
            Err(ErrorKind::ExpectedBool(buf2lstring(buf)))
        }
    }
}

impl FromUtf8 for u8 {
    fn from_utf8(buf: &[u8]) -> Result<Self, ErrorKind> {
        let s = str::from_utf8(buf)?;
        s.parse().map_err(|e| ErrorKind::ExpectedInt(s.into(), e))
    }
}

impl FromUtf8 for i8 {
    fn from_utf8(buf: &[u8]) -> Result<Self, ErrorKind> {
        let s = str::from_utf8(buf)?;
        s.parse().map_err(|e| ErrorKind::ExpectedInt(s.into(), e))
    }
}

impl FromUtf8 for u16 {
    fn from_utf8(buf: &[u8]) -> Result<Self, ErrorKind> {
        let s = str::from_utf8(buf)?;
        s.parse().map_err(|e| ErrorKind::ExpectedInt(s.into(), e))
    }
}

impl FromUtf8 for i16 {
    fn from_utf8(buf: &[u8]) -> Result<Self, ErrorKind> {
        let s = str::from_utf8(buf)?;
        s.parse().map_err(|e| ErrorKind::ExpectedInt(s.into(), e))
    }
}

impl FromUtf8 for u32 {
    fn from_utf8(buf: &[u8]) -> Result<Self, ErrorKind> {
        let s = str::from_utf8(buf)?;
        s.parse().map_err(|e| ErrorKind::ExpectedInt(s.into(), e))
    }
}

impl FromUtf8 for i32 {
    fn from_utf8(buf: &[u8]) -> Result<Self, ErrorKind> {
        let s = str::from_utf8(buf)?;
        s.parse().map_err(|e| ErrorKind::ExpectedInt(s.into(), e))
    }
}

impl FromUtf8 for usize {
    fn from_utf8(buf: &[u8]) -> Result<Self, ErrorKind> {
        let s = str::from_utf8(buf)?;
        s.parse().map_err(|e| ErrorKind::ExpectedInt(s.into(), e))
    }
}
