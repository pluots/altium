mod section_keys;
use crate::common::{buf2lstring, split_altium_map, split_once, Color};
use crate::errors::Error;
use crate::font::Font;
use crate::parse::ParseUtf8;
use crate::sch::SheetStyle;
use cfb::CompoundFile;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, BufReader, Read, Seek};
use std::path::Path;
use std::{fmt, str};

use self::section_keys::update_section_keys;

/// Separator in textlike streams
const SEP: u8 = b'|';

/// Representation of a schematic file
pub struct SchLib<F> {
    pub cfile: CompoundFile<F>,
    header: Header,
}

impl SchLib<File> {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let mut cfile = cfb::open(path)?;
        let mut buf: Vec<u8> = Vec::new();
        let mut header = Header::parse(&mut cfile, &mut buf)?;
        buf.clear();
        update_section_keys(&mut cfile, &mut buf, &mut header)?;
        // section_keys.map.entry(key)
        Ok(Self { header, cfile })
    }

    /// Iterate through component metadata available without an exclusive lock
    pub fn components_meta(&self) -> &[ComponentMeta] {
        &self.header.components
    }

    /// Fonts used in this library
    pub fn fonts(&self) -> &[Font] {
        &self.header.fonts
    }

    /// Unique ID of this schematic library
    pub fn unique_id(&self) -> Option<&str> {
        self.header.unique_id.as_deref()
    }
}

impl<F: Read + Seek> SchLib<F> {}

impl<F> fmt::Debug for SchLib<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SchLib")
            .field("header", &self.header)
            .finish()
    }
}

/// Information contained within the `FileHeader` stream
#[derive(Clone, Debug, Default)]
pub struct Header {
    weight: u32,
    minor_version: u8,
    unique_id: Option<Box<str>>,
    fonts: Vec<Font>,
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

impl Header {
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
    fn parse<F: Read + Seek>(
        cfile: &mut CompoundFile<F>,
        buf: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        let mut stream = cfile.open_stream(Self::STREAMNAME)?;
        stream.read_to_end(buf)?;

        let to_parse = buf
            .get(Self::PFX_LEN..)
            .ok_or(Error::new_invalid_stream(Self::STREAMNAME, 0))?
            .strip_suffix(Self::SFX)
            .ok_or(Error::new_invalid_stream(Self::STREAMNAME, buf.len()))?;
        let sep_pos = to_parse
            .iter()
            .position(|b| *b == b'|')
            .unwrap_or(to_parse.len());
        if &to_parse[..sep_pos] != Self::HEADER {
            return Err(Error::new_invalid_stream(Self::STREAMNAME, Self::PFX_LEN));
        }

        let mut ret = Self::default();

        for (key, val) in split_altium_map(to_parse) {
            match key {
                Self::HEADER_KEY => continue,
                b"Weight" => ret.weight = val.parse_utf8()?,
                b"MinorVersion" => ret.minor_version = val.parse_utf8()?,
                b"UniqueID" => ret.unique_id = Some(val.parse_utf8()?),
                b"FontIdCount" => ret.fonts = vec![Font::default(); val.parse_utf8()?],
                b"UseMBCS" => ret.use_mbcs = val.parse_utf8()?,
                b"IsBOC" => ret.is_boc = val.parse_utf8()?,
                b"SheetStyle" => ret.sheet_style = val.parse_utf8()?,
                b"BorderOn" => ret.border_on = val.parse_utf8()?,
                b"SheetNumberSpaceSize" => ret.sheet_number_space_size = val.parse_utf8()?,
                b"AreaColor" => ret.area_color = val.parse_utf8()?,
                b"SnapGridOn" => ret.snap_grid_on = val.parse_utf8()?,
                b"SnapGridSize" => ret.snap_grid_size = val.parse_utf8()?,
                b"VisibleGridOn" => ret.visible_grid_on = val.parse_utf8()?,
                b"VisibleGridSize" => ret.visible_grid_size = val.parse_utf8()?,
                b"CustomX" => ret.custom_x = val.parse_utf8()?,
                b"CustomY" => ret.custom_y = val.parse_utf8()?,
                b"UseCustomSheet" => ret.use_custom_sheet = val.parse_utf8()?,
                b"ReferenceZonesOn" => ret.reference_zones_on = val.parse_utf8()?,
                b"Display_Unit" => ret.display_unit = val.parse_utf8()?,
                b"CompCount" => ret.components = vec![ComponentMeta::default(); val.parse_utf8()?],
                x if x.starts_with(Self::FONT_NAME_PFX) => {
                    let idx: usize = key[Self::FONT_NAME_PFX.len()..].parse_utf8()?;
                    ret.fonts[idx - 1].name = val.parse_utf8()?;
                }
                x if x.starts_with(Self::FONT_SIZE_PFX) => {
                    let idx: usize = key[Self::FONT_SIZE_PFX.len()..].parse_utf8()?;
                    ret.fonts[idx - 1].size = val.parse_utf8()?;
                }
                x if x.starts_with(Self::COMP_LIBREF_PFX) => {
                    let idx: usize = key[Self::COMP_LIBREF_PFX.len()..].parse_utf8()?;
                    let tmp: String = val.parse_utf8()?;
                    ret.components[idx].libref = tmp.clone();
                    ret.components[idx].sec_key = tmp;
                }
                x if x.starts_with(Self::COMP_DESC_PFX) => {
                    let idx: usize = key[Self::COMP_DESC_PFX.len()..].parse_utf8()?;
                    ret.components[idx].description = val.parse_utf8()?;
                }
                x if x.starts_with(Self::COMP_PARTCOUNT_PFX) => {
                    let idx: usize = key[Self::COMP_PARTCOUNT_PFX.len()..].parse_utf8()?;
                    ret.components[idx].part_count = val.parse_utf8()?;
                }
                _ => eprintln!("unsupported key {}:{}", buf2lstring(key), buf2lstring(val)),
            }
        }

        Ok(ret)
    }
}

// pub struct Components {}

/// Information available in the header, libref and part count
#[derive(Clone, Debug, Default)]
pub struct ComponentMeta {
    libref: String,
    sec_key: String,
    description: String,
    part_count: u16,
}

impl ComponentMeta {
    pub fn libref(&self) -> &str {
        &self.libref
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn part_countr(&self) -> u16 {
        self.part_count
    }
}

pub struct Component {
    meta: ComponentMeta,
}

impl Component {
    /// Get the metadata for this component
    pub fn meta(&self) -> &ComponentMeta {
        &self.meta
    }

    /// Parse a stream to
    fn parse(buf: &[u8]) -> Result<Self, Error> {
        todo!()
    }
}
