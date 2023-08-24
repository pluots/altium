use std::io::{Read, Seek};
use std::str;

use cfb::CompoundFile;

use super::SchLibMeta;
use crate::common::split_altium_map;
use crate::error::ErrorKind;
use crate::logging::log_unsupported_key;
use crate::parse::ParseUtf8;
use crate::Error;

const SEC_KEY_STREAM: &str = "SectionKeys";
const PFX_LEN: usize = 5;
const SFX: &[u8] = &[0x00];
const LIBREF: &[u8] = b"LibRef";
const SECKEY: &[u8] = b"SectionKey";

/// Update a header with section keys.
///
/// The `SectionKeys` stream stores a map of `libref -> section key` for some
/// librefs that are too long to be a section key (stream name) themselves. Go
/// through our extracted components and make sure that `sec_key` is replaced by
/// this map where needed.
pub(crate) fn update_section_keys<F: Read + Seek>(
    cfile: &mut CompoundFile<F>,
    tmp_buf: &mut Vec<u8>,
    header: &mut SchLibMeta,
) -> Result<(), Error> {
    if !cfile.exists(SEC_KEY_STREAM) {
        return Ok(());
    }

    let mut stream = cfile.open_stream(SEC_KEY_STREAM)?;
    stream.read_to_end(tmp_buf)?;

    let to_parse = tmp_buf
        .get(PFX_LEN..)
        .ok_or(ErrorKind::new_invalid_stream(SEC_KEY_STREAM, 0))?
        .strip_suffix(SFX)
        .ok_or(ErrorKind::new_invalid_stream(SEC_KEY_STREAM, tmp_buf.len()))?;

    // keep a map of libref -> section key
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
        // Find any keys that exist in our map and replace them
        let Ok(idx) = map.binary_search_by_key(&comp.libref(), |x| x.0) else {
            // If they aren't in our map, fixup only
            comp.sec_key = fixup_sec_key(&comp.sec_key);
            continue;
        };

        comp.sec_key = fixup_sec_key(map[idx].1);
    }

    Ok(())
}

/// Altium does some transformations for its stream paths, e.g. `/` -> `_`
fn fixup_sec_key(path: &str) -> Box<str> {
    path.replace('/', "_").into()
}
