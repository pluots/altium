//! Trait for a common way to extract records

use std::fmt::Write;

use crate::{common::split_altium_map, error::AddContext, sch::SchRecord, Error};

/// Given a record with an ID, parse the items
///
/// See the [`crate::sch::record`] module for more information.
pub trait FromRecord {
    const RECORD_ID: u32;

    /// Parse from a list of records
    fn from_record<'a, I: Iterator<Item = (&'a [u8], &'a [u8])>>(
        records: I,
    ) -> Result<SchRecord, Error>;

    /// Parse this record if `RECORD_ID` matches. Otherwise, return `None`
    fn parse_if_matches(record_id: u32, buf: &[u8]) -> Option<Result<SchRecord, Error>> {
        if record_id == Self::RECORD_ID {
            let ret = Self::from_record(split_altium_map(buf))
                .or_context(|| format!("with record id: {}", Self::RECORD_ID));
            Some(ret)
        } else {
            None
        }
    }
}
