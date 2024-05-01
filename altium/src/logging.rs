use std::sync::atomic::{AtomicU32, Ordering};

use crate::common::buf2lstr;

/// Track how many unsupported keys we have, useful for testing
pub static UNSUPPORTED_KEYS: AtomicU32 = AtomicU32::new(0);

/// Log the unsupported key
pub fn log_unsupported_key(key: &[u8], val: &[u8]) {
    UNSUPPORTED_KEYS.fetch_add(1, Ordering::Relaxed);
    log::warn!("unsupported key {}={}", buf2lstr(key), buf2lstr(val));
}

/// Called from our proc macro to log a key
pub fn macro_unsupported_key(name: &str, key: &[u8], val: &[u8]) {
    UNSUPPORTED_KEYS.fetch_add(1, Ordering::Relaxed);
    log::warn!(
        "unsupported key for `{name}`: {}={} (via `FromRecord` derive)",
        buf2lstr(key),
        buf2lstr(val)
    );
}
