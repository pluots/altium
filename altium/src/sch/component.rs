//! Things related to the entire component

use std::cmp::Ordering;
use std::fs::OpenOptions;
use std::io;
use std::path::Path;
use std::sync::Arc;

use svg::node::element::SVG as Svg;

use super::record::parse_all_records;
use super::storage::Storage;
use super::{SchDrawCtx, SchRecord};
use crate::draw::{Canvas, Draw, SvgCtx};
use crate::font::FontCollection;
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
            name: &self.name,
        };

        for record in &self.records {
            record.draw(&mut draw, &ctx);
        }
        draw.svg()
    }

    /// Draw this component to a SVG and write it to a file. Will create the file
    /// if it does not exist.
    pub fn save_svg<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
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

    /// Compare two components based on name only
    pub fn name_cmp(&self, other: &Self) -> Option<Ordering> {
        self.name.partial_cmp(&other.name)
    }

    /// Iterate over all records in this component
    pub fn records(&self) -> &[SchRecord] {
        self.records.as_slice()
    }
}

impl Draw for &[SchRecord] {
    type Context<'a> = SchDrawCtx<'a>;

    fn draw<C: Canvas>(&self, canvas: &mut C, ctx: &Self::Context<'_>) {
        self.iter().for_each(|r| r.draw(canvas, ctx));
    }
}

impl Draw for Component {
    type Context<'a> = ();

    fn draw<C: Canvas>(&self, canvas: &mut C, _ctx: &()) {
        let ctx = SchDrawCtx {
            fonts: &self.fonts,
            storage: &self.storage,
            name: &self.name,
        };

        self.records.as_slice().draw(canvas, &ctx);
    }
}
