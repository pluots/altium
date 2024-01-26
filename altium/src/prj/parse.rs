use std::borrow::ToOwned;

use ini::Properties;

use crate::{
    common::UniqueId,
    error::{AddContext, ErrorKind},
    parse::FromUtf8,
    Error,
};

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
    match sec.get(key) {
        Some("1") => true,
        None | Some(_) => false,
    }
}

/// Extract a `UniqueId` from a buffer
pub fn parse_unique_id(sec: &Properties, key: &str) -> Result<UniqueId, Error> {
    sec.get(key)
        .ok_or_else(|| ErrorKind::MissingSection(key.to_owned()))
        .and_then(|v| UniqueId::from_utf8(v.as_bytes()))
        .map_err(|e| e.context("parse_unique_id"))
}
