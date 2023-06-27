use std::str;
use std::{num::ParseIntError, str::FromStr};

use crate::errors::Error;

#[derive(Debug, PartialEq)]
pub struct UniqueId([u8; 8]);

impl UniqueId {
    pub fn from_slice<S: AsRef<[u8]>>(buf: S) -> Option<Self> {
        buf.as_ref().try_into().ok().map(Self)
    }
}

/// Altium uses the format `Key1=Val1|Key2=Val2...`, this handles that
pub fn split_altium_map(buf: &[u8]) -> impl Iterator<Item = (&[u8], &[u8])> {
    buf.split(|b| *b == b'|')
        .filter(|x| !x.is_empty())
        .map(|x| split_once(x, b'=').unwrap())
}

/// Implement `str::split_once` for any buffer
pub fn split_once<T>(buf: &[T], split: T) -> Option<(&[T], &[T])>
where
    T: PartialEq<T>,
{
    let pos = buf.iter().position(|x| *x == split)?;
    Some((&buf[..pos], &buf[pos + 1..]))
}

/// Try to parse a utf8 buffer to an integer
pub fn parse_utf8<T>(buf: &[u8]) -> Result<T, Error>
where
    T: FromStr<Err = ParseIntError>,
{
    let s = str::from_utf8(buf)?;
    s.parse::<T>().map_err(|e| Error::ExpectedInt(s.into(), e))
}

/// Quick helper method
pub fn buf2string(buf: &[u8]) -> Result<String, Error> {
    Ok(str::from_utf8(buf)?.to_owned())
}

/// Quick helper method
pub fn buf2string_lossy(buf: &[u8]) -> String {
    String::from_utf8_lossy(buf).to_string()
}

pub fn parse_bool(buf: &[u8]) -> Result<bool, Error> {
    if buf == b"T" {
        Ok(true)
    } else if buf == b"F" {
        Ok(false)
    } else {
        Err(Error::ExpectedBool(buf2string_lossy(buf)))
    }
}

pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn parse(buf: &[u8]) -> Result<Self, Error> {
        if buf.len() != 3 {
            Err(Error::ExpectedColor(buf.into()))
        } else {
            Ok(Self {
                r: buf[0],
                g: buf[1],
                b: buf[2],
            })
        }
    }
}
