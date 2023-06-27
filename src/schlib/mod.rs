use cfb::CompoundFile;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, BufReader, Read, Seek};
use std::path::Path;
use std::str;

use crate::common::{buf2string, buf2string_lossy, parse_utf8, split_altium_map, split_once};
use crate::errors::Error;
use crate::font::Font;

/// Separator in textlike streams
const SEP: u8 = b'|';

/// Representation of a schematic file
pub struct SchLib<F> {
    pub cfile: CompoundFile<F>,
    header: Header,
    section_keys: SectionKeys,
}

impl SchLib<File> {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let mut cfile = cfb::open(path)?;
        let mut buf: Vec<u8> = Vec::new();
        let header = Header::parse(&mut cfile, &mut buf)?;
        buf.clear();
        let section_keys = SectionKeys::parse(&mut cfile, &mut buf)?;
        Ok(Self {
            header,
            cfile,
            section_keys,
        })
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
    pub fn unique_id(&self) -> &str {
        &self.header.uniqe_id
    }
}

impl<F: Read + Seek> SchLib<F> {}

/// Information contained within the `FileHeader` stream
#[derive(Clone, Debug)]
pub struct Header {
    fonts: Vec<Font>,
    components: Vec<ComponentMeta>,
    uniqe_id: Box<str>,
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
    const PXF_LEN: usize = 5;
    const SFX: &[u8] = &[0x00];

    /* font-related items */
    /// Number of fonts, e.g. `FontIdCount=4`
    const FONT_COUNT: &[u8] = b"FontIdCount";
    /// `FontName1=Times New Roman`
    const FONT_NAME: &[u8] = b"FontName";
    /// `Size1=9`
    const FONT_SIZE: &[u8] = b"Size";

    /* part-related items */
    /// `PartCount=10`
    const COMP_COUNT: &[u8] = b"CompCount";
    /// `Libref0=Part Name`
    const COMP_LIBREF: &[u8] = b"LibRef";
    /// `CompDescr0=Long description of thing Name`
    const COMP_DESCR: &[u8] = b"CompDescr";
    /// `PartCount0=2` number of subcomponents (seems to default to 2?)
    const COMP_PARTCOUNT: &[u8] = b"PartCount";

    /* generic parameters */
    /// `UniqueID=RFOIKHCI`
    const UNIQUE_ID: &[u8] = b"UniqueID";

    /// Validate a `FileHeader` and extract its information
    ///
    /// `buf` should be empty, we just reuse it to avoid reallocation
    fn parse<F: Read + Seek>(
        cfile: &mut CompoundFile<F>,
        buf: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        let mut stream = cfile.open_stream(Self::STREAMNAME)?;
        stream.read_to_end(buf)?;
        println!("{:x?}", &buf[..10]);
        println!("{:x?}", &buf[buf.len() - 10..]);
        let to_parse = buf
            .get(Self::PXF_LEN..)
            .ok_or(Error::new_invalid_stream(Self::STREAMNAME, 0))?
            .strip_suffix(Self::SFX)
            .ok_or(Error::new_invalid_stream(Self::STREAMNAME, buf.len()))?;
        let sep_pos = to_parse
            .iter()
            .position(|b| *b == b'|')
            .unwrap_or(to_parse.len());
        if &to_parse[..sep_pos] != Self::HEADER {
            return Err(Error::new_invalid_stream(Self::STREAMNAME, Self::PXF_LEN));
        }

        // Note that fonts start at index 1 but components start at index 0
        let mut unique_id: Option<Box<str>> = None;
        let mut fonts: Option<Vec<Font>> = None;
        let mut components: Option<Vec<ComponentMeta>> = None;

        for (key, val) in split_altium_map(to_parse) {
            if key == Self::HEADER_KEY {
                continue;
            }
            if key == Self::FONT_COUNT {
                fonts = Some(vec![Font::default(); parse_utf8(val)?]);
            } else if key == Self::COMP_COUNT {
                components = Some(vec![ComponentMeta::default(); parse_utf8(val)?]);
            } else if key.starts_with(Self::FONT_NAME) {
                let idx: usize = parse_utf8(
                    key.strip_prefix(Self::FONT_NAME)
                        .ok_or(Error::new_invalid_key(key))?,
                )?;
                fonts.as_mut().expect("uninitialized fonts")[idx - 1].name = buf2string(val)?;
            } else if key.starts_with(Self::FONT_SIZE) {
                let idx: usize = parse_utf8(
                    key.strip_prefix(Self::FONT_SIZE)
                        .ok_or(Error::new_invalid_key(key))?,
                )?;
                fonts.as_mut().expect("uninitialized fonts")[idx - 1].size = parse_utf8(val)?;
            } else if key.starts_with(Self::COMP_LIBREF) {
                let idx: usize = parse_utf8(
                    key.strip_prefix(Self::COMP_LIBREF)
                        .ok_or(Error::new_invalid_key(key))?,
                )?;
                components.as_mut().expect("uninitialized components")[idx].libref =
                    buf2string(val)?;
            } else if key.starts_with(Self::COMP_DESCR) {
                let idx: usize = parse_utf8(
                    key.strip_prefix(Self::COMP_DESCR)
                        .ok_or(Error::new_invalid_key(key))?,
                )?;
                components.as_mut().expect("uninitialized components")[idx].description =
                    buf2string(val)?;
            } else if key.starts_with(Self::COMP_PARTCOUNT) {
                let idx: usize = parse_utf8(
                    key.strip_prefix(Self::COMP_PARTCOUNT)
                        .ok_or(Error::new_invalid_key(key))?,
                )?;
                components.as_mut().expect("uninitialized components")[idx].part_count =
                    parse_utf8(val)?;
            } else if key == Self::UNIQUE_ID {
                unique_id = Some(str::from_utf8(val)?.into())
            }
        }

        Ok(Self {
            components: components.unwrap_or_default(),
            uniqe_id: unique_id.unwrap(),
            fonts: fonts.unwrap_or_default(),
        })
    }
}

/// Optional stream that stores keys
struct SectionKeys {
    map: BTreeMap<Box<str>, Box<str>>,
}

impl SectionKeys {
    const STREAMNAME: &str = "SectionKeys";

    fn parse<F: Read + Seek>(
        cfile: &mut CompoundFile<F>,
        buf: &mut Vec<u8>,
    ) -> Result<Self, Error> {
        let map = BTreeMap::new();
        if !cfile.exists(Self::STREAMNAME) {
            return Ok(Self { map });
        }

        let mut stream = cfile.open_stream(Self::STREAMNAME)?;
        stream.read_to_end(buf);

        todo!("haven't yet found an example for this");

        Ok(Self { map })
    }
}

// pub struct Components {}

/// Information available in the header, libref and part count
#[derive(Clone, Debug, Default)]
pub struct ComponentMeta {
    libref: String,
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
}
