use crate::{
    common::{Location, PosHoriz, PosVert, Rgb, Rotation90},
    font::Font,
};

/// Generic trait for something that can be drawn to. Beware, unstable!
///
/// These functions are allowed to return without doing anything if needed.
pub trait Canvas: crate::sealed::Sealed {
    fn draw_text(&mut self, item: DrawText);
    fn draw_line(&mut self, item: DrawLine);
    fn draw_polygon(&mut self, item: DrawPolygon);
    fn draw_rectangle(&mut self, item: DrawRectangle);
    fn draw_image(&mut self, item: DrawImage);
    fn draw_polyline(&mut self, item: DrawPolyLine) {
        // Fallback to a default with the regular DrawLine

        for window in item.locations.windows(2) {
            let &[a, b] = window else { unreachable!() };

            self.draw_line(DrawLine {
                start: a,
                end: b,
                color: item.color,
                width: item.width,
                start_cap: item.start_cap,
                end_cap: item.end_cap,
                line_join: item.line_join,
            });
        }
    }
    fn draw_arc(&mut self, item: DrawArc);
    fn add_comment<S: Into<String>>(&mut self, _comment: S) {}
}

/// Line ending.
///
/// See <https://docs.rs/lyon_tessellation/1.0.5/lyon_tessellation/enum.LineCap.html> for more.
#[derive(Clone, Copy, Debug, Default)]
pub enum LineCap {
    /// Stop at the endpoint
    #[default]
    Butt,
    /// Square past the endpoint
    Square,
    /// Rounded cap centered at the endpoint
    Round,
}

/// How two lines should be combined
///
/// See <https://svgwg.org/specs/strokes/#StrokeLinejoinProperty> for more.
#[derive(Clone, Copy, Debug, Default)]
pub enum LineJoin {
    /// Sharp corners
    #[default]
    Miter,
    /// Miter but possibly don't come to a fine point
    MiterClip,
    /// Round over the join point
    Round,
    /// Square off the join point
    Bevel,
}

/// Helper struct to write some text
#[derive(Clone, Copy, Debug, Default)]
pub struct DrawText<'a> {
    pub x: i32,
    pub y: i32,
    pub text: &'a str,
    pub font: &'a Font,
    pub anchor_h: PosHoriz,
    pub anchor_v: PosVert,
    pub color: Rgb,
    pub rotation: Rotation90,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DrawLine {
    pub start: Location,
    pub end: Location,
    pub color: Rgb,
    pub width: u32,
    pub start_cap: LineCap,
    pub end_cap: LineCap,
    pub line_join: LineJoin,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DrawPolyLine<'a> {
    pub locations: &'a [Location],
    pub color: Rgb,
    pub width: u32,
    pub start_cap: LineCap,
    pub end_cap: LineCap,
    pub line_join: LineJoin,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DrawRectangle {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub fill_color: Rgb,
    pub stroke_color: Rgb,
    pub stroke_width: u32,
    pub start_cap: LineCap,
    pub end_cap: LineCap,
    pub line_join: LineJoin,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DrawPolygon<'a> {
    pub locations: &'a [Location],
    pub fill_color: Rgb,
    pub stroke_color: Rgb,
    pub stroke_width: u32,
    pub start_cap: LineCap,
    pub end_cap: LineCap,
    pub line_join: LineJoin,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DrawArc {
    pub center: Location,
    pub x_radius: u32,
    pub y_radius: u32,
    /// Radians
    pub start_angle: f32,
    /// Radians
    pub end_angle: f32,
    pub width: u32,
    pub color: Rgb,
    pub start_cap: LineCap,
    pub end_cap: LineCap,
    pub line_join: LineJoin,
}

pub struct DrawImage {}
