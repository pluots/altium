use crate::{common::buf2lstring, Error};
use std::{
    num::ParseIntError,
    str::{self, FromStr},
};

/// Extension trait for `&[u8]`
pub trait ParseUtf8 {
    /// Parse this as utf8 to whatever the target type is
    fn parse_utf8<T: FromUtf8>(self) -> Result<T, Error>;
}

/// Implement this on anything we want to `buf.parse_ascii()` from
pub trait FromUtf8: Sized {
    fn from_utf8(buf: &[u8]) -> Result<Self, Error>;
}

impl<'a> ParseUtf8 for &'a [u8] {
    fn parse_utf8<T: FromUtf8>(self: &'a [u8]) -> Result<T, Error> {
        T::from_utf8(self)
    }
}

impl FromUtf8 for String {
    fn from_utf8(buf: &[u8]) -> Result<Self, Error> {
        Ok(str::from_utf8(buf)?.to_owned())
    }
}

impl FromUtf8 for Box<str> {
    fn from_utf8(buf: &[u8]) -> Result<Self, Error> {
        Ok(str::from_utf8(buf)?.into())
    }
}

impl FromUtf8 for bool {
    fn from_utf8(buf: &[u8]) -> Result<Self, Error> {
        if buf == b"T" {
            Ok(true)
        } else if buf == b"F" {
            Ok(false)
        } else {
            Err(Error::ExpectedBool(buf2lstring(buf)))
        }
    }
}

impl FromUtf8 for u8 {
    fn from_utf8(buf: &[u8]) -> Result<Self, Error> {
        let s = str::from_utf8(buf)?;
        s.parse().map_err(|e| Error::ExpectedInt(s.into(), e))
    }
}

impl FromUtf8 for i8 {
    fn from_utf8(buf: &[u8]) -> Result<Self, Error> {
        let s = str::from_utf8(buf)?;
        s.parse().map_err(|e| Error::ExpectedInt(s.into(), e))
    }
}

impl FromUtf8 for u16 {
    fn from_utf8(buf: &[u8]) -> Result<Self, Error> {
        let s = str::from_utf8(buf)?;
        s.parse().map_err(|e| Error::ExpectedInt(s.into(), e))
    }
}

impl FromUtf8 for u32 {
    fn from_utf8(buf: &[u8]) -> Result<Self, Error> {
        let s = str::from_utf8(buf)?;
        s.parse().map_err(|e| Error::ExpectedInt(s.into(), e))
    }
}

impl FromUtf8 for usize {
    fn from_utf8(buf: &[u8]) -> Result<Self, Error> {
        let s = str::from_utf8(buf)?;
        s.parse().map_err(|e| Error::ExpectedInt(s.into(), e))
    }
}
