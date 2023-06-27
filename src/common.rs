use std::str;
use std::{num::ParseIntError, str::FromStr};

use crate::errors::Error;
use crate::parse::{FromUtf8, ParseUtf8};

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

/// Quick helper method for a lossy string
pub fn buf2lstring(buf: &[u8]) -> String {
    String::from_utf8_lossy(buf).to_string()
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl FromUtf8 for Color {
    fn from_utf8(buf: &[u8]) -> Result<Self, Error> {
        const RMASK: u32 = 0x0000FF;
        const GMASK: u32 = 0x00FF00;
        const BMASK: u32 = 0xFF0000;
        let num: u32 = buf.parse_utf8()?;
        Ok(Self {
            r: (num & RMASK).try_into().unwrap(),
            g: (num & GMASK >> 8).try_into().unwrap(),
            b: (num & BMASK >> 16).try_into().unwrap(),
        })
    }
}
