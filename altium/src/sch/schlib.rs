mod section_keys;

use std::cell::RefCell;
use std::fs::File;
use std::io::{Cursor, Read, Seek};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{fmt, str};

use cfb::CompoundFile;
use section_keys::update_section_keys;

use crate::common::{buf2lstring, split_altium_map, Color, UniqueId};
use crate::error::{AddContext, ErrorKind};
use crate::font::{Font, FontCollection};
use crate::parse::ParseUtf8;
use crate::sch::{storage::Storage, Component, SheetStyle};
use crate::Error;

/// Reasonable size for many pins
const DATA_DEFAULT_CAP: usize = 200;

/// This is our top-level representation of a schematic library.
pub struct SchLib<F> {
    /// Our open compoundfile buffer
    cfile: RefCell<CompoundFile<F>>,
    /// Information contained in the compound file header. We use this as a
    /// lookup to see what we can extract from the file.
    header: SchLibMeta,
    storage: Arc<Storage>,
}

/// Impls that are specific to a file
impl SchLib<File> {
    /// Open a file from disk
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let cfile = cfb::open(&path)?;
        Self::from_cfile(cfile).or_context(|| format!("opening {}", path.as_ref().display()))
    }
}

impl<'a> SchLib<Cursor<&'a [u8]>> {
    /// Open an in-memory file from a buffer
    pub fn from_buffer(buf: &'a [u8]) -> Result<Self, Error> {
        let cfile = cfb::CompoundFile::open(Cursor::new(buf))?;
        Self::from_cfile(cfile)
    }
}

impl<F: Read + Seek> SchLib<F> {
    /// Unique ID of this schematic library
    fn unique_id(&self) -> UniqueId {
        self.header.unique_id
    }

    /// Get information about this file in general. Use this if you want to get
    /// the libref or description of a component, or check what components
    /// exist.
    pub fn component_meta(&self) -> &[ComponentMeta] {
        &self.header.components
    }

    /// Lookup a single component by its libref
    ///
    /// # Panics
    ///
    /// Panics if there are any failures reading the component. This shouldn't happen with files
    /// that Altium generates.
    pub fn get_component(&self, libref: &str) -> Option<Component> {
        self.try_get_component(libref).unwrap()
    }

    /// Lookup a single component by its libref, propegating errors if they arise
    fn try_get_component(&self, libref: &str) -> Result<Option<Component>, Error> {
        let Some(meta) = &self
            .header
            .components
            .iter()
            .find(|meta| &*meta.libref == libref)
        else {
            return Ok(None);
        };

        let key = &meta.sec_key;

        // Data is required. TBD what "PinTextData" and "PinWideText" contain.
        let data_path = PathBuf::from_iter([key, "Data"]);
        let _pintext_path = PathBuf::from_iter([key, "PinTextData"]);
        let _pinwide_path = PathBuf::from_iter([key, "PinWideText"]);

        let mut buf = Vec::with_capacity(DATA_DEFAULT_CAP);

        {
            // Scope of refcell borrow
            let mut cfile_ref = self.cfile.borrow_mut();
            let mut stream = cfile_ref.open_stream(&data_path).unwrap_or_else(|e| {
                panic!(
                    "missing required stream `{}` with error {e}",
                    data_path.display()
                )
            });
            stream.read_to_end(&mut buf).unwrap();
        }

        let comp = Component::from_buf(
            libref,
            &buf,
            Arc::clone(&self.header.fonts),
            Arc::clone(&self.storage),
        )?;

        Ok(Some(comp))
    }

    /// Create an iterator over all components in this library.
    pub fn components(&self) -> ComponentsIter<'_, F> {
        ComponentsIter {
            schlib: self,
            current: 0,
        }
    }

    /// Create an iterator over all fonts stored in this library.
    pub fn fonts(&self) -> impl Iterator<Item = &Font> {
        self.header.fonts.iter()
    }

    /// Get information about the blob items stored
    pub fn storage(&self) -> &Storage {
        &self.storage
    }

    /// Create a `SchLib` representation from any `Read`able compound file.
    fn from_cfile(mut cfile: CompoundFile<F>) -> Result<Self, Error> {
        let mut tmp_buf: Vec<u8> = Vec::new(); // scratch memory

        let mut header = SchLibMeta::parse_cfile(&mut cfile, &mut tmp_buf)?;
        tmp_buf.clear();

        let mut storage = Storage::parse_cfile(&mut cfile, &mut tmp_buf)?;
        tmp_buf.clear();

        update_section_keys(&mut cfile, &mut tmp_buf, &mut header)?;

        // section_keys.map.entry(key)
        Ok(Self {
            cfile: RefCell::new(cfile),
            header,
            storage: storage.into(),
        })
    }
}

impl<F> fmt::Debug for SchLib<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SchLib")
            .field("header", &self.header)
            .finish_non_exhaustive()
    }
}

/// Iterator over components in a library
pub struct ComponentsIter<'a, F> {
    schlib: &'a SchLib<F>,
    current: usize,
}

impl<'a, F: Read + Seek> Iterator for ComponentsIter<'a, F> {
    type Item = Component;

    fn next(&mut self) -> Option<Self::Item> {
        let meta = self.schlib.component_meta();
        if self.current >= meta.len() {
            None
        } else {
            let libref = meta[self.current].libref();
            let ret = self
                .schlib
                .try_get_component(libref)
                .expect("component should exist!");
            self.current += 1;
            // We assume that there are no errors
            Some(ret.unwrap())
        }
    }
}

/// Information contained within the `FileHeader` stream. These are things we
/// can look up directly, without needing to page the entire file.
#[derive(Clone, Debug, Default)]
pub(crate) struct SchLibMeta {
    weight: u32,
    minor_version: u8,
    /// Unique id of this schlib
    unique_id: UniqueId,
    /// Table of fonts found in the header
    ///
    /// This is an `Arc` because we want other types to have an easy way to
    /// share this information.
    fonts: Arc<FontCollection>,
    ///
    use_mbcs: bool,
    is_boc: bool,
    sheet_style: SheetStyle,
    border_on: bool,
    sheet_number_space_size: u16,
    area_color: Color,
    snap_grid_on: bool,
    snap_grid_size: u16,
    visible_grid_on: bool,
    visible_grid_size: u16,
    custom_x: u32,
    custom_y: u32,
    use_custom_sheet: bool,
    reference_zones_on: bool,
    display_unit: u16, // FIXME: enum
    components: Vec<ComponentMeta>,
}

/// Parse implementation
impl SchLibMeta {
    const STREAMNAME: &str = "FileHeader";

    /// Magic header found in all streams
    const HEADER: &[u8] =
        b"HEADER=Protel for Windows - Schematic Library Editor Binary File Version 5.0";
    const HEADER_KEY: &[u8] = b"HEADER";

    // /// Every header starts with this
    // const PFX: &[u8] = &[0x7a, 0x04, 0x00, 0x00, b'|'];
    // Seems like each stream starts with 4 random bytes followed by a `|`?
    const PFX_LEN: usize = 5;
    const SFX: &[u8] = &[0x00];

    /* font-related items */
    /// `FontName1=Times New Roman`
    const FONT_NAME_PFX: &[u8] = b"FontName";
    /// `Size1=9`
    const FONT_SIZE_PFX: &[u8] = b"Size";

    /* part-related items */
    /// `Libref0=Part Name`
    const COMP_LIBREF_PFX: &[u8] = b"LibRef";
    /// `CompDescr0=Long description of thing Name`
    const COMP_DESC_PFX: &[u8] = b"CompDescr";
    /// `PartCount0=2` number of subcomponents (seems to default to 2?)
    const COMP_PARTCOUNT_PFX: &[u8] = b"PartCount";

    /// Validate a `FileHeader` and extract its information
    ///
    /// `buf` should be empty, we just reuse it to avoid reallocation
    fn parse_cfile<F: Read + Seek>(
        cfile: &mut CompoundFile<F>,
        tmp_buf: &mut Vec<u8>,
    ) -> Result<Self, ErrorKind> {
        let mut stream = cfile.open_stream(Self::STREAMNAME)?;
        stream.read_to_end(tmp_buf)?;

        println!("parsing cfile:\n{}", String::from_utf8_lossy(tmp_buf));

        let to_parse = tmp_buf
            .get(Self::PFX_LEN..)
            .ok_or(ErrorKind::new_invalid_stream(Self::STREAMNAME, 0))?
            .strip_suffix(Self::SFX)
            .ok_or(ErrorKind::new_invalid_stream(
                Self::STREAMNAME,
                tmp_buf.len(),
            ))?;

        let sep_pos = to_parse
            .iter()
            .position(|b| *b == b'|')
            .unwrap_or(to_parse.len());
        if &to_parse[..sep_pos] != Self::HEADER {
            return Err(ErrorKind::new_invalid_stream(
                Self::STREAMNAME,
                Self::PFX_LEN,
            ));
        }

        let mut skip_keys = Vec::new();
        let mut ret = Self::default();
        let mut fonts = Vec::new();

        // Iterate through each key. Based on its type, parse a value.
        for (mut key, val) in split_altium_map(to_parse) {
            // Altium does something where it will store a UTF8 version of a key
            // preceded by `%UTF8%` and a non-UTF8 version without it. Maybe for
            // backward compat? We just take the UTF8 version, since the
            // non-UTF8 version seems garbage (e.g. contains just 0xb0 for `°`
            // rather than 0xc2 0xb0. Maybe it's truncated utf16?)
            if key.starts_with(b"%UTF8%") {
                key = &key[6..];
                skip_keys.push(key);
            } else if skip_keys.contains(&key) {
                continue;
            }

            match key {
                Self::HEADER_KEY => continue,
                b"Weight" => ret.weight = val.parse_as_utf8()?,
                b"MinorVersion" => ret.minor_version = val.parse_as_utf8()?,
                b"UniqueID" => ret.unique_id = val.parse_as_utf8()?,
                b"FontIdCount" => fonts = vec![Font::default(); val.parse_as_utf8()?],
                b"UseMBCS" => ret.use_mbcs = val.parse_as_utf8()?,
                b"IsBOC" => ret.is_boc = val.parse_as_utf8()?,
                b"SheetStyle" => ret.sheet_style = val.parse_as_utf8()?,
                b"BorderOn" => ret.border_on = val.parse_as_utf8()?,
                b"SheetNumberSpaceSize" => ret.sheet_number_space_size = val.parse_as_utf8()?,
                b"AreaColor" => ret.area_color = val.parse_as_utf8()?,
                b"SnapGridOn" => ret.snap_grid_on = val.parse_as_utf8()?,
                b"SnapGridSize" => ret.snap_grid_size = val.parse_as_utf8()?,
                b"VisibleGridOn" => ret.visible_grid_on = val.parse_as_utf8()?,
                b"VisibleGridSize" => ret.visible_grid_size = val.parse_as_utf8()?,
                b"CustomX" => ret.custom_x = val.parse_as_utf8()?,
                b"CustomY" => ret.custom_y = val.parse_as_utf8()?,
                b"UseCustomSheet" => ret.use_custom_sheet = val.parse_as_utf8()?,
                b"ReferenceZonesOn" => ret.reference_zones_on = val.parse_as_utf8()?,
                b"Display_Unit" => ret.display_unit = val.parse_as_utf8()?,
                b"CompCount" => {
                    ret.components = vec![ComponentMeta::default(); val.parse_as_utf8()?];
                }
                x if x.starts_with(Self::FONT_NAME_PFX) => {
                    let idx: usize = key[Self::FONT_NAME_PFX.len()..].parse_as_utf8()?;
                    fonts[idx - 1].name = val.parse_as_utf8()?;
                }
                x if x.starts_with(Self::FONT_SIZE_PFX) => {
                    let idx: usize = key[Self::FONT_SIZE_PFX.len()..].parse_as_utf8()?;
                    fonts[idx - 1].size = val.parse_as_utf8()?;
                }
                x if x.starts_with(Self::COMP_LIBREF_PFX) => {
                    let idx: usize = key[Self::COMP_LIBREF_PFX.len()..].parse_as_utf8()?;
                    let tmp: Box<str> = val.parse_as_utf8()?;
                    ret.components[idx].libref = tmp.clone();
                    ret.components[idx].sec_key = tmp;
                }
                x if x.starts_with(Self::COMP_DESC_PFX) => {
                    let idx: usize = key[Self::COMP_DESC_PFX.len()..].parse_as_utf8()?;
                    ret.components[idx].description = val.parse_as_utf8()?;
                }
                x if x.starts_with(Self::COMP_PARTCOUNT_PFX) => {
                    let idx: usize = key[Self::COMP_PARTCOUNT_PFX.len()..].parse_as_utf8()?;
                    ret.components[idx].part_count = val.parse_as_utf8()?;
                }
                _ => eprintln!(
                    "unsupported file header key {}:{}",
                    buf2lstring(key),
                    buf2lstring(val)
                ),
            }
        }

        ret.fonts = Arc::new(fonts.into());
        println!("done parsing cfile");
        Ok(ret)
    }
}

// pub struct Components {}

/// Information available in the header about a single component: includes
/// libref and part count
#[derive(Clone, Debug, Default)]
pub struct ComponentMeta {
    /// Name of the thing in Altium
    libref: Box<str>,
    /// Name of the thing in our OLE file
    sec_key: Box<str>,
    /// Description
    description: Box<str>,
    // FIXME: what is this?
    part_count: u16,
}

impl ComponentMeta {
    /// Library reference of this type
    pub fn libref(&self) -> &str {
        &self.libref
    }

    /// This component's description
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Number of subparts within a component
    ///
    /// FIXME: this seems to be doubled?
    fn part_count(&self) -> u16 {
        self.part_count
    }
}
