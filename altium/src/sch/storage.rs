//! Representations of things in the `Storage` file. Usually these are images
//! represented as zlib-compressed data.
//!
//! Our goal is that these blobs have one representation in memory. Once
//! created, we keep it in memory and share it.

use std::{collections::BTreeMap, sync::Arc};

#[derive(Clone, Debug, Default)]
pub struct Storage(BTreeMap<Box<str>, Option<Arc<[u8]>>>);

impl Storage {}
