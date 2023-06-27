use altium_macros::FromRecord;
use num_enum::TryFromPrimitive;

use crate::{
    common::{buf2lstring, Color},
    parse::{FromRecord, ParseUtf8},
    Error,
};

#[derive(Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum SchRecord {
    Undefined,
    Component(Component),
    Pin(Pin),
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

// pub struct Component {
//     meta: ComponentMeta,
// }

// impl Component {
//     /// Get the metadata for this component
//     pub fn meta(&self) -> &ComponentMeta {
//         &self.meta
//     }

//     /// Parse a stream to
//     fn parse(buf: &[u8]) -> Result<Self, Error> {
//         todo!()
//     }

//     fn draw(&self) -> svg::Document {
//         todo!()
//     }
// }

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 1)]
pub struct Component {
    libref: String,
    description: Option<String>,
    /// Number of parts
    part_count: u8,
    /// Alternative display modes
    display_mode_count: u8,
    index_in_sheet: i8,
    owner_part_id: u8,
    current_part_id: u8,
    library_path: String,
    source_library_name: String,
    sheet_part_file_name: String,
    target_file_name: String,
    unique_id: String,
    area_color: Color,
    color: Color,
    part_id_locked: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Pin {
    owner_index: u8,
    owner_part_id: u8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 3)]
pub struct IeeeSymbol {
    owner_index: u8,
    /// bar bar
    #[from_record(rename = b"bar")]
    owner_part_id: u8,
    #[from_record(rename = b"foo")]
    is_not_accessible: bool,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 4)]
pub struct Label {
    owner_index: u8,
    owner_part_id: u8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 5)]
pub struct Bezier {
    owner_index: u8,
    owner_part_id: u8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 6)]
pub struct PolyLine {
    owner_index: u8,
    owner_part_id: u8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 7)]
pub struct Polygon {
    owner_index: u8,
    owner_part_id: u8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 8)]
pub struct Ellipse {
    owner_index: u8,
    owner_part_id: u8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 9)]
pub struct Piechart {
    owner_index: u8,
    owner_part_id: u8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 10)]
pub struct RectangleRounded {
    owner_index: u8,
    owner_part_id: u8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 11)]
pub struct ElipticalArc {
    owner_index: u8,
    owner_part_id: u8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 12)]
pub struct Arc {
    owner_index: u8,
    owner_part_id: u8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 13)]
pub struct Line {
    owner_index: u8,
    owner_part_id: u8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 14)]
pub struct Rectangle {
    owner_index: u8,
    owner_part_id: u8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 15)]
pub struct SheetSymbol {
    owner_index: u8,
    owner_part_id: u8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 16)]
pub struct SheetEntry {
    owner_index: u8,
    owner_part_id: u8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 17)]
pub struct PowerPort {
    owner_index: u8,
    owner_part_id: u8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 18)]
pub struct Port {
    owner_index: u8,
    owner_part_id: u8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 22)]
pub struct NoErc {
    owner_index: u8,
    owner_part_id: u8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 25)]
pub struct NetLabel {
    owner_index: u8,
    owner_part_id: u8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 26)]
pub struct Bus {
    owner_index: u8,
    owner_part_id: u8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 27)]
pub struct Wire {
    owner_index: u8,
    owner_part_id: u8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 28)]
pub struct TextFrame {
    owner_index: u8,
    owner_part_id: u8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 29)]
pub struct Junction {
    owner_index: u8,
    owner_part_id: u8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 30)]
pub struct Image {
    owner_index: u8,
    owner_part_id: u8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 31)]
pub struct Sheet {
    owner_index: u8,
    owner_part_id: u8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 32)]
pub struct SheetName {
    owner_index: u8,
    owner_part_id: u8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 33)]
pub struct FileName {
    owner_index: u8,
    owner_part_id: u8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 34)]
pub struct Designator {
    owner_index: u8,
    owner_part_id: u8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 37)]
pub struct BusEntry {
    owner_index: u8,
    owner_part_id: u8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 39)]
pub struct Template {
    owner_index: u8,
    owner_part_id: u8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 41)]
pub struct Parameter {
    owner_index: u8,
    owner_part_id: u8,
}

#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 44)]
pub struct ImplementationList {
    owner_index: u8,
    owner_part_id: u8,
}
