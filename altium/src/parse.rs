//! Traits to help with parsing
mod bin;
mod from_record;
mod utf8;

pub use bin::{extract_sized_buf, extract_sized_utf8_buf, BufLenMatch};
pub use from_record::FromRecord;
pub use utf8::{FromUtf8, ParseUtf8};
