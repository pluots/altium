use ini::Properties;
use std::borrow::ToOwned;

use crate::{common::UniqueId, errors::ErrorKind};

/// Parse a string or default to an empty string
pub fn parse_string(sec: &Properties, key: &str) -> String {
    sec.get(key).map(ToOwned::to_owned).unwrap_or_default()
}

/// Parse an integer or default to 0
pub fn parse_int(sec: &Properties, key: &str) -> i32 {
    sec.get(key)
        .and_then(|v| v.parse().ok())
        .unwrap_or_default()
}

/// Parse a boolean value
pub fn parse_bool(sec: &Properties, key: &str) -> bool {
    sec.get(key).map(|v| v == "1").unwrap_or_default()
}

/// Extract a `UniqueId` from a buffer
pub fn parse_unique_id(sec: &Properties, key: &str) -> Result<UniqueId, ErrorKind> {
    sec.get(key)
        .ok_or_else(|| ErrorKind::MissingSection(key.to_owned()))
        .map(|v| UniqueId::from_slice(v).ok_or(ErrorKind::MissingUniqueId(v.to_owned())))?
}
