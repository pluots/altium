//! How to draw records, components, etc
use svg::node::{element as el, Text};
use svg::Node;

use crate::common::{Color, Coords2};
use crate::draw::{Draw, SvgCtx};
use crate::font::Font;
use crate::sch::params::Justification;
use crate::sch::pin::{Rotation, SchPin};
use crate::sch::record;

impl Draw for record::SchRecord {
    fn draw_svg(&self, svg: &mut SvgCtx, fonts: &[Font]) {
        match self {
            record::SchRecord::MetaData(v) => v.draw_svg(svg, fonts),
            record::SchRecord::Pin(v) => v.draw_svg(svg, fonts),
            record::SchRecord::IeeeSymbol(v) => v.draw_svg(svg, fonts),
            record::SchRecord::Label(v) => v.draw_svg(svg, fonts),
            record::SchRecord::Bezier(v) => v.draw_svg(svg, fonts),
            record::SchRecord::PolyLine(v) => v.draw_svg(svg, fonts),
            record::SchRecord::Polygon(v) => v.draw_svg(svg, fonts),
            record::SchRecord::Ellipse(v) => v.draw_svg(svg, fonts),
            record::SchRecord::Piechart(v) => v.draw_svg(svg, fonts),
            record::SchRecord::RectangleRounded(v) => v.draw_svg(svg, fonts),
            record::SchRecord::ElipticalArc(v) => v.draw_svg(svg, fonts),
            record::SchRecord::Arc(v) => v.draw_svg(svg, fonts),
            record::SchRecord::Line(v) => v.draw_svg(svg, fonts),
            record::SchRecord::Rectangle(v) => v.draw_svg(svg, fonts),
            record::SchRecord::SheetSymbol(v) => v.draw_svg(svg, fonts),
            record::SchRecord::SheetEntry(v) => v.draw_svg(svg, fonts),
            record::SchRecord::PowerPort(v) => v.draw_svg(svg, fonts),
            record::SchRecord::Port(v) => v.draw_svg(svg, fonts),
            record::SchRecord::NoErc(v) => v.draw_svg(svg, fonts),
            record::SchRecord::NetLabel(v) => v.draw_svg(svg, fonts),
            record::SchRecord::Bus(v) => v.draw_svg(svg, fonts),
            record::SchRecord::Wire(v) => v.draw_svg(svg, fonts),
            record::SchRecord::TextFrame(v) => v.draw_svg(svg, fonts),
            record::SchRecord::Junction(v) => v.draw_svg(svg, fonts),
            record::SchRecord::Image(v) => v.draw_svg(svg, fonts),
            record::SchRecord::Sheet(v) => v.draw_svg(svg, fonts),
            record::SchRecord::SheetName(v) => v.draw_svg(svg, fonts),
            record::SchRecord::FileName(v) => v.draw_svg(svg, fonts),
            record::SchRecord::BusEntry(v) => v.draw_svg(svg, fonts),
            record::SchRecord::Template(v) => v.draw_svg(svg, fonts),
            record::SchRecord::Parameter(v) => v.draw_svg(svg, fonts),
            record::SchRecord::ImplementationList(v) => v.draw_svg(svg, fonts),
            // non-printing types
            record::SchRecord::Designator(_) | record::SchRecord::Undefined => (),
        }
    }
}

impl Draw for SchPin {
    fn draw_svg(&self, svg: &mut SvgCtx, _fonts: &[Font]) {
        let len: i32 = self.length.try_into().unwrap();
        let (x1, y1) = (self.location_x, self.location_y);
        let (x2, y2) = match self.rotation {
            Rotation::R0 => (x1 + len, y1),
            Rotation::R90 => (x1, y1 + len),
            Rotation::R180 => (x1 - len, y1),
            Rotation::R270 => (x1, y1 - len),
        };

        let node = el::Line::new()
            .set("x1", svg.x_coord(x1, x2 - x1))
            .set("x2", svg.x_coord(x2, x1 - x2))
            .set("y1", svg.y_coord(y1, y2 - y1))
            .set("y2", svg.y_coord(y2, y1 - y2))
            .set("stroke", "black");
        svg.add_node(node);
    }
}

impl Draw for record::MetaData {}
impl Draw for record::IeeeSymbol {}

impl Draw for record::Label {
    fn draw_svg(&self, svg: &mut SvgCtx, fonts: &[Font]) {
        let font = &fonts[usize::from(self.font_id)];
        let (width, height) = text_dims(&self.text, font);

        let txtnode = Text::new(self.text.clone());
        let mut node = el::Text::new()
            .set("x", svg.x_coord(self.location_x, width))
            .set("y", svg.y_coord(self.location_y, height))
            .set("font-size", format!("{}px", font.size))
            // Default ot sans-serif if not specified
            .set("font-family", format!("{}, sans-serif", font.name));
        node.append(txtnode);
        svg.add_node(node);
    }
}

impl Draw for record::Bezier {}
impl Draw for record::PolyLine {}
impl Draw for record::Polygon {}
impl Draw for record::Ellipse {}
impl Draw for record::Piechart {}
impl Draw for record::RectangleRounded {}
impl Draw for record::ElipticalArc {}
impl Draw for record::Arc {}
impl Draw for record::Line {}

impl Draw for record::Rectangle {
    fn draw_svg(&self, svg: &mut SvgCtx, _fonts: &[Font]) {
        dbg!(&self);
        let width = self.corner_x - self.location_x;
        let height = self.corner_y - self.location_y;
        dbg!(width, height);

        let node = el::Rectangle::new()
            .set("width", width)
            .set("height", height)
            // Need top left corner to set location
            .set("x", svg.x_coord(self.location_x, width))
            .set("y", svg.y_coord(self.location_y, height))
            .set("fill", self.area_color.to_hex())
            .set("stroke", self.color.to_hex())
            .set("stroke-width", self.line_width);
        svg.add_node(node);
    }
}

impl Draw for record::SheetSymbol {}
impl Draw for record::SheetEntry {}
impl Draw for record::PowerPort {}
impl Draw for record::Port {}
impl Draw for record::NoErc {}
impl Draw for record::NetLabel {}
impl Draw for record::Bus {}
impl Draw for record::Wire {}
impl Draw for record::TextFrame {}
impl Draw for record::Junction {}
impl Draw for record::Image {}
impl Draw for record::Sheet {}
impl Draw for record::SheetName {}
impl Draw for record::FileName {}

impl Draw for record::BusEntry {}
impl Draw for record::Template {}

impl Draw for record::Parameter {
    fn draw_svg(&self, svg: &mut SvgCtx, fonts: &[Font]) {
        let font = &fonts[usize::from(self.font_id)];
        let (width, height) = text_dims(&self.text, font);
        let mut node = Text::new(self.text.clone());
        node.assign("x", svg.x_coord(self.location_x, width));
        node.assign("y", svg.y_coord(self.location_y, height));
        svg.add_node(node);
    }
}

impl Draw for record::ImplementationList {}

// fn draw_text(
//     svg: &mut DrawSvg,
//     location: Coords2,
//     text: &str,
//     font: &Font,
//     align: Justification,
//     color: Color,
//     rotation: Rotation
// ) {
//     use Justification as J;
//     use Rotation as R;

//     let width = i32::from(font.size) * i32::try_from(text.len()).unwrap();
//     let height = i32::from(font.size);
//     let (bl_x, bl_y, ) = match (align, rotation) {
//         (J::BottomLeft, R::R0) => todo!(),
//         (J::BottomLeft, R::R90) => todo!(),
//         (J::BottomLeft, R::R180) => todo!(),
//         (J::BottomLeft, R::R270) => todo!(),
//         (J::BottomCenter, R::R0) => todo!(),
//         (J::BottomCenter, R::R90) => todo!(),
//         (J::BottomCenter, R::R180) => todo!(),
//         (J::BottomCenter, R::R270) => todo!(),
//         (J::BottomRight, R::R0) => todo!(),
//         (J::BottomRight, R::R90) => todo!(),
//         (J::BottomRight, R::R180) => todo!(),
//         (J::BottomRight, R::R270) => todo!(),
//         (J::CenterLeft, R::R0) => todo!(),
//         (J::CenterLeft, R::R90) => todo!(),
//         (J::CenterLeft, R::R180) => todo!(),
//         (J::CenterLeft, R::R270) => todo!(),
//         (J::CenterCenter, R::R0) => todo!(),
//         (J::CenterCenter, R::R90) => todo!(),
//         (J::CenterCenter, R::R180) => todo!(),
//         (J::CenterCenter, R::R270) => todo!(),
//         (J::CenterRight, R::R0) => todo!(),
//         (J::CenterRight, R::R90) => todo!(),
//         (J::CenterRight, R::R180) => todo!(),
//         (J::CenterRight, R::R270) => todo!(),
//         (J::TopLeft, R::R0) => todo!(),
//         (J::TopLeft, R::R90) => todo!(),
//         (J::TopLeft, R::R180) => todo!(),
//         (J::TopLeft, R::R270) => todo!(),
//         (J::TopCenter, R::R0) => todo!(),
//         (J::TopCenter, R::R90) => todo!(),
//         (J::TopCenter, R::R180) => todo!(),
//         (J::TopCenter, R::R270) => todo!(),
//         (J::TopRight, R::R0) => todo!(),
//         (J::TopRight, R::R90) => todo!(),
//         (J::TopRight, R::R180) => todo!(),
//         (J::TopRight, R::R270) => todo!(),
//     }

//     node.assign("x", svg.x_coord(self.location_x, width));
//     node.assign("y", svg.y_coord(self.location_y, height));
// }

/// Estimate the size of text
fn text_dims(text: &str, font: &Font) -> (i32, i32) {
    let width = i32::from(font.size) * i32::try_from(text.len()).unwrap();
    let height = i32::from(font.size);

    (width, height)
}
