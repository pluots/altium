use core::{
    cmp::{max, min},
    mem,
};

use svg::node::element::SVG as Svg;

use crate::font::Font;

pub struct SvgCtx {
    svg: Svg,
    /// `(min, max)` values of x
    x_range: Option<(i32, i32)>,
    /// `(min, max)` values of y
    y_range: Option<(i32, i32)>,
}

impl SvgCtx {
    pub fn new() -> Self {
        Self {
            svg: Svg::new(),
            x_range: None,
            y_range: None,
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
        let (mut min_x, mut max_x) = dbg!(self.x_range.unwrap_or((x, x)));
        let edge = dbg!(x + width); // Add width (allows for negative values)
        min_x = min(min(min_x, x), edge);
        max_x = max(max(max_x, x), edge);

        self.x_range = Some(dbg!((min_x, max_x)));
        x
    }

    /// Translate from (0, 0) in bottom left to (0, 0) in top left
    ///
    /// Updates the y location ranges if needed
    pub fn y_coord(&mut self, y: i32, height: i32) -> i32 {
        let new_y = -y - height;
        let (mut min_y, mut max_y) = self.y_range.unwrap_or((new_y, new_y));
        let edge = new_y - height; // Add height (allows for negative values)
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

        dbg!(min_x, max_x, side_extra, min_y, max_y, vert_extra);

        svg = svg.set(
            "viewBox",
            format!(
                "{} {} {} {}",
                min_x - side_extra,
                min_y - vert_extra,
                max_x + side_extra,
                max_y + vert_extra,
            ),
        );
        svg
    }
}

pub trait Draw {
    /// Draw this element to a SVG and return the new SVG
    ///
    /// This has a defualt implementation that does nothing for easier
    /// reusability
    #[allow(unused)]
    fn draw_svg(&self, svg: &mut SvgCtx, fonts: &[Font]) {}
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
        assert_eq!(svg.y_range, Some((-50, -30)));
    }
}
