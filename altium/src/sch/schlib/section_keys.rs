use std::io::{Read, Seek};
use std::str;

use cfb::CompoundFile;

use super::SchLibMeta;
use crate::common::split_altium_map;
use crate::error::ErrorKind;
use crate::logging::log_unsupported_key;
use crate::parse::ParseUtf8;
use crate::Error;

const STREAMNAME: &str = "SectionKeys";
const PFX_LEN: usize = 5;
const SFX: &[u8] = &[0x00];
const LIBREF: &[u8] = b"LibRef";
const SECKEY: &[u8] = b"SectionKey";

/// Update a header with section keys.
pub(crate) fn update_section_keys<F: Read + Seek>(
    cfile: &mut CompoundFile<F>,
    tmp_buf: &mut Vec<u8>,
    header: &mut SchLibMeta,
) -> Result<(), Error> {
    if !cfile.exists(STREAMNAME) {
        return Ok(());
    }

    let mut stream = cfile.open_stream(STREAMNAME)?;
    stream.read_to_end(tmp_buf)?;

    let to_parse = tmp_buf
        .get(PFX_LEN..)
        .ok_or(ErrorKind::new_invalid_stream(STREAMNAME, 0))?
        .strip_suffix(SFX)
        .ok_or(ErrorKind::new_invalid_stream(STREAMNAME, tmp_buf.len()))?;

    // libref -> section key
    let mut map: Vec<(&str, &str)> = Vec::new();

    for (key, val) in split_altium_map(to_parse) {
        match key {
            b"KeyCount" => map = vec![Default::default(); val.parse_as_utf8()?],
            x if x.starts_with(LIBREF) => {
                let idx: usize = key[LIBREF.len()..].parse_as_utf8()?;
                map[idx].0 = val.parse_as_utf8()?;
            }
            x if x.starts_with(SECKEY) => {
                let idx: usize = key[SECKEY.len()..].parse_as_utf8()?;
                map[idx].1 = val.parse_as_utf8()?;
            }
            _ => log_unsupported_key(key, val),
        }
    }

    for comp in &mut header.components {
        // Find any keys that exist in our map
        let Ok(idx) = map.binary_search_by_key(&comp.libref(), |x| x.0) else {
            continue;
        };
        comp.sec_key = map[idx].1.into();
    }

    Ok(())
}
