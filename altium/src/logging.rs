use std::sync::atomic::{AtomicU32, Ordering};

use crate::common::buf2lstring;

/// Track how many unsupported keys we have, useful for testing
pub static UNSUPPORTED_KEYS: AtomicU32 = AtomicU32::new(0);

/// Log the unsupported key
pub fn log_unsupported_key(key: &[u8], val: &[u8]) {
    UNSUPPORTED_KEYS.fetch_add(1, Ordering::Relaxed);
    eprintln!("unsupported key {}={}", buf2lstring(key), buf2lstring(val));
}
