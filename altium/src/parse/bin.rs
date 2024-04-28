use std::{fmt::Debug, str};

use crate::{common::str_from_utf8, error::TruncBuf, ErrorKind};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BufLenMatch {
    /// Length is a 3-byte value in a 4-byte integer, and the upper value must
    /// be equal to 0x01. This is used to indicate binary data
    U24UpperOne,
    /// 3-byte value in a 4-byte integer, the upper value must be equal to 0x00
    U24UpperZero,
    /// Length is a 4-byte value
    U32,
    /// Length is a single byte value
    U8,
}

/// Extract a buffer that starts with a 1-, 3-, or 4-byte header.
///
/// - `len_match`: Configure how the leading bytes define the length
/// - `expect_nul`: Configure whether or not there should be a nul terminator
pub fn extract_sized_buf(
    buf: &[u8],
    len_match: BufLenMatch,
    expect_nul: bool,
) -> Result<(&[u8], &[u8]), ErrorKind> {
    let (data_len, rest): (usize, _) = match len_match {
        BufLenMatch::U24UpperOne | BufLenMatch::U24UpperZero => {
            let (arr, rest) = split_chunk::<4>(buf)?;
            let mut arr = *arr;
            let l3 = arr[3];
            arr[3] = 0x00;

            if len_match == BufLenMatch::U24UpperOne {
                assert_eq!(l3, 0x01, "expected 0x01 in uppper bit but got {l3}");
            } else if len_match == BufLenMatch::U24UpperZero {
                assert_eq!(l3, 0x00, "expected 0x00 in uppper bit but got {l3}");
            }

            let len = u32::from_le_bytes(arr).try_into().unwrap();
            (len, rest)
        }
        BufLenMatch::U32 => {
            let (arr, rest) = split_chunk::<4>(buf)?;
            let len = u32::from_le_bytes(*arr).try_into().unwrap();
            (len, rest)
        }
        BufLenMatch::U8 => {
            let (arr, rest) = split_chunk::<1>(buf)?;
            (arr[0].into(), rest)
        }
    };

    let data = rest
        .get(..data_len)
        .ok_or(ErrorKind::BufferTooShort(data_len, rest.into()))?;
    let rest = &rest[data_len..];

    if expect_nul {
        let Some(0) = data.last() else {
            return Err(ErrorKind::ExpectedNul(TruncBuf::new_end(data)));
        };
        Ok((&data[..data.len() - 1], rest))
    } else {
        Ok((data, rest))
    }
}

/// Helper method for `split_first_chunk` that returns a buffer error
pub fn split_chunk<const N: usize>(buf: &[u8]) -> Result<(&[u8; N], &[u8]), ErrorKind> {
    buf.split_first_chunk::<N>()
        .ok_or_else(|| ErrorKind::BufferTooShort(N, TruncBuf::new(buf)))
}

/// Extract a buffer that starts with a 1-, 3- or 4-byte header to a string
pub fn extract_sized_utf8_buf(
    buf: &[u8],
    len_match: BufLenMatch,
    expect_nul: bool,
) -> Result<(&str, &[u8]), ErrorKind> {
    let (str_buf, rest) = extract_sized_buf(buf, len_match, expect_nul)?;
    let text = str_from_utf8(str_buf)?;
    Ok((text, rest))
}
