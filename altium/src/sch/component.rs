//! Things related to the entire component

use core::fmt::Write;
use std::fs::File;
use std::io;
use std::path::Path;
use std::sync::Arc;

use svg::node::element::SVG as Svg;

use super::storage::Storage;
use super::{SchDrawCtx, SchRecord};
use crate::draw::{Draw, SvgCtx};
use crate::error::AddContext;
use crate::font::FontCollection;
use crate::sch::pin::SchPin;
use crate::sch::record::parse_any_record;
use crate::Error;

/// Representation of a component
// TODO: should this just be "entry" or something? Not sure how it works with a
// schematic
#[derive(Clone, Debug)]
pub struct Component {
    pub(crate) name: Box<str>,
    pub(crate) records: Vec<SchRecord>,
    // TODO: figure out how to combine these
    pub(crate) fonts: Arc<FontCollection>,
    pub(crate) storage: Arc<Storage>,
}

impl Component {
    pub(crate) fn from_buf(
        name: &str,
        buf: &[u8],
        fonts: Arc<FontCollection>,
        storage: Arc<Storage>,
    ) -> Result<Self, Error> {
        Ok(Component {
            name: name.into(),
            records: parse_all_records(buf, name)?,
            fonts,
            storage,
        })
    }

    /// Draw this component to a SVG
    pub fn svg(&self) -> Svg {
        let mut draw = SvgCtx::new();
        let ctx = SchDrawCtx {
            fonts: &self.fonts,
            storage: &self.storage,
        };

        for record in &self.records {
            record.draw_svg(&mut draw, &ctx);
        }
        draw.svg()
    }

    /// Draw this component to a SVG and write it to a file
    pub fn save_svg<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let file = File::open(path)?;
        svg::write(&file, &self.svg())
    }

    /// The name of this part
    pub fn name(&self) -> &str {
        &self.name
    }

    /// This part's description
    pub fn description(&self) -> &str {
        let meta = self
            .records
            .iter()
            .find_map(|record| {
                if let SchRecord::MetaData(d) = record {
                    Some(d)
                } else {
                    None
                }
            })
            .expect("no metadata record");
        meta.description.as_deref().unwrap_or("")
    }
}

/// Given a buffer for a component, split the records up
///
/// Name is only used for diagnostics
fn parse_all_records(buf: &[u8], name: &str) -> Result<Vec<SchRecord>, Error> {
    // Our info u32 is something like `0xttllllll`, where `tt` are 8 bits
    // representing a type (currently only values 0 and 1 known) and the `l`s
    // are the length
    const TY_SHIFT: u32 = 24;
    const TY_MAEK: u32 = 0xff000000;
    const LEN_MASK: u32 = 0x00ffffff;
    const UTF8_RECORD_TY: u32 = 0x00;
    const PIN_RECORD_TY: u32 = 0x01;
    // No magic values :)
    const U32_BYTES: usize = 4;

    let mut working = buf;
    let mut parsed = Vec::new();
    while !working.is_empty() {
        assert!(
            working.len() >= 4,
            "expected at least 4 bytes, only got {}",
            working.len()
        );

        let info = u32::from_le_bytes(working[..4].try_into().unwrap());
        let ty = (info & TY_MAEK) >> TY_SHIFT;
        let len: usize = (info & LEN_MASK).try_into().unwrap();

        // Don't include the null terminator (which is included in `len`)
        let to_parse = &working[U32_BYTES..(U32_BYTES + len - 1)];

        // But do do a sanity check that the null exists
        assert_eq!(working[U32_BYTES + len - 1], 0, "Expected null terimation");

        working = &working[U32_BYTES + len..];

        let record = match ty {
            UTF8_RECORD_TY => parse_any_record(to_parse),
            PIN_RECORD_TY => SchPin::parse(to_parse).map_err(Into::into),
            _ => panic!("unexpected record type {ty:02x}"),
        };

        parsed.push(record.context(format!("in `parse_all_records` for `{name}`"))?);
    }

    Ok(parsed)
}
