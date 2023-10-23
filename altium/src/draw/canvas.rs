use crate::{
    common::{Location, PosHoriz, PosVert, Rgb, Rotation90},
    font::Font,
};

/// Generic trait for something that can be drawn. Beware, unstable!
pub trait Canvas: crate::sealed::Sealed {
    fn draw_text(&mut self, item: DrawText);
    fn draw_line(&mut self, item: DrawLine);
    fn draw_polygon(&mut self, item: DrawPolygon);
    fn draw_rectangle(&mut self, item: DrawRectangle);
    fn draw_image(&mut self, item: DrawImage);
    fn add_comment<S: Into<String>>(&mut self, _comment: S) {}
}

/// Helper struct to write some text
#[derive(Clone, Debug, Default)]
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

#[derive(Clone, Debug, Default)]
pub struct DrawLine {
    pub start: Location,
    pub end: Location,
    pub color: Rgb,
    pub width: u16,
    // pub width: Option<&'a str>,
}

#[derive(Clone, Debug, Default)]
pub struct DrawRectangle {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub fill_color: Rgb,
    pub stroke_color: Rgb,
    pub stroke_width: u16,
}

#[derive(Clone, Debug, Default)]
pub struct DrawPolygon<'a> {
    pub locations: &'a [Location],
    pub fill_color: Rgb,
    pub stroke_color: Rgb,
    pub stroke_width: u16,
}

pub struct DrawImage {}
