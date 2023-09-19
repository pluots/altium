use super::SchRecord;
use crate::error::AddContext;
use crate::{
    sch::{pin::SchPin, record::parse_any_record},
    Error,
};

/// Given a buffer for a component, split the records up
///
/// Name is only used for diagnostics
pub fn parse_all_records(buf: &[u8], err_name: &str) -> Result<Vec<SchRecord>, Error> {
    // Our info u32 is something like `0xttllllll`, where `tt` are 8 bits
    // representing a type (currently only values 0 and 1 known) and the `l`s
    // are the length
    const TY_SHIFT: u32 = 24;
    const TY_MAEK: u32 = 0xff000000;
    const LEN_MASK: u32 = 0x00ffffff;
    const UTF8_RECORD_TY: u32 = 0x00;
    const PIN_RECORD_TY: u32 = 0x01;
    // No magic values :)
    const U32_BYTES: usize = 4;

    let mut working = buf;
    let mut parsed = Vec::new();
    while !working.is_empty() {
        assert!(
            working.len() >= 4,
            "expected at least 4 bytes, only got {}",
            working.len()
        );

        let info = u32::from_le_bytes(working[..4].try_into().unwrap());
        let ty = (info & TY_MAEK) >> TY_SHIFT;
        let len: usize = (info & LEN_MASK).try_into().unwrap();

        // Don't include the null terminator (which is included in `len`)
        let to_parse = &working[U32_BYTES..(U32_BYTES + len - 1)];

        // But do do a sanity check that the null exists
        assert_eq!(working[U32_BYTES + len - 1], 0, "Expected null terimation");

        working = &working[U32_BYTES + len..];

        let record = match ty {
            UTF8_RECORD_TY => parse_any_record(to_parse),
            PIN_RECORD_TY => SchPin::parse(to_parse).map_err(Into::into),
            _ => panic!("unexpected record type {ty:02x}"),
        };

        parsed.push(record.or_context(|| format!("in `parse_all_records` for `{err_name}`"))?);
    }

    Ok(parsed)
}
