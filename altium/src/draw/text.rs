use svg::node::{element as el, Text};
use svg::Node;

use super::SvgCtx;
use crate::common::Location;
use crate::{
    common::{Color, PosHoriz, PosVert, Rotation},
    font::Font,
    sch::Justification,
};

/// Helper struct to write some text
#[derive(Clone, Debug, Default)]
pub struct DrawText<'a> {
    pub x: i32,
    pub y: i32,
    pub text: &'a str,
    pub font: &'a Font,
    pub anchor_h: PosHoriz,
    pub anchor_v: PosVert,
    pub color: Color,
    pub rotation: Rotation,
}

impl<'a> DrawText<'a> {
    #[allow(clippy::similar_names)]
    pub fn draw(self, svg: &mut SvgCtx) {
        use Justification as J;
        use PosHoriz::{Center, Left, Right};
        use PosVert::{Bottom, Middle, Top};
        use Rotation::{R0, R180, R270, R90};

        let cmt = svg::node::Comment::new(format!("{self:#?}"));
        svg.add_node(cmt);

        let (x, y) = (self.x, self.y);
        let (width, height) = text_dims(self.text, self.font.size);
        let halfwidth = width / 2;
        let halfheight = height / 2;

        let anchor = match self.anchor_h {
            PosHoriz::Left => "start",
            PosHoriz::Center => "middle",
            PosHoriz::Right => "end",
        };

        /// Offset of max x from min x
        let (xoffn, xoffp) = match self.anchor_h {
            Left => (0, width),
            Center => (halfwidth, halfwidth),
            Right => (width, 0),
        };

        let (yoffn, yoffp) = match self.anchor_v {
            Top => (height, 0),
            Middle => (halfheight, halfheight),
            Bottom => (0, height),
        };

        let (xmin, xmax, ymin, ymax) = match self.rotation {
            R0 => (x - xoffn, x + xoffp, y - yoffn, y + yoffp),
            R90 => (x - yoffp, x + yoffn, y - xoffn, y + xoffp),
            R180 => (x - xoffp, x + xoffn, y - yoffp, y + yoffn),
            R270 => (x - yoffn, x + yoffp, y - xoffp, y + xoffn),
        };

        svg.x_coord(xmin, xmax - xmin);
        svg.x_coord(ymin, ymax - ymin);

        let txtnode = Text::new(self.text);
        let mut node = el::Text::new()
            .set("x", svg.x_coord(x, width))
            .set("y", svg.y_coord(y, height))
            .set("text-anchor", anchor)
            .set("font-size", format!("{}px", self.font.size * 7 / 10))
            .set("font-family", format!("{}, sans-serif", self.font.name))
            .set("transform", format!("rotate({})", self.rotation.as_int()));
        node.append(txtnode);
        svg.add_node(node);

        // Add a circle to the text anchor
        let node2 = el::Circle::new()
            .set("cx", svg.x_coord(x, width))
            .set("cy", svg.y_coord(y, height))
            .set("r", 0.5)
            .set("fill", "red");
        svg.add_node(node2);
    }
}

/// Estimate the size of text
fn text_dims(text: &str, font_size: u16) -> (i32, i32) {
    let fsize_i32: i32 = font_size.into();
    let width = fsize_i32 * i32::try_from(text.len()).unwrap();
    let height = fsize_i32;

    (width, height)
}

#[derive(Clone, Debug, Default)]
pub struct DrawLine<'a> {
    pub start: Location,
    pub end: Location,
    pub color: Color,
    pub width: Option<&'a str>,
}

impl DrawLine<'_> {
    pub fn draw(self, svg: &mut SvgCtx) {
        let dx = self.start.x - self.end.x;
        let dy = self.start.y - self.end.y;

        let mut node = el::Line::new()
            .set("x1", svg.x_coord(self.start.x, 0))
            .set("x2", svg.x_coord(self.end.x, 0))
            .set("y1", svg.y_coord(self.start.y, dy))
            .set("y2", svg.y_coord(self.end.y, dy))
            .set("stroke", self.color.to_hex());

        if let Some(w) = self.width {
            node = node.set("stroke-width", w);
        }

        svg.add_node(node);
    }
}
