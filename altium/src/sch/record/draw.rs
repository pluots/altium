//! How to draw records, components, etc
use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};
use svg::node::{element as el, Text};
use svg::Node;

use crate::common::{Color, Location, PosHoriz, PosVert, Rotation, Visibility};
use crate::draw::canvas::DrawRectangle;
use crate::draw::canvas::{Canvas, DrawLine, DrawText};
use crate::draw::{Draw, DrawPolygon};
use crate::font::{Font, FontCollection};
use crate::sch::params::Justification;
use crate::sch::pin::SchPin;
use crate::sch::record;
use crate::sch::storage::Storage;

// 500k embedded
const MAX_EMBED_SIZE: usize = 500_000;

// FIXME: This context is super bad and weird with, like, triple indirection
// (since the info comes from`Arc`s). We can fix it somehow but it's low
// priority.
#[derive(Debug)]
pub struct SchDrawCtx<'a> {
    pub fonts: &'a FontCollection,
    pub storage: &'a Storage,
}

impl Draw for record::SchRecord {
    type Context<'a> = SchDrawCtx<'a>;

    fn draw<C: Canvas>(&self, canvas: &mut C, ctx: &SchDrawCtx<'_>) {
        match self {
            // record::SchRecord::MetaData(v) => v.draw(canvas, ctx),
            record::SchRecord::Pin(v) => v.draw(canvas, ctx),
            // record::SchRecord::IeeeSymbol(v) => v.draw(canvas, ctx),
            record::SchRecord::Label(v) => v.draw(canvas, ctx),
            // record::SchRecord::Bezier(v) => v.draw(canvas, ctx),
            record::SchRecord::PolyLine(v) => v.draw(canvas, ctx),
            record::SchRecord::Polygon(v) => v.draw(canvas, ctx),
            // record::SchRecord::Ellipse(v) => v.draw(canvas, ctx),
            // record::SchRecord::Piechart(v) => v.draw(canvas, ctx),
            record::SchRecord::RectangleRounded(v) => v.draw(canvas, ctx),
            // record::SchRecord::ElipticalArc(v) => v.draw(canvas, ctx),
            // record::SchRecord::Arc(v) => v.draw(canvas, ctx),
            record::SchRecord::Line(v) => v.draw(canvas, ctx),
            record::SchRecord::Rectangle(v) => v.draw(canvas, ctx),
            // record::SchRecord::SheetSymbol(v) => v.draw(canvas, ctx),
            // record::SchRecord::SheetEntry(v) => v.draw(canvas, ctx),
            // record::SchRecord::PowerPort(v) => v.draw(canvas, ctx),
            // record::SchRecord::Port(v) => v.draw(canvas, ctx),
            // record::SchRecord::NoErc(v) => v.draw(canvas, ctx),
            // record::SchRecord::NetLabel(v) => v.draw(canvas, ctx),
            // record::SchRecord::Bus(v) => v.draw(canvas, ctx),
            // record::SchRecord::Wire(v) => v.draw(canvas, ctx),
            // record::SchRecord::TextFrame(v) => v.draw(canvas, ctx),
            // record::SchRecord::Junction(v) => v.draw(canvas, ctx),
            record::SchRecord::Image(v) => v.draw(canvas, ctx),
            // record::SchRecord::Sheet(v) => v.draw(canvas, ctx),
            // record::SchRecord::SheetName(v) => v.draw(canvas, ctx),
            // record::SchRecord::FileName(v) => v.draw(canvas, ctx),
            // record::SchRecord::BusEntry(v) => v.draw(canvas, ctx),
            // record::SchRecord::Template(v) => v.draw(canvas, ctx),
            record::SchRecord::Parameter(v) => v.draw(canvas, ctx),
            // record::SchRecord::ImplementationList(v) => v.draw_svg(svg, ctx),
            // non-printing types
            // record::SchRecord::Designator(_) | record::SchRecord::Undefined => (),
            // TODO
            _ => (),
        }
    }
}

impl Draw for SchPin {
    type Context<'a> = SchDrawCtx<'a>;

    fn draw<C: Canvas>(&self, canvas: &mut C, ctx: &SchDrawCtx<'_>) {
        use PosHoriz::{Center, Left, Right};
        use PosVert::{Bottom, Middle, Top};
        use Rotation::{R0, R180, R270, R90};

        canvas.add_comment(format!("{self:#?}"));

        let len: i32 = self.length.try_into().unwrap();
        let (x1, y1) = (self.location_x, self.location_y);

        let start = self.location();
        let end = self.location_conn();

        canvas.draw_line(DrawLine {
            start,
            end,
            color: Color::black(),
            width: 4,
            // ..Default::default()
        });

        // Altium draws a small white plus at the pin's connect position, so we
        // do too
        canvas.draw_line(DrawLine {
            start: end.add_x(1),
            end: end.add_x(-1),
            color: Color::white(),
            width: 1,
        });

        canvas.draw_line(DrawLine {
            start: end.add_y(1),
            end: end.add_y(-1),
            color: Color::white(),
            width: 1,
        });

        // FIXME: use actual spacing & fonts from pin spec
        let (name_x, name_y, des_x, des_y, txt_rotation) = match self.rotation {
            R0 => (start.x - 2, start.y, start.x + 2, start.y + 1, R0),
            R90 => (start.x, start.y + 2, start.x - 2, start.y, R90),
            R180 => (start.x - 1, start.y, start.x - 1, start.y + 2, R0),
            R270 => (start.x, start.y, start.x, start.y, R90),
        };

        if self.name_vis == Visibility::Visible {
            let (anchor_h, anchor_v) = match self.rotation {
                R0 | R90 => (Right, Middle),
                R180 | R270 => (Left, Middle),
            };

            // Display name to the right of the pin
            canvas.draw_text(DrawText {
                x: name_x,
                y: name_y,
                text: &self.name,
                anchor_h,
                anchor_v,
                rotation: txt_rotation,
                ..Default::default()
            });
        }

        if self.designator_vis == Visibility::Visible {
            let (anchor_h, anchor_v) = match self.rotation {
                R0 | R90 => (Left, Bottom),
                R180 | R270 => (Right, Bottom),
            };

            canvas.draw_text(DrawText {
                x: des_x,
                y: des_y,
                text: &self.designator,
                anchor_h,
                anchor_v,
                rotation: txt_rotation,
                ..Default::default()
            });
        }
    }
}

// impl Draw for record::MetaData {}
// impl Draw for record::IeeeSymbol {}

impl Draw for record::Label {
    type Context<'a> = SchDrawCtx<'a>;

    fn draw<C: Canvas>(&self, canvas: &mut C, ctx: &SchDrawCtx<'_>) {
        let font = &ctx.fonts.get_idx(self.font_id.into());
        let (anchor_h, anchor_v) = self.justification.into();
        canvas.draw_text(DrawText {
            x: self.location_x,
            y: self.location_y,
            text: &self.text,
            font,
            anchor_h,
            anchor_v,
            color: self.color,
            ..Default::default() // rotation: todo!(),
        });
    }
}

// impl Draw for record::Bezier {}

impl Draw for record::PolyLine {
    type Context<'a> = SchDrawCtx<'a>;

    fn draw<C: Canvas>(&self, canvas: &mut C, ctx: &Self::Context<'_>) {
        for window in self.locations.windows(2) {
            let &[a, b] = window else { unreachable!() };

            canvas.draw_line(DrawLine {
                start: a,
                end: b,
                color: self.color,
                width: self.line_width * 4,
            });
        }
    }
}

impl Draw for record::Polygon {
    type Context<'a> = SchDrawCtx<'a>;

    fn draw<C: Canvas>(&self, canvas: &mut C, ctx: &SchDrawCtx<'_>) {
        canvas.draw_polygon(DrawPolygon {
            locations: &self.locations,
            fill_color: self.area_color,
            stroke_color: self.color,
            stroke_width: self.line_width,
        });
    }
}

// impl Draw for record::Ellipse {}
// impl Draw for record::Piechart {}

impl Draw for record::RectangleRounded {
    type Context<'a> = SchDrawCtx<'a>;

    fn draw<C: Canvas>(&self, canvas: &mut C, ctx: &SchDrawCtx<'_>) {
        let width = self.corner_x - self.location_x;
        let height = self.corner_y - self.location_y;

        // FIXME: rounded rectangle
        canvas.draw_rectangle(DrawRectangle {
            x: self.location_x,
            y: self.location_y,
            width,
            height,
            fill_color: self.area_color,
            stroke_color: self.color,
            stroke_width: self.line_width,
        });
    }
}

// impl Draw for record::ElipticalArc {}
// impl Draw for record::Arc {}

impl Draw for record::Line {
    type Context<'a> = SchDrawCtx<'a>;

    fn draw<C: Canvas>(&self, canvas: &mut C, ctx: &Self::Context<'_>) {
        canvas.draw_line(DrawLine {
            start: Location::new(self.location_x, self.location_y),
            end: Location::new(self.corner_x, self.corner_y),
            color: self.color,
            width: self.line_width,
        });
    }
}

impl Draw for record::Rectangle {
    type Context<'a> = SchDrawCtx<'a>;

    fn draw<C: Canvas>(&self, canvas: &mut C, ctx: &SchDrawCtx<'_>) {
        let width = self.corner_x - self.location_x;
        let height = self.corner_y - self.location_y;

        canvas.draw_rectangle(DrawRectangle {
            x: self.location_x,
            y: self.location_y,
            width,
            height,
            fill_color: self.area_color,
            stroke_color: self.color,
            stroke_width: self.line_width,
        });
    }
}

// impl Draw for record::SheetSymbol {}
// impl Draw for record::SheetEntry {}
// impl Draw for record::PowerPort {}
// impl Draw for record::Port {}
// impl Draw for record::NoErc {}
// impl Draw for record::NetLabel {}
// impl Draw for record::Bus {}
// impl Draw for record::Wire {}
// impl Draw for record::TextFrame {}
// impl Draw for record::Junction {}
impl Draw for record::Image {
    type Context<'a> = SchDrawCtx<'a>;

    fn draw<C: Canvas>(&self, canvas: &mut C, ctx: &SchDrawCtx<'_>) {
        // TODO
        // TODO: just set to the URL. Maybe set whether or not to encode
        // somehow?
        // if !self.embed_image {
        //     return;
        // }

        // let Some(data) = ctx.storage.get_data(&self.file_name) else {
        //     eprintln!("unable to find image at {}", self.file_name);
        //     return;
        // };

        // let width = self.corner_x - self.location_x;
        // let height = self.corner_y - self.location_y;

        // let mut b64_str = "data:image/png;base64,".to_owned();
        // STANDARD_NO_PAD.encode_string(data, &mut b64_str);
        // assert!(
        //     b64_str.len() < MAX_EMBED_SIZE,
        //     "max size {MAX_EMBED_SIZE} bytes but got {}",
        //     b64_str.len()
        // );

        // let node = el::Image::new()
        //     .set("width", width)
        //     .set("height", height)
        //     // Need top left corner to set location
        //     .set("x", svg.x_coord(self.location_x, width))
        //     .set("y", svg.y_coord(self.location_y, height))
        //     .set("xlink:href", b64_str);
        // svg.add_node(node);

        // svg.enable_inline_images();
    }
}
// impl Draw for record::Sheet {}
// impl Draw for record::SheetName {}
// impl Draw for record::FileName {}
// impl Draw for record::BusEntry {}
// impl Draw for record::Template {}

impl Draw for record::Parameter {
    type Context<'a> = SchDrawCtx<'a>;

    fn draw<C: Canvas>(&self, canvas: &mut C, ctx: &SchDrawCtx<'_>) {
        let font = &ctx.fonts.get_idx(self.font_id.into());
        canvas.draw_text(DrawText {
            x: self.location_x,
            y: self.location_y,
            text: &self.text,
            font,
            ..Default::default()
        });
    }
}

// impl Draw for record::ImplementationList {}
