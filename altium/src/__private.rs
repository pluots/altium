//! Things only used for testing

use std::sync::atomic::Ordering;

use crate::{common::buf2lstr, logging::UNSUPPORTED_KEYS};

pub fn num_unsupported_keys() -> u32 {
    UNSUPPORTED_KEYS.load(Ordering::Relaxed)
}
