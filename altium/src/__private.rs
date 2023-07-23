//! Things only used for testing

use std::sync::atomic::Ordering;

use crate::{common::buf2lstring, logging::UNSUPPORTED_KEYS};

pub fn num_unsupported_keys() -> u32 {
    UNSUPPORTED_KEYS.load(Ordering::Relaxed)
}

/// Called from our proc macro to log a key
pub fn macro_unsupported_key(name: &str, key: &[u8], val: &[u8]) {
    UNSUPPORTED_KEYS.fetch_add(1, Ordering::Relaxed);
    eprintln!(
        "unsupported key for `{name}`: {}={} (via `FromRecord` derive)",
        buf2lstring(key),
        buf2lstring(val)
    );
}
