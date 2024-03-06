use std::{cell::RefCell, fs::File, path::Path, io::{Cursor, Read, Seek}, fmt};
use crate::{Error, error::AddContext};
use cfb::CompoundFile;

/// This is our top-level representation of a PCB library.
pub struct PcbLib<F> {
    /// Our open compoundfile buffer
    cfile: RefCell<CompoundFile<F>>,
    // /// Information contained in the compound file header. We use this as a
    // /// lookup to see what we can extract from the file.
    // header: SchLibMeta,
    // storage: Arc<Storage>,
}

/// Impls that are specific to a file
impl PcbLib<File> {
    /// Open a file from disk
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let cfile = cfb::open(&path)?;
        Self::from_cfile(cfile).or_context(|| format!("opening {}", path.as_ref().display()))
    }
}

impl<'a> PcbLib<Cursor<&'a [u8]>> {
    /// Open an in-memory file from a buffer
    pub fn from_buffer(buf: &'a [u8]) -> Result<Self, Error> {
        let cfile = cfb::CompoundFile::open(Cursor::new(buf))?;
        Self::from_cfile(cfile)
    }
}


impl<F: Read + Seek> PcbLib<F> {
    // /// Create an iterator over all components in this library.
    // pub fn components(&self) -> ComponentsIter<'_, F> {
    //     ComponentsIter {
    //         schlib: self,
    //         current: 0,
    //     }
    // }

    // /// Create an iterator over all fonts stored in this library.
    // pub fn fonts(&self) -> impl Iterator<Item = &Font> {
    //     self.header.fonts.iter()
    // }

    // /// Get information about the blob items stored
    // pub fn storage(&self) -> &Storage {
    //     &self.storage
    // }

    /// Create a `PcbLib` representation from any `Read`able compound file.
    fn from_cfile(mut cfile: CompoundFile<F>) -> Result<Self, Error> {
        let mut tmp_buf: Vec<u8> = Vec::new(); // scratch memory

        // let mut header = SchLibMeta::parse_cfile(&mut cfile, &mut tmp_buf)?;
        // tmp_buf.clear();

        // let mut storage = Storage::parse_cfile(&mut cfile, &mut tmp_buf)?;
        // tmp_buf.clear();

        // update_section_keys(&mut cfile, &mut tmp_buf, &mut header)?;

        // section_keys.map.entry(key)
        Ok(Self {
            cfile: RefCell::new(cfile),
            // header,
            // storage: storage.into(),
        })
    }
}

impl<F> fmt::Debug for PcbLib<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PcbLib")
            // .field("header", &self.header)
            .finish_non_exhaustive()
    }
}
