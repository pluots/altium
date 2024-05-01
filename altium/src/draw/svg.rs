use std::cmp::{max, min};
use std::mem;

use svg::node::element::SVG as Svg;
use svg::node::{element as el, Text};
use svg::Node;

use super::{canvas, Canvas};
use crate::common::{PosHoriz, PosVert, Rotation};

#[derive(Clone, Debug)]
pub struct SvgCtx {
    svg: Svg,
    /// `(min, max)` values of x
    x_range: Option<(i32, i32)>,
    /// `(min, max)` values of y
    y_range: Option<(i32, i32)>,
    /// True if the image header has already been set
    has_embedded_images: bool,
}

impl SvgCtx {
    pub fn new() -> Self {
        Self {
            svg: Svg::new(),
            x_range: None,
            y_range: None,
            has_embedded_images: false,
        }
    }

    /// Add a node to this svg
    pub fn add_node<T>(&mut self, node: T)
    where
        T: Into<Box<dyn svg::Node>>,
    {
        // Bad API means we need to do memory tricks...
        let mut working = Svg::new();
        mem::swap(&mut self.svg, &mut working);
        working = working.add(node);
        mem::swap(&mut self.svg, &mut working);
    }

    /// Translate from (0, 0) in bottom left to (0, 0) in top left. Makes sure
    /// `x` and `x + width` are within the view box.
    pub fn x_coord(&mut self, x: i32, width: i32) -> i32 {
        let (mut min_x, mut max_x) = self.x_range.unwrap_or((x, x));
        let edge = x + width; // Add width (allows for negative values)
        min_x = min(min(min_x, x), edge);
        max_x = max(max(max_x, x), edge);

        self.x_range = Some((min_x, max_x));
        x
    }

    /// Translate from (0, 0) in bottom left to (0, 0) in top left
    ///
    /// Updates the y location ranges if needed
    pub fn y_coord(&mut self, y: i32, height: i32) -> i32 {
        let new_y = -y - height;
        let (mut min_y, mut max_y) = self.y_range.unwrap_or((new_y, new_y));
        let edge = new_y + height; // Add height (allows for negative values)
        min_y = min(min(min_y, new_y), edge);
        max_y = max(max(max_y, new_y), edge);

        self.y_range = Some((min_y, max_y));
        new_y
    }

    /// Get the svg
    pub fn svg(self) -> Svg {
        let mut svg = self.svg;
        let (min_x, max_x) = self.x_range.unwrap_or((0, 0));
        let (min_y, max_y) = self.y_range.unwrap_or((0, 0));

        // Add a 5% border on all sides
        let side_extra = (max_x - min_x) / 20;
        let vert_extra = (max_y - min_y) / 20;

        svg = svg.set(
            "viewBox",
            format!(
                "{} {} {} {}",
                min_x - side_extra,
                min_y - vert_extra,
                (max_x - min_x) + side_extra * 2,
                (max_y - min_y) + vert_extra * 2,
            ),
        );

        if self.has_embedded_images {
            svg = svg.set("xmlns:xlink", "http://www.w3.org/1999/xlink");
        }

        svg
    }

    /// Set xlink header for embedded images
    pub fn enable_inline_images(&mut self) {
        self.has_embedded_images = true;
    }
}

impl Canvas for SvgCtx {
    #[allow(clippy::similar_names)]
    fn draw_text(&mut self, item: canvas::DrawText) {
        use PosHoriz::{Center, Left, Right};
        use PosVert::{Bottom, Middle, Top};
        use Rotation::{R0, R180, R270, R90};

        let cmt = svg::node::Comment::new(format!("{item:#?}"));
        self.add_node(cmt);

        let (x, y) = (item.x, item.y);
        let (width, height) = text_dims(item.text, item.font.size);
        let halfwidth = width / 2;
        let halfheight = height / 2;

        let anchor = match item.anchor_h {
            PosHoriz::Left => "start",
            PosHoriz::Center => "middle",
            PosHoriz::Right => "end",
        };

        // Offset of max x from min x
        let (xoffn, xoffp) = match item.anchor_h {
            Left => (0, width),
            Center => (halfwidth, halfwidth),
            Right => (width, 0),
        };

        let (yoffn, yoffp) = match item.anchor_v {
            Top => (height, 0),
            Middle => (halfheight, halfheight),
            Bottom => (0, height),
        };

        let (xmin, xmax, ymin, ymax) = match item.rotation {
            R0 => (x - xoffn, x + xoffp, y - yoffn, y + yoffp),
            R90 => (x - yoffp, x + yoffn, y - xoffn, y + xoffp),
            R180 => (x - xoffp, x + xoffn, y - yoffp, y + yoffn),
            R270 => (x - yoffn, x + yoffp, y - xoffp, y + xoffn),
        };

        self.x_coord(xmin, xmax - xmin);
        self.x_coord(ymin, ymax - ymin);

        let txtnode = Text::new(item.text);
        let mut node = el::Text::new()
            .set("x", self.x_coord(x, width))
            .set("y", self.y_coord(y, height))
            .set("text-anchor", anchor)
            .set("font-size", format!("{}px", item.font.size * 7 / 10))
            .set("font-family", format!("{}, sans-serif", item.font.name))
            .set("transform", format!("rotate({})", item.rotation.as_int()));
        node.append(txtnode);
        self.add_node(node);

        // Add a circle to the text anchor
        let node2 = el::Circle::new()
            .set("cx", self.x_coord(x, width))
            .set("cy", self.y_coord(y, height))
            .set("r", 0.5)
            .set("fill", "red");
        self.add_node(node2);
    }

    fn draw_line(&mut self, item: canvas::DrawLine) {
        // let dx = item.start.x - item.end.x;
        let dy = item.start.y - item.end.y;

        let mut node = el::Line::new()
            .set("x1", self.x_coord(item.start.x, 0))
            .set("x2", self.x_coord(item.end.x, 0))
            .set("y1", self.y_coord(item.start.y, dy))
            .set("y2", self.y_coord(item.end.y, dy))
            .set("stroke", item.color.to_hex());

        // if let Some(w) = item.width {
        //     node = node.set("stroke-width", w);
        // }
        node = node.set("stroke-width", format!("{}px", item.width));

        self.add_node(node);
    }

    fn draw_rectangle(&mut self, item: canvas::DrawRectangle) {
        let node = el::Rectangle::new()
            .set("width", item.width)
            .set("height", item.height)
            // Need top left corner to set location
            .set("x", self.x_coord(item.x, item.width))
            .set("y", self.y_coord(item.y, item.height))
            .set("fill", item.fill_color.to_hex())
            .set("stroke", item.stroke_color.to_hex())
            .set("stroke-width", item.stroke_width);
        self.add_node(node);
    }

    fn draw_polygon(&mut self, _item: canvas::DrawPolygon) {
        // todo!()
    }

    fn draw_image(&mut self, _item: canvas::DrawImage) {}

    fn add_comment<S: Into<String>>(&mut self, comment: S) {
        let cmt = svg::node::Comment::new(comment);
        self.add_node(cmt);
    }
}

/// Estimate the size of text
fn text_dims(text: &str, font_size: u16) -> (i32, i32) {
    let fsize_i32: i32 = font_size.into();
    let width = fsize_i32 * i32::try_from(text.len()).unwrap();
    let height = fsize_i32;

    (width, height)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_coord_offsets() {
        let mut svg = SvgCtx::new();
        assert_eq!(10, svg.x_coord(10, 20));
        assert_eq!(-30, svg.y_coord(10, 20));
        assert_eq!(svg.x_range, Some((10, 30)));
        assert_eq!(svg.y_range, Some((-30, -10)));
    }
}
