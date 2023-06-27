//! Traits to help with parsing
mod from_record;
mod utf8;

pub use from_record::FromRecord;
pub use utf8::{FromUtf8, ParseUtf8};
