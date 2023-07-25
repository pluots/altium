#![allow(unused)]

use std::fs::File;
use std::io::{self, Read, Seek};
use std::path::Path;

use cfb::CompoundFile;

use crate::Error;

/// Magic string found in the `FileHeader` stream
const HEADER: &str = "HEADER=Protel for Windows - Schematic Library Editor Binary File Version 5.0";

/// Representation of a schematic file
pub struct SchDoc<F> {
    cfile: CompoundFile<F>,
}

impl SchDoc<File> {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let cfile = cfb::open(path)?;

        Ok(Self { cfile })
    }
}

impl<F: Read + Seek> SchDoc<F> {}