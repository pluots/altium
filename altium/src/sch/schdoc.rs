// #![allow(unused)]

use core::fmt;
use std::cell::RefCell;
use std::fs::File;
use std::io::{Cursor, Read, Seek};
use std::path::Path;
use std::sync::Arc;

use cfb::CompoundFile;

use super::record::{parse_all_records, Sheet};
use super::storage::Storage;
use super::{SchDrawCtx, SchRecord};
use crate::common::split_altium_map;
use crate::draw::{Canvas, Draw};
use crate::error::AddContext;
use crate::parse::{extract_sized_buf, BufLenMatch, ParseUtf8};
use crate::{Error, ErrorKind, UniqueId};

/// Magic string found in the `FileHeader` stream
const HEADER: &str = "Protel for Windows - Schematic Capture Binary File Version 5.0";
/// Where most content is stored
const DATA_STREAM: &str = "FileHeader";

/// Representation of a schematic file
pub struct SchDoc<F> {
    #[allow(dead_code)]
    cfile: RefCell<CompoundFile<F>>,
    sheet: Sheet,
    records: Vec<SchRecord>,
    unique_id: UniqueId,
    storage: Arc<Storage>,
}

/// Impls that are specific to a file
impl SchDoc<File> {
    /// Open a file from disk
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let cfile = cfb::open(&path)?;
        Self::from_cfile(cfile)
            .context("parsing SchLib")
            .or_context(|| format!("with file {}", path.as_ref().display()))
    }
}

impl<'a> SchDoc<Cursor<&'a [u8]>> {
    /// Open an in-memory file from a buffer
    pub fn from_buffer(buf: &'a [u8]) -> Result<Self, Error> {
        let cfile = cfb::CompoundFile::open(Cursor::new(buf))?;
        Self::from_cfile(cfile).context("parsing SchDoc from Cursor")
    }
}

impl<F: Read + Seek> SchDoc<F> {
    /// Iterate over all records
    pub fn records(&self) -> impl Iterator<Item = &SchRecord> {
        // FIXME: iterator over the sheet too
        self.records.iter()
    }

    /// Draw this schematic document
    pub fn draw<C: Canvas>(&self, canvas: &mut C) {
        let ctx = SchDrawCtx {
            storage: &self.storage,
            fonts: &self.sheet.fonts,
        };
        self.records().for_each(|r| r.draw(canvas, &ctx));
    }

    /// Create a `SchLib` representation from any `Read`able compound file.
    fn from_cfile(mut cfile: CompoundFile<F>) -> Result<Self, Error> {
        let mut tmp_buf: Vec<u8> = Vec::new(); // scratch memory

        let storage = Storage::parse_cfile(&mut cfile, &mut tmp_buf)?;
        tmp_buf.clear();

        {
            let mut stream = cfile.open_stream(DATA_STREAM).map_err(|e| {
                Error::from(e).context(format!("reading required stream `{DATA_STREAM}`"))
            })?;
            stream.read_to_end(&mut tmp_buf).unwrap();
        }

        let (rest, unique_id) = parse_header(&tmp_buf)?;
        let mut records = parse_all_records(rest, "SchDoc::from_cfile")?;
        let sheet_pos = records
            .iter()
            .position(|x| matches!(x, SchRecord::Sheet(_)));
        let sheet = sheet_pos
            .map(|idx| {
                let SchRecord::Sheet(sheet) = records.remove(idx) else {
                    unreachable!()
                };
                sheet
            })
            .unwrap_or_default();

        Ok(Self {
            cfile: RefCell::new(cfile),
            records,
            sheet,
            storage: storage.into(),
            unique_id,
        })
    }
}

impl<F> fmt::Debug for SchDoc<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SchDoc")
            .field("unique_id", &self.unique_id)
            .finish_non_exhaustive()
    }
}

/// Extract the header, return the residual and the document unique ID
fn parse_header(buf: &[u8]) -> Result<(&[u8], UniqueId), Error> {
    let mut uid = None;
    let (hdr, rest) = extract_sized_buf(buf, BufLenMatch::U32, true)?;
    for (key, val) in split_altium_map(hdr) {
        match key {
            b"HEADER" => {
                if val != HEADER.as_bytes() {
                    return Err(ErrorKind::new_invalid_header(val, HEADER).into());
                }
            }
            b"UniqueID" => uid = Some(val.parse_as_utf8()?),
            _ => (),
        }
    }

    let uid = uid.ok_or(ErrorKind::MissingUniqueId)?;

    Ok((rest, uid))
}
