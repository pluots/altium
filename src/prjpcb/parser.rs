use ini::Properties;
use std::borrow::ToOwned;

use crate::{common::UniqueId, errors::Error};

pub fn parse_string(sec: &Properties, key: &str) -> String {
    sec.get(key).map(ToOwned::to_owned).unwrap_or_default()
}

pub fn parse_int(sec: &Properties, key: &str) -> i32 {
    sec.get(key)
        .map(|v| v.parse().ok())
        .flatten()
        .unwrap_or_default()
}

pub fn parse_bool(sec: &Properties, key: &str) -> bool {
    sec.get(key).map(|v| v == "1").unwrap_or_default()
}

pub fn parse_unique_id(sec: &Properties, key: &str) -> Result<UniqueId, Error> {
    sec.get(key)
        .ok_or_else(|| Error::MissingSection(key.to_owned()))
        .map(|v| UniqueId::from_slice(v).ok_or(Error::UniqueId(v.to_owned())))?
}
