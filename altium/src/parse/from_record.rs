use crate::{sch::SchRecord, Error};

/// Given a record with an ID, parse the items
pub trait FromRecord {
    const RECORD_ID: u32;

    fn from_record<'a, I: Iterator<Item = (&'a [u8], &'a [u8])>>(
        records: I,
    ) -> Result<SchRecord, Error>;
}
