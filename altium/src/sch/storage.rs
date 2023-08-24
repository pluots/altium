//! Representations of things in the `Storage` file. Usually these are images
//! represented as zlib-compressed data.

use core::fmt;
use std::ffi::OsStr;
use std::fmt::LowerHex;
use std::io::{self, Cursor, Read, Seek, Write};
use std::sync::Mutex;
use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

use cfb::CompoundFile;
use flate2::read::ZlibDecoder;

use crate::common::split_altium_map;
use crate::error::{AddContext, TruncBuf};
use crate::parse::{extract_sized_buf, extract_sized_utf8_buf, BufLenMatch};
use crate::{Error, ErrorKind};

/// The `Storage` stream maps between path-like strings and zlib-compressed
/// data.
///
/// We try to avoid:
///
/// 1. Decompressing anything we don't have to, and
/// 2. Decompressing anything more than once
///
/// So, we have a solution where:
///
/// 1. Anybody who needs data can find the mutex via the btree
/// 2. Locking the mutex is only required to check if the data has been
///    inflated, and to inflate it for the first time
/// 3. Once data has been inflated, it is placed in an `Arc` and the original
///    data is freed
/// 4. Anybody else who needs this data will just get the `Arc` pointer for free
///
/// Seems pretty elegant
#[derive(Debug, Default)]
pub struct Storage(BTreeMap<Box<str>, Mutex<CompressedData>>);

/// Contain data that may or may not be compressed
#[derive(Clone)]
pub enum CompressedData {
    Compressed(Box<[u8]>),
    Expanded(Arc<[u8]>),
}

impl Storage {
    const STREAMNAME: &'static str = "Storage";

    /// Get the data from a key (path) name if available
    ///
    /// # Panics
    ///
    /// Panics if the data is not well formed (shouldn't happen when reading
    /// Altium-created files)
    pub fn get_data(&self, path: &str) -> Option<Arc<[u8]>> {
        self.try_get_data(path).map(Result::unwrap)
    }

    /// Get the data with a certain path name if available
    pub fn try_get_data(&self, path: &str) -> Option<Result<Arc<[u8]>, Error>> {
        let Some(mtx) = self.0.get(path) else {
            return None;
        };

        let data_res = (*mtx.lock().unwrap()).uncompressed();

        Some(data_res.or_context(|| format!("accessing data for '{path}'")))
    }

    /// Objects in this storage
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.0.keys().map(AsRef::as_ref)
    }

    pub(crate) fn parse_cfile<F: Read + Seek>(
        cfile: &mut CompoundFile<F>,
        tmp_buf: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        let mut stream = cfile.open_stream(Self::STREAMNAME)?;
        // FIXME: we could use a bufreader or something here to make this more
        // efficient
        stream.read_to_end(tmp_buf)?;
        Self::parse(tmp_buf)
    }

    pub(crate) fn parse(buf: &[u8]) -> Result<Self, Error> {
        let (mut header, mut rest) =
            extract_sized_buf(buf, BufLenMatch::U32).context("parsing storage")?;

        assert_eq!(
            header.last(),
            Some(&0),
            "expected null termination at {:02x}",
            TruncBuf::new_end(header)
        );
        header = &header[..header.len().saturating_sub(1)];

        let mut map_kv = split_altium_map(header);
        let Some((b"HEADER", b"Icon storage")) = map_kv.next() else {
            return Err(ErrorKind::new_invalid_header(header).context("parsing storage"));
        };

        // Weight indicates how many items are in the storage
        let Some((b"Weight", weight_val)) = map_kv.next() else {
            assert!(
                rest.is_empty(),
                "weight not present but rest was not empty at {}",
                TruncBuf::new(rest)
            );
            return Ok(Self::default());
        };

        let mut map = BTreeMap::new();
        let mut path;
        let mut data;

        while !rest.is_empty() {
            // We can just discard the data length (first three bytes) since we
            // get it again
            // 0x01 and 0xd0 are magic
            let Some([_, _, _, 0x01, 0xd0]) = rest.get(..5) else {
                return Err(ErrorKind::InvalidStorageData(rest.into()).context("parsing storage"));
            };

            rest = &rest[5..];

            // Path comes first, then data
            (path, rest) = extract_sized_utf8_buf(rest, BufLenMatch::U8)?;
            (data, rest) = extract_sized_buf(rest, BufLenMatch::U32)?;

            map.insert(
                path.into(),
                Mutex::new(CompressedData::Compressed(data.into())),
            );
        }

        Ok(Self(map))
    }
}

impl CompressedData {
    /// If the data is compressed, uncompress it first. Once uncompressed,
    /// return a pointer to that data.
    fn uncompressed(&mut self) -> Result<Arc<[u8]>, ErrorKind> {
        let compressed = match self {
            Self::Compressed(d) => d,
            Self::Expanded(arc) => return Ok(Arc::clone(arc)),
        };

        // ZLib can be about 2:1 - 5:1
        let mut tmp_buf = Vec::with_capacity(compressed.len() * 2);

        let mut z = ZlibDecoder::new(&**compressed);
        z.read_to_end(&mut tmp_buf)?;

        // FIXME: Altium seems to store PNG images with a white background?
        // set it to transparent here
        let img = image::load_from_memory(&tmp_buf)?;
        tmp_buf.clear();

        let mut img = img.into_rgba8();
        img.pixels_mut()
            .filter(|px| px[0] == 255 && px[1] == 255 && px[2] == 255)
            .for_each(|px| px[3] = 0);

        img.write_to(
            &mut Cursor::new(&mut tmp_buf),
            image::ImageOutputFormat::Png,
        )?;

        let arc: Arc<[u8]> = tmp_buf.into();
        *self = Self::Expanded(Arc::clone(&arc));

        Ok(arc)
    }
}

impl fmt::Debug for CompressedData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (name, data): (_, &[u8]) = match self {
            CompressedData::Compressed(v) => ("Compressed", v),
            CompressedData::Expanded(v) => ("Expanded", v),
        };

        write!(f, "{name}({:02x})", &TruncBuf::new(data))
    }
}

/// Windows paths are stored by Altium, this lets us get the file name of either
/// kind.
pub fn file_name(path: &str) -> &str {
    let is_sep = |ch: char| ch == '\\' || ch == '/';
    let rpos = path.rfind(is_sep).unwrap_or(path.len());
    path[(rpos)..].trim_start_matches(is_sep)
}
