use std::ops::{Deref, DerefMut};

use altium::draw::{
    Canvas,
    DrawImage,
    DrawLine,
    DrawPolygon,
    DrawRectangle,
    DrawText,
    PosHoriz,
    PosVert,
    Rgb,
};
use eframe::egui;
use egui::{Align2, Color32, RichText, Stroke};
use egui_plot::{Line, PlotPoint, PlotPoints, PlotUi, Polygon, Text};

#[repr(transparent)]
pub struct PlotUiWrapper<'a>(pub &'a mut PlotUi);

impl Deref for PlotUiWrapper<'_> {
    type Target = PlotUi;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl DerefMut for PlotUiWrapper<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

impl altium::sealed::Sealed for PlotUiWrapper<'_> {}
impl Canvas for PlotUiWrapper<'_> {
    fn draw_text(&mut self, item: DrawText) {
        let txt = RichText::new(item.text).size(f32::from(item.font.size()) * 2.0);
        // let txt = RichText::new(item.text).size(f32::from(item.font.size()) * 13.8);
        self.text(
            Text::new(PlotPoint::new(item.x, item.y), txt)
                .anchor(to_align2(item.anchor_v, item.anchor_h))
                .color(to_c32(item.color)),
        );
    }

    fn draw_line(&mut self, item: DrawLine) {
        self.line(
            Line::new(vec![
                [f64::from(item.start.x()), f64::from(item.start.y())],
                [f64::from(item.end.x()), f64::from(item.end.y())],
            ])
            .color(to_c32(item.color))
            .width(item.width as f32),
        );
    }

    fn draw_rectangle(&mut self, item: DrawRectangle) {
        let poly = Polygon::new(vec![
            [f64::from(item.x), f64::from(item.y)],
            [f64::from(item.x + item.width), f64::from(item.y)],
            [
                f64::from(item.x + item.width),
                f64::from(item.y + item.height),
            ],
            [f64::from(item.x), f64::from(item.y + item.height)],
            [f64::from(item.x), f64::from(item.y)],
        ])
        .stroke(Stroke {
            // width: f32::from(item.stroke_width) * 20.0,
            width: 10.0,
            color: Color32::LIGHT_RED,
            // color: to_c32(item.stroke_color),
        })
        // .color(Color32::LIGHT_GREEN)
        .fill_color(to_c32(item.fill_color).gamma_multiply(0.6));

        self.polygon(poly);
    }

    fn draw_polygon(&mut self, item: DrawPolygon) {
        let poly = Polygon::new(
            item.locations
                .iter()
                .map(|loc| [f64::from(loc.x()), f64::from(loc.y())])
                .collect::<PlotPoints>(),
        )
        .stroke(Stroke {
            width: item.stroke_width as f32 * 20.0,
            color: to_c32(item.stroke_color),
        })
        .fill_color(to_c32(item.fill_color).gamma_multiply(1.0));

        self.polygon(poly);
    }

    fn draw_image(&mut self, _item: DrawImage) {
        // todo!()
    }
}

fn to_align2(v: PosVert, h: PosHoriz) -> Align2 {
    match (v, h) {
        (PosVert::Top, PosHoriz::Left) => Align2::LEFT_TOP,
        (PosVert::Top, PosHoriz::Center) => Align2::CENTER_TOP,
        (PosVert::Top, PosHoriz::Right) => Align2::RIGHT_TOP,
        (PosVert::Middle, PosHoriz::Left) => Align2::LEFT_CENTER,
        (PosVert::Middle, PosHoriz::Center) => Align2::CENTER_CENTER,
        (PosVert::Middle, PosHoriz::Right) => Align2::RIGHT_CENTER,
        (PosVert::Bottom, PosHoriz::Left) => Align2::LEFT_BOTTOM,
        (PosVert::Bottom, PosHoriz::Center) => Align2::CENTER_BOTTOM,
        (PosVert::Bottom, PosHoriz::Right) => Align2::RIGHT_BOTTOM,
    }
}

fn to_c32(color: Rgb) -> Color32 {
    let Rgb {
        mut r,
        mut g,
        mut b,
    } = color;
    if r == 0 && g == 0 && b == 0 {
        r = 100;
        g = 100;
        b = 100;
    } else if r < 20 && g < 20 && b < 20 {
        r *= 30;
        g *= 30;
        b *= 30;
    }

    Color32::from_rgb(r, g, b)
}
