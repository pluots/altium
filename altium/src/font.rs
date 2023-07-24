//! Objects related to font as Altium sees it.

use std::ops::Deref;

/// A font that is stored in a library
#[derive(Clone, Debug, Default)]
pub struct Font {
    pub(crate) name: Box<str>,
    pub(crate) size: u16,
}

impl Font {
    /// The name of this font
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Font size (points)
    pub fn size(&self) -> u16 {
        self.size
    }
}

/// A set of fonts
///
// We might want to change this to something like `BTreeMap<u16, Arc<Font>>`.
// More `Arc`s exist, but in exchange we get to update them in the future.
//
// Or `Arc<RwLock<BTreeMap<u16, Arc<Font>>>>`. Yucky, but editable (edit the
// font if you're the only user duplicate it if you're not)
#[derive(Clone, Debug, Default)]
pub struct FontCollection(pub(crate) Vec<Font>);

impl Deref for FontCollection {
    type Target = [Font];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Vec<Font>> for FontCollection {
    fn from(value: Vec<Font>) -> Self {
        Self(value)
    }
}
