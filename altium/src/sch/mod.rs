mod params;
mod pin;
pub mod record;

use std::fs::File;
use std::io::{self, Read, Seek};
use std::path::Path;

use cfb::CompoundFile;

use crate::errors::Error;

pub use params::SheetStyle;
pub use record::SchRecord;

/// Magic string found in the `FileHeader` stream
const HEADER: &str = "HEADER=Protel for Windows - Schematic Library Editor Binary File Version 5.0";

/// Representation of a schematic file
pub struct Schematic<F> {
    cfile: CompoundFile<F>,
}

impl Schematic<File> {
    fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let cfile = cfb::open(path)?;

        Ok(Self { cfile })
    }
}

impl<F: Read + Seek> Schematic<F> {}
