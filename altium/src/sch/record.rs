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

use std::str;

use altium_macros::FromRecord;
pub use draw::SchDrawCtx;
use num_enum::TryFromPrimitive;

use super::params::Justification;
use super::pin::SchPin;
use crate::common::{ReadOnlyState, UniqueId};
use crate::error::AddContext;
use crate::Error;
use crate::{
    common::Color,
    parse::{FromRecord, ParseUtf8},
    ErrorKind,
};

/// A schematic record may be any of the below items
#[derive(Clone, Debug, PartialEq)]
#[repr(u32)]
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
}

/// Try all known record types (excludes pins)
pub fn parse_any_record(buf: &[u8]) -> Result<SchRecord, Error> {
    let buf = buf.strip_prefix(b"|RECORD=").unwrap();
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
        .unwrap_or_else(|| {
            eprintln!("unknown record id {record_id}");
            Ok(SchRecord::Undefined)
        })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, TryFromPrimitive)]
#[repr(u8)]
enum PinType {
    Input = 0,
    Id = 1,
    Output = 2,
    OpenCollector = 3,
    Passive = 4,
    HighZ = 5,
    OpenEmitter = 6,
    Power = 7,
}

/// Component metadata (AKA "Component")
#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 1, use_box = true)]
pub struct MetaData {
    all_pin_count: u32,
    area_color: Color,
    color: Color,
    current_part_id: u8,
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
    sheet_part_file_name: Box<str>,
    source_library_name: Box<str>,
    target_file_name: Box<str>,
    unique_id: UniqueId,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 3)]
pub struct IeeeSymbol {
    is_not_accessible: bool,
    location_x: i32,
    location_y: i32,
    owner_index: u8,
    owner_part_id: i8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 4)]
pub struct Label {
    color: Color,
    font_id: u16,
    index_in_sheet: i16,
    is_not_accessible: bool,
    location_x: i32,
    location_y: i32,
    owner_index: u8,
    owner_part_id: i8,
    text: Box<str>,
    unique_id: UniqueId,
    justification: Justification,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 5)]
pub struct Bezier {
    owner_index: u8,
    owner_part_id: i8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 6)]
pub struct PolyLine {
    owner_index: u8,
    owner_part_id: i8,
    is_not_accessible: bool,
    line_width: i8,
    color: Color,
    location_count: u16,
    // TODO: how to handle X1 Y1 X2 Y2 headers
    unique_id: UniqueId,
    index_in_sheet: i16,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 7)]
pub struct Polygon {
    owner_index: u8,
    owner_part_id: i8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 8)]
pub struct Ellipse {
    owner_index: u8,
    owner_part_id: i8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 9)]
pub struct Piechart {
    owner_index: u8,
    owner_part_id: i8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 10)]
pub struct RectangleRounded {
    owner_index: u8,
    owner_part_id: i8,
    owner_part_display_mode: i8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 11)]
pub struct ElipticalArc {
    owner_index: u8,
    owner_part_id: i8,
    is_not_accessible: bool,
    index_in_sheet: i16,
    location_x: i32,
    location_y: i32,
    radius: i8,
    radius_frac: i16,
    secondary_radius: i8,
    secondary_radius_frac: i16,
    line_width: i8,
    start_angle: f32,
    end_angle: f32,
    color: Color,
    unique_id: UniqueId,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 12)]
pub struct Arc {
    owner_index: u8,
    owner_part_id: i8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 13)]
pub struct Line {
    owner_index: u8,
    owner_part_id: i8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 14)]
pub struct Rectangle {
    area_color: Color,
    color: Color,
    /// Top right corner
    corner_x: i32,
    corner_y: i32,
    index_in_sheet: i16,
    is_not_accessible: bool,
    is_solid: bool,
    line_width: u16,
    /// Bottom left corner
    location_x: i32,
    location_y: i32,
    owner_index: u8,
    owner_part_id: i8,
    owner_part_display_mode: i8,
    unique_id: UniqueId,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 15)]
pub struct SheetSymbol {
    owner_index: u8,
    owner_part_id: i8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 16)]
pub struct SheetEntry {
    owner_index: u8,
    owner_part_id: i8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 17)]
pub struct PowerPort {
    owner_index: u8,
    owner_part_id: i8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 18)]
pub struct Port {
    owner_index: u8,
    owner_part_id: i8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 22)]
pub struct NoErc {
    owner_index: u8,
    owner_part_id: i8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 25)]
pub struct NetLabel {
    owner_index: u8,
    owner_part_id: i8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 26)]
pub struct Bus {
    owner_index: u8,
    owner_part_id: i8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 27)]
pub struct Wire {
    owner_index: u8,
    owner_part_id: i8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 28)]
pub struct TextFrame {
    owner_index: u8,
    owner_part_id: i8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 29)]
pub struct Junction {
    owner_index: u8,
    owner_part_id: i8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 30)]
pub struct Image {
    owner_index: u8,
    owner_part_id: i8,
    is_not_accessible: bool,
    index_in_sheet: i16,
    location_x: i32,
    location_y: i32,
    corner_x: i32,
    corner_y: i32,
    keep_aspect: bool,
    embed_image: bool,
    file_name: Box<str>,
    unique_id: UniqueId,
    corner_x_frac: i32,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 31)]
pub struct Sheet {
    owner_index: u8,
    owner_part_id: i8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 32)]
pub struct SheetName {
    owner_index: u8,
    owner_part_id: i8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 33)]
pub struct FileName {
    owner_index: u8,
    owner_part_id: i8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 34)]
pub struct Designator {
    owner_index: u8,
    owner_part_id: i8,
    location_x: i32,
    location_y: i32,
    color: Color,
    #[from_record(rename = b"FontID")]
    font_id: u16,
    unique_id: UniqueId,
    name: Box<str>,
    index_in_sheet: i16,
    text: Box<str>,
    read_only_state: ReadOnlyState,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 37)]
pub struct BusEntry {
    owner_index: u8,
    owner_part_id: i8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 39)]
pub struct Template {
    owner_index: u8,
    owner_part_id: i8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 41)]
pub struct Parameter {
    owner_index: u8,
    owner_part_id: i8,
    location_x: i32,
    location_y: i32,
    index_in_sheet: i16,
    color: Color,
    font_id: u16,
    unique_id: UniqueId,
    name: Box<str>,
    is_hidden: bool,
    text: Box<str>,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 44)]
pub struct ImplementationList {
    owner_index: u8,
    owner_part_id: i8,
}
