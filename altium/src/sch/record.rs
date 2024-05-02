//! Records that are stored.
//!
//! A component in a schematic lib typically look something like the following
//! (line breaks and indents added only for clarity)
//!
//! ```text
//! |RECORD=1
//!     |LibReference=CombinedPins
//!     |ComponentDescription=Single passive pin at (0,0) with 0 rotation
//!     |PartCount=2|DisplayModeCount=1
//!     |IndexInSheet=-1
//!     |OwnerPartId=-1
//!     |CurrentPartId=1
//!     |LibraryPath=*
//!     |SourceLibraryName=*
//!     |SheetPartFileName=*
//!     |TargetFileName=*
//!     |UniqueID=OXKUDQCE
//!     |AreaColor=11599871
//!     |Color=128
//!     |PartIDLocked=T
//!     |AllPinCount=1
//!     [pin binary data]
//! |RECORD=34
//!     |IndexInSheet=-1
//!     |OwnerPartId=-1
//!     |Location.X=-5
//!     |Location.Y=5
//!     |Color=8388608
//!     |FontID=2
//!     |Text=Pin?
//!     |Name=Designator
//!     |ReadOnlyState=1
//!     |UniqueID=VWHBCSOI
//! |RECORD=41
//!     |IndexInSheet=-1
//!     |OwnerPartId=-1
//!     |Location.X=-5
//!     |Location.Y=-15
//!     |Color=8388608
//!     |FontID=2
//!     |IsHidden=T
//!     |Name=Comment
//!     |UniqueID=HEUKMELO
//! ```
//!
//! Notably, it's just a bunch of "records" (with meaningful number identifiers)
//! thrown together. Anything that implements `FromRecord` can take the contents
//! of one of these records (an indented section) and parse it.
//!
//! We provide a derive macro for `FromRecord`, so most types in this module
//! don't need to do anything special.
mod draw;
mod parse;

use std::str;

use altium_macros::FromRecord;
pub use draw::SchDrawCtx;
pub(super) use parse::parse_all_records;
use serde::{Deserialize, Serialize};

use super::params::Justification;
use super::pin::SchPin;
use crate::common::{mils_to_nm, Location, LocationFract, ReadOnlyState, UniqueId};
use crate::error::{AddContext, TruncBuf};
use crate::font::FontCollection;
use crate::Error;
use crate::{
    common::Rgb,
    parse::{FromRecord, ParseUtf8},
    ErrorKind,
};

/// A representation of a record in a schematic library or document.
///
/// Altium represents schematic formats as a collection of records. Some have a visual
/// representation whereas others are just metadata. Each variant represents a record
/// type.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SchRecord {
    Undefined,
    MetaData(Box<MetaData>),
    Pin(SchPin),
    IeeeSymbol(IeeeSymbol),
    Label(Label),
    Bezier(Bezier),
    PolyLine(PolyLine),
    Polygon(Polygon),
    Ellipse(Ellipse),
    Piechart(Piechart),
    RectangleRounded(RectangleRounded),
    ElipticalArc(ElipticalArc),
    Arc(Arc),
    Line(Line),
    Rectangle(Rectangle),
    SheetSymbol(SheetSymbol),
    SheetEntry(SheetEntry),
    PowerPort(PowerPort),
    Port(Port),
    NoErc(NoErc),
    NetLabel(NetLabel),
    Bus(Bus),
    Wire(Wire),
    TextFrame(TextFrame),
    Junction(Junction),
    Image(Image),
    Sheet(Sheet),
    SheetName(SheetName),
    FileName(FileName),
    Designator(Designator),
    BusEntry(BusEntry),
    Template(Template),
    Parameter(Parameter),
    ImplementationList(ImplementationList),
    Implementation(Implementation),
    ImplementationChild1(ImplementationChild1),
    ImplementationChild2(ImplementationChild2),
}

impl SchRecord {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Undefined => "Undefined",
            Self::MetaData(_) => "MetaData",
            Self::Pin(_) => "Pin",
            Self::IeeeSymbol(_) => "IeeeSymbol",
            Self::Label(_) => "Label",
            Self::Bezier(_) => "Bezier",
            Self::PolyLine(_) => "PolyLine",
            Self::Polygon(_) => "Polygon",
            Self::Ellipse(_) => "Ellipse",
            Self::Piechart(_) => "Piechart",
            Self::RectangleRounded(_) => "RectangleRounded",
            Self::ElipticalArc(_) => "ElipticalArc",
            Self::Arc(_) => "Arc",
            Self::Line(_) => "Line",
            Self::Rectangle(_) => "Rectangle",
            Self::SheetSymbol(_) => "SheetSymbol",
            Self::SheetEntry(_) => "SheetEntry",
            Self::PowerPort(_) => "PowerPort",
            Self::Port(_) => "Port",
            Self::NoErc(_) => "NoErc",
            Self::NetLabel(_) => "NetLabel",
            Self::Bus(_) => "Bus",
            Self::Wire(_) => "Wire",
            Self::TextFrame(_) => "TextFrame",
            Self::Junction(_) => "Junction",
            Self::Image(_) => "Image",
            Self::Sheet(_) => "Sheet",
            Self::SheetName(_) => "SheetName",
            Self::FileName(_) => "FileName",
            Self::Designator(_) => "Designator",
            Self::BusEntry(_) => "BusEntry",
            Self::Template(_) => "Template",
            Self::Parameter(_) => "Parameter",
            Self::ImplementationList(_) => "ImplementationList",
            Self::Implementation(_) => "Implementation",
            Self::ImplementationChild1(_) => "ImplementationChild1",
            Self::ImplementationChild2(_) => "ImplementationChild2",
        }
    }
}

/// Try all known record types (excludes binary pins)
pub fn parse_any_record(buf: &[u8]) -> Result<SchRecord, Error> {
    let buf = buf.strip_prefix(b"|RECORD=").unwrap_or_else(|| {
        let tb = TruncBuf::new(buf);
        panic!("no record prefix in {tb} ({})", tb.as_str());
    });
    let num_chars = buf.iter().take_while(|ch| ch.is_ascii_digit()).count();
    let record_id_str = str::from_utf8(&buf[..num_chars]).unwrap();
    let record_id: u32 = record_id_str
        .parse()
        .map_err(|e| ErrorKind::ExpectedInt(record_id_str.into(), e))
        .context("error in parse_any_record")?;
    let to_parse = &buf[num_chars..];

    // Try parsing all our types, they will just skip to the next one if the
    // record ID doesn't match
    MetaData::parse_if_matches(record_id, to_parse)
        .or_else(|| SchPin::parse_if_matches(record_id, to_parse))
        .or_else(|| IeeeSymbol::parse_if_matches(record_id, to_parse))
        .or_else(|| Label::parse_if_matches(record_id, to_parse))
        .or_else(|| Bezier::parse_if_matches(record_id, to_parse))
        .or_else(|| PolyLine::parse_if_matches(record_id, to_parse))
        .or_else(|| Polygon::parse_if_matches(record_id, to_parse))
        .or_else(|| Ellipse::parse_if_matches(record_id, to_parse))
        .or_else(|| Piechart::parse_if_matches(record_id, to_parse))
        .or_else(|| RectangleRounded::parse_if_matches(record_id, to_parse))
        .or_else(|| ElipticalArc::parse_if_matches(record_id, to_parse))
        .or_else(|| Arc::parse_if_matches(record_id, to_parse))
        .or_else(|| Line::parse_if_matches(record_id, to_parse))
        .or_else(|| Rectangle::parse_if_matches(record_id, to_parse))
        .or_else(|| SheetSymbol::parse_if_matches(record_id, to_parse))
        .or_else(|| SheetEntry::parse_if_matches(record_id, to_parse))
        .or_else(|| PowerPort::parse_if_matches(record_id, to_parse))
        .or_else(|| Port::parse_if_matches(record_id, to_parse))
        .or_else(|| NoErc::parse_if_matches(record_id, to_parse))
        .or_else(|| NetLabel::parse_if_matches(record_id, to_parse))
        .or_else(|| Bus::parse_if_matches(record_id, to_parse))
        .or_else(|| Wire::parse_if_matches(record_id, to_parse))
        .or_else(|| TextFrame::parse_if_matches(record_id, to_parse))
        .or_else(|| Junction::parse_if_matches(record_id, to_parse))
        .or_else(|| Image::parse_if_matches(record_id, to_parse))
        .or_else(|| Sheet::parse_if_matches(record_id, to_parse))
        .or_else(|| SheetName::parse_if_matches(record_id, to_parse))
        .or_else(|| FileName::parse_if_matches(record_id, to_parse))
        .or_else(|| Designator::parse_if_matches(record_id, to_parse))
        .or_else(|| BusEntry::parse_if_matches(record_id, to_parse))
        .or_else(|| Template::parse_if_matches(record_id, to_parse))
        .or_else(|| Parameter::parse_if_matches(record_id, to_parse))
        .or_else(|| ImplementationList::parse_if_matches(record_id, to_parse))
        .or_else(|| Implementation::parse_if_matches(record_id, to_parse))
        .or_else(|| ImplementationChild1::parse_if_matches(record_id, to_parse))
        .or_else(|| ImplementationChild2::parse_if_matches(record_id, to_parse))
        .unwrap_or_else(|| {
            log::error!("unknown record id {record_id}");
            Ok(SchRecord::Undefined)
        })
}

/// Component metadata (AKA "Component")
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, FromRecord, Serialize, Deserialize)]
#[from_record(id = 1, use_box = true)]
pub struct MetaData {
    all_pin_count: u32,
    pub area_color: Rgb,
    pub color: Rgb,
    current_part_id: u8,
    database_table_name: Box<str>,
    #[from_record(rename = b"ComponentDescription")]
    pub(crate) description: Option<Box<str>>,
    /// Alternative display modes
    display_mode_count: u8,
    index_in_sheet: i16,
    library_path: Box<str>,
    #[from_record(rename = b"LibReference")]
    libref: Box<str>,
    owner_part_id: i8,
    /// Number of parts
    part_count: u8,
    part_id_locked: bool,
    not_use_db_table_name: bool,
    orientation: i32,
    sheet_part_file_name: Box<str>,
    design_item_id: Box<str>,
    source_library_name: Box<str>,
    target_file_name: Box<str>,
    location: Location,
    unique_id: UniqueId,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, FromRecord, Serialize, Deserialize)]
#[from_record(id = 3)]
pub struct IeeeSymbol {
    is_not_accessible: bool,
    pub location: Location,
    owner_index: u8,
    owner_part_id: i8,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, FromRecord, Serialize, Deserialize)]
#[from_record(id = 4)]
pub struct Label {
    pub color: Rgb,
    font_id: u16,
    index_in_sheet: i16,
    is_not_accessible: bool,
    is_mirrored: bool,
    pub location: LocationFract,
    orientation: i32,
    owner_index: u8,
    owner_part_id: i8,
    text: Box<str>,
    pub unique_id: UniqueId,
    pub justification: Justification,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, FromRecord, Serialize, Deserialize)]
#[from_record(id = 5)]
pub struct Bezier {
    color: Rgb,
    index_in_sheet: i16,
    is_not_accessible: bool,
    #[from_record(convert = mils_to_nm)]
    line_width: u32,
    #[from_record(array = true, count = b"LocationCount", map = (X -> x, Y -> y))]
    pub locations: Vec<Location>,
    owner_index: u8,
    owner_part_id: i8,
    owner_part_display_mode: i8,
    pub unique_id: UniqueId,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, FromRecord, Serialize, Deserialize)]
#[from_record(id = 6)]
pub struct PolyLine {
    owner_index: u8,
    owner_part_id: i8,
    is_not_accessible: bool,
    index_in_sheet: i16,
    #[from_record(convert = mils_to_nm)]
    line_width: u32,
    pub color: Rgb,
    #[from_record(array = true, count = b"LocationCount", map = (X -> x, Y -> y))]
    pub locations: Vec<LocationFract>,
    pub unique_id: UniqueId,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, FromRecord, Serialize, Deserialize)]
#[from_record(id = 7)]
pub struct Polygon {
    pub area_color: Rgb,
    pub color: Rgb,
    index_in_sheet: i16,
    is_not_accessible: bool,
    is_solid: bool,
    #[from_record(convert = mils_to_nm)]
    line_width: u32,
    #[from_record(array = true, count = b"LocationCount", map = (X -> x, Y -> y))]
    pub locations: Vec<Location>,
    owner_index: u8,
    owner_part_id: i8,
    owner_part_display_mode: i8,
    pub unique_id: UniqueId,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, FromRecord, Serialize, Deserialize)]
#[from_record(id = 8)]
pub struct Ellipse {
    pub area_color: Rgb,
    pub color: Rgb,
    index_in_sheet: i16,
    is_not_accessible: bool,
    is_solid: bool,
    #[from_record(convert = mils_to_nm)]
    line_width: u32,
    pub location: Location,
    owner_index: u8,
    owner_part_id: i8,
    owner_part_display_mode: i8,
    radius: i32,
    secondary_radius: i32,
    pub unique_id: UniqueId,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, FromRecord, Serialize, Deserialize)]
#[from_record(id = 9)]
pub struct Piechart {
    owner_index: u8,
    owner_part_id: i8,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, FromRecord, Serialize, Deserialize)]
#[from_record(id = 10)]
pub struct RectangleRounded {
    pub area_color: Rgb,
    pub color: Rgb,
    corner: Location,
    corner_x_radius: i32,
    corner_y_radius: i32,
    index_in_sheet: i16,
    is_not_accessible: bool,
    is_solid: bool,
    #[from_record(convert = mils_to_nm)]
    line_width: u32,
    location: Location,
    owner_index: u8,
    owner_part_id: i8,
    owner_part_display_mode: i8,
    transparent: bool,
    pub unique_id: UniqueId,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, FromRecord, Serialize, Deserialize)]
#[from_record(id = 11)]
pub struct ElipticalArc {
    owner_index: u8,
    owner_part_id: i8,
    is_not_accessible: bool,
    index_in_sheet: i16,
    location: LocationFract,
    radius: i8,
    radius_frac: i32,
    secondary_radius: i8,
    secondary_radius_frac: i32,
    #[from_record(convert = mils_to_nm)]
    line_width: i32,
    start_angle: f32,
    end_angle: f32,
    pub color: Rgb,
    pub unique_id: UniqueId,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, FromRecord, Serialize, Deserialize)]
#[from_record(id = 12)]
pub struct Arc {
    owner_index: u8,
    owner_part_id: i8,
    is_not_accessible: bool,
    index_in_sheet: i16,
    pub location: LocationFract,
    radius: i8,
    radius_frac: i32,
    secondary_radius: i8,
    secondary_radius_frac: i32,
    #[from_record(convert = mils_to_nm)]
    line_width: i32,
    start_angle: f32,
    end_angle: f32,
    pub color: Rgb,
    pub unique_id: UniqueId,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, FromRecord, Serialize, Deserialize)]
#[from_record(id = 13)]
pub struct Line {
    pub color: Rgb,
    corner_x: i32,
    corner_y: i32,
    index_in_sheet: i16,
    is_not_accessible: bool,
    is_solid: bool,
    #[from_record(convert = mils_to_nm)]
    line_width: u32,
    location_count: u16,
    location_x: i32,
    location_y: i32,
    owner_index: u8,
    owner_part_id: i8,
    owner_part_display_mode: i8,
    pub unique_id: UniqueId,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, FromRecord, Serialize, Deserialize)]
#[from_record(id = 14)]
pub struct Rectangle {
    pub area_color: Rgb,
    pub color: Rgb,
    /// Top right corner
    pub corner: LocationFract,
    index_in_sheet: i16,
    is_not_accessible: bool,
    pub is_solid: bool,
    #[from_record(convert = mils_to_nm)]
    line_width: u32,
    /// Bottom left corner
    pub location: LocationFract,
    owner_index: u8,
    owner_part_id: i8,
    owner_part_display_mode: i8,
    pub transparent: bool,
    pub unique_id: UniqueId,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, FromRecord, Serialize, Deserialize)]
#[from_record(id = 15)]
pub struct SheetSymbol {
    owner_index: u8,
    owner_part_id: i8,
    index_in_sheet: i16,
    #[from_record(convert = mils_to_nm)]
    line_width: u32,
    pub color: Rgb,
    pub area_color: Rgb,
    is_solid: bool,
    pub location: Location,
    symbol_type: Box<str>,
    show_net_name: bool,
    x_size: i32,
    y_size: i32,
    orientation: i32,
    font_id: u16,
    text: Box<str>,
    pub unique_id: UniqueId,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, FromRecord, Serialize, Deserialize)]
#[from_record(id = 16)]
pub struct SheetEntry {
    owner_index: u8,
    owner_part_id: i8,
    index_in_sheet: i16,
    pub text_color: Rgb,
    pub area_color: Rgb,
    text_font_id: u16,
    text_style: Box<str>,
    name: Box<str>,
    pub unique_id: UniqueId,
    arrow_kind: Box<str>,
    distance_from_top: i32,
    color: Rgb,
    #[from_record(rename = b"IOType")]
    io_type: i32,
    side: i32,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, FromRecord, Serialize, Deserialize)]
#[from_record(id = 17)]
pub struct PowerPort {
    owner_index: u8,
    owner_part_id: i8,
    is_cross_sheet_connector: bool,
    index_in_sheet: i16,
    style: i16,
    show_net_name: bool,
    pub location: Location,
    orientation: i32,
    font_id: u16,
    text: Box<str>,
    pub unique_id: UniqueId,
    pub color: Rgb,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, FromRecord, Serialize, Deserialize)]
#[from_record(id = 18)]
pub struct Port {
    alignment: u16,
    pub area_color: Rgb,
    border_width: i32,
    pub color: Rgb,
    font_id: u16,
    height: i32,
    width: i32,
    index_in_sheet: i16,
    #[from_record(rename = b"IOType")]
    io_type: u16,
    pub location: Location,
    name: Box<str>,
    owner_index: u8,
    owner_part_id: i8,
    pub text_color: Rgb,
    pub unique_id: UniqueId,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, FromRecord, Serialize, Deserialize)]
#[from_record(id = 22)]
pub struct NoErc {
    owner_index: u8,
    owner_part_id: i8,
    index_in_sheet: i16,
    orientation: i16,
    symbol: Box<str>,
    is_active: bool,
    suppress_all: bool,
    pub location: Location,
    pub color: Rgb,
    pub unique_id: UniqueId,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, FromRecord, Serialize, Deserialize)]
#[from_record(id = 25)]
pub struct NetLabel {
    owner_index: u8,
    owner_part_id: i8,
    index_in_sheet: i16,
    pub location: Location,
    pub color: Rgb,
    font_id: u16,
    text: Box<str>,
    pub unique_id: UniqueId,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, FromRecord, Serialize, Deserialize)]
#[from_record(id = 26)]
pub struct Bus {
    owner_index: u8,
    owner_part_id: i8,
    index_in_sheet: i16,
    #[from_record(convert = mils_to_nm)]
    line_width: u32,
    pub color: Rgb,
    #[from_record(array = true, count = b"LocationCount", map = (X -> x, Y -> y))]
    pub locations: Vec<Location>,
    pub unique_id: UniqueId,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, FromRecord, Serialize, Deserialize)]
#[from_record(id = 27)]
pub struct Wire {
    owner_index: u8,
    owner_part_id: i8,
    #[from_record(convert = mils_to_nm)]
    line_width: u32,
    pub color: Rgb,
    #[from_record(array = true, count = b"LocationCount", map = (X -> x, Y -> y))]
    pub locations: Vec<Location>,
    index_in_sheet: i16,
    pub unique_id: UniqueId,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, FromRecord, Serialize, Deserialize)]
#[from_record(id = 28)]
pub struct TextFrame {
    location: LocationFract,
    corner: LocationFract,
    pub area_color: Rgb,
    owner_index: u8,
    owner_part_id: i8,
    font_id: u16,
    alignment: u16,
    word_wrap: bool,
    pub text: Box<str>,
    index_in_sheet: i16,
    clip_to_rect: bool,
    pub unique_id: UniqueId,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, FromRecord, Serialize, Deserialize)]
#[from_record(id = 29)]
pub struct Junction {
    owner_index: u8,
    owner_part_id: i8,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, FromRecord, Serialize, Deserialize)]
#[from_record(id = 30)]
pub struct Image {
    owner_index: u8,
    owner_part_id: i8,
    is_not_accessible: bool,
    index_in_sheet: i16,
    pub location: Location,
    pub corner: Location,
    keep_aspect: bool,
    embed_image: bool,
    file_name: Box<str>,
    pub unique_id: UniqueId,
    corner_x_frac: i32,
    corner_y_frac: i32,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, FromRecord, Serialize, Deserialize)]
#[from_record(id = 31)]
pub struct Sheet {
    owner_index: u8,
    owner_part_id: i8,
    snap_grid_size: i32,
    snap_grid_on: bool,
    visible_grid_on: bool,
    visible_grid_size: i32,
    custom_x: i32,
    custom_y: i32,
    custom_x_zones: u16,
    custom_y_zones: u16,
    custom_margin_width: u16,
    hot_spot_grid_on: bool,
    hot_spot_grid_size: i32,
    system_font: u16,
    #[from_record(array = true, count = b"FontIdCount", map = (FontName -> name, Size -> size))]
    pub(super) fonts: FontCollection,
    border_on: bool,
    sheet_number_space_size: i32,
    pub area_color: Rgb,
    // FIXME: make this an enum
    #[from_record(rename = b"Display_Unit")]
    display_unit: u16,
    #[from_record(rename = b"UseMBCS")]
    use_mbcs: bool,
    #[from_record(rename = b"IsBOC")]
    is_boc: bool,
    // FIXME: seems to be base64
    file_version_info: Box<str>,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, FromRecord, Serialize, Deserialize)]
#[from_record(id = 32)]
pub struct SheetName {
    owner_index: u8,
    owner_part_id: i8,
    index_in_sheet: i16,
    pub location: Location,
    pub color: Rgb,
    font_id: u16,
    pub text: Box<str>,
    pub unique_id: UniqueId,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, FromRecord, Serialize, Deserialize)]
#[from_record(id = 33)]
pub struct FileName {
    owner_index: u8,
    owner_part_id: i8,
    index_in_sheet: i16,
    pub location: Location,
    pub color: Rgb,
    font_id: u16,
    pub text: Box<str>,
    pub unique_id: UniqueId,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, FromRecord, Serialize, Deserialize)]
#[from_record(id = 34)]
pub struct Designator {
    owner_index: u8,
    owner_part_id: i8,
    pub location: Location,
    pub color: Rgb,
    #[from_record(rename = b"FontID")]
    font_id: u16,
    pub unique_id: UniqueId,
    pub name: Box<str>,
    index_in_sheet: i16,
    pub text: Box<str>,
    read_only_state: ReadOnlyState,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, FromRecord, Serialize, Deserialize)]
#[from_record(id = 37)]
pub struct BusEntry {
    owner_index: u8,
    owner_part_id: i8,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, FromRecord, Serialize, Deserialize)]
#[from_record(id = 39)]
pub struct Template {
    owner_index: u8,
    owner_part_id: i8,
    is_not_accessible: bool,
    file_name: Box<str>,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, FromRecord, Serialize, Deserialize)]
#[from_record(id = 41)]
pub struct Parameter {
    owner_index: u8,
    owner_part_id: i8,
    pub location: Location,
    index_in_sheet: i16,
    pub color: Rgb,
    font_id: u16,
    pub unique_id: UniqueId,
    read_only_state: i8,
    pub name: Box<str>,
    is_hidden: bool,
    pub text: Box<str>,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, FromRecord, Serialize, Deserialize)]
#[from_record(id = 44)]
pub struct ImplementationList {
    owner_index: u8,
    owner_part_id: i8,
}

/// Things like models, including footprints
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, FromRecord, Serialize, Deserialize)]
#[from_record(id = 45)]
pub struct Implementation {
    owner_index: u8,
    owner_part_id: i8,
    use_component_library: bool,
    model_name: Box<str>,
    model_type: Box<str>,
    datafile_count: u16,
    model_datafile_entity0: Box<str>,
    model_datafile_kind0: Box<str>,
    is_current: bool,
    datalinks_locked: bool,
    database_datalinks_locked: bool,
    pub unique_id: UniqueId,
    index_in_sheet: i16,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, FromRecord, Serialize, Deserialize)]
#[from_record(id = 46)]
pub struct ImplementationChild1 {
    owner_index: u8,
    owner_part_id: i8,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, FromRecord, Serialize, Deserialize)]
#[from_record(id = 48)]
pub struct ImplementationChild2 {
    owner_index: u8,
    owner_part_id: i8,
}
