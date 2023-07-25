use std::str;

use crate::{common::split_once, error::TruncBuf, Error, ErrorKind};

#[derive(Clone, Copy, Debug)]
pub enum BufLenMatch {
    /// Length is a 3-byte value in a 4-byte integer, and the upper value must
    /// be equal to 0x01. This is used to indicate binary data
    U24UpperOne,
    /// Length is a 4-byte value
    U32,
    /// Length is a single byte value
    U8,
}

/// Extract a buffer that starts with a 3-byte header
pub fn extract_sized_buf(buf: &[u8], len_match: BufLenMatch) -> Result<(&[u8], &[u8]), ErrorKind> {
    let (data_len, rest): (usize, _) = match len_match {
        BufLenMatch::U24UpperOne => {
            let [l0, l1, l2, l3, rest @ ..] = buf else {
                return Err(ErrorKind::BufferTooShort(4, TruncBuf::new(buf)));
            };

            assert_eq!(*l3, 0x01, "expected 0x01 in uppper bit but got {l3}");
            let len = u32::from_le_bytes([*l0, *l1, *l2, 0x00])
                .try_into()
                .unwrap();
            (len, rest)
        }
        BufLenMatch::U32 => {
            let (Some(len_buf), Some(rest)) = (buf.get(..4), buf.get(4..)) else {
                return Err(ErrorKind::BufferTooShort(4, TruncBuf::new(buf)));
            };
            let len = u32::from_le_bytes(len_buf.try_into().unwrap())
                .try_into()
                .unwrap();
            (len, rest)
        }
        BufLenMatch::U8 => {
            let [l0, rest @ ..] = buf else {
                return Err(ErrorKind::BufferTooShort(4, TruncBuf::new(buf)));
            };
            ((*l0).into(), rest)
        }
    };

    let data = rest
        .get(..data_len)
        .ok_or(ErrorKind::BufferTooShort(data_len, rest.into()))?;
    Ok((data, &rest[data_len..]))
}

/// Extract a buffer that starts with a 3-byte header to a string
pub fn extract_sized_utf8_buf(
    buf: &[u8],
    len_match: BufLenMatch,
) -> Result<(&str, &[u8]), ErrorKind> {
    let (str_buf, rest) = extract_sized_buf(buf, len_match)?;
    let text = str::from_utf8(str_buf)?;
    Ok((text, rest))
}
