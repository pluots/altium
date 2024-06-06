//! How to draw records, components, etc

use log::warn;

use crate::common::{Location, LocationFract, PosHoriz, PosVert, Rgb, Rotation90, Visibility};
use crate::draw::canvas::{Canvas, DrawLine, DrawText};
use crate::draw::canvas::{DrawRectangle, LineCap};
use crate::draw::{Draw, DrawArc, DrawPolyLine, DrawPolygon};
use crate::font::FontCollection;
use crate::sch::pin::SchPin;
use crate::sch::record;
use crate::sch::storage::Storage;

// 500k embedded
#[allow(unused)]
const MAX_EMBED_SIZE: usize = 500_000;

/// Grid width used for reference; 0.25mm (?? should be )
const GW: i32 = 250_000;
const GW_H: i32 = GW / 2;
const GW_Q: i32 = GW / 4;

/// What to use when we have nothing else
const THICK_LINE_WIDTH: u32 = 30_000;
const THIN_LINE_WIDTH: u32 = 10_000;

/// Context needed to draw most schematic items
// FIXME: This context is super bad and weird with, like, triple indirection
// (since the info comes from`Arc`s). We can fix it somehow but it's low
// priority.
#[derive(Debug)]
pub struct SchDrawCtx<'a> {
    pub fonts: &'a FontCollection,
    pub storage: &'a Storage,
    /// Just for reference
    pub name: &'a str,
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
            record::SchRecord::Arc(v) => v.draw(canvas, ctx),
            record::SchRecord::Line(v) => v.draw(canvas, ctx),
            record::SchRecord::Rectangle(v) => v.draw(canvas, ctx),
            record::SchRecord::SheetSymbol(v) => v.draw(canvas, ctx),
            // record::SchRecord::SheetEntry(v) => v.draw(canvas, ctx),
            record::SchRecord::PowerPort(v) => v.draw(canvas, ctx),
            record::SchRecord::Port(v) => v.draw(canvas, ctx),
            record::SchRecord::NoErc(v) => v.draw(canvas, ctx),
            record::SchRecord::NetLabel(v) => v.draw(canvas, ctx),
            record::SchRecord::Bus(v) => v.draw(canvas, ctx),
            record::SchRecord::Wire(v) => v.draw(canvas, ctx),
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

    fn draw<C: Canvas>(&self, canvas: &mut C, _ctx: &SchDrawCtx<'_>) {
        use PosHoriz::{Left, Right};
        use PosVert::{Bottom, Middle};
        use Rotation90::{R0, R180, R270, R90};

        const X_WIDTH: u32 = 5_000;

        canvas.add_comment(format!("{self:#?}"));

        let start = self.location();
        let end = self.location_conn();

        canvas.draw_line(DrawLine {
            start,
            end,
            color: Rgb::black(),
            width: THICK_LINE_WIDTH,
            start_cap: LineCap::Round,
            ..Default::default()
        });

        // Altium draws a small white plus at the pin's connect position, so we
        // do too
        canvas.draw_line(DrawLine {
            start: end.add_x(10000),
            end: end.add_x(-10000),
            color: Rgb::white(),
            width: X_WIDTH,
            ..Default::default()
        });

        canvas.draw_line(DrawLine {
            start: end.add_y(10000),
            end: end.add_y(-10000),
            color: Rgb::white(),
            width: X_WIDTH,
            ..Default::default()
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
            x: self.location.x,
            y: self.location.y,
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

    fn draw<C: Canvas>(&self, canvas: &mut C, _ctx: &Self::Context<'_>) {
        let locations: Vec<_> = self
            .locations
            .iter()
            .copied()
            .map(LocationFract::as_location)
            .collect();

        canvas.draw_polyline(DrawPolyLine {
            locations: &locations,
            color: self.color,
            width: self.line_width,
            ..Default::default()
        });
    }
}

impl Draw for record::Polygon {
    type Context<'a> = SchDrawCtx<'a>;

    fn draw<C: Canvas>(&self, canvas: &mut C, _ctx: &SchDrawCtx<'_>) {
        canvas.draw_polygon(DrawPolygon {
            locations: &self.locations,
            fill_color: self.area_color,
            stroke_color: self.color,
            stroke_width: self.line_width,
            ..Default::default()
        });
    }
}

// impl Draw for record::Ellipse {}
// impl Draw for record::Piechart {}

impl Draw for record::RectangleRounded {
    type Context<'a> = SchDrawCtx<'a>;

    fn draw<C: Canvas>(&self, canvas: &mut C, _ctx: &SchDrawCtx<'_>) {
        let width = self.corner.x - self.location.x;
        let height = self.corner.y - self.location.y;

        // FIXME: rounded rectangle
        canvas.draw_rectangle(DrawRectangle {
            x: self.location.x,
            y: self.location.y,
            width,
            height,
            fill_color: self.area_color,
            stroke_color: self.color,
            stroke_width: self.line_width,
            ..Default::default()
        });
    }
}

// impl Draw for record::ElipticalArc {}
impl Draw for record::Arc {
    type Context<'a> = SchDrawCtx<'a>;

    fn draw<C: Canvas>(&self, canvas: &mut C, _ctx: &Self::Context<'_>) {
        canvas.draw_arc(DrawArc {
            center: self.location.as_location(),
            x_radius: self.radius,
            y_radius: self.secondary_radius,
            start_angle: self.start_angle.to_radians(),
            end_angle: self.end_angle.to_radians(),
            color: self.color,
            width: self.line_width,
            ..Default::default()
        });
    }
}

impl Draw for record::Line {
    type Context<'a> = SchDrawCtx<'a>;

    fn draw<C: Canvas>(&self, canvas: &mut C, _ctx: &Self::Context<'_>) {
        canvas.draw_line(DrawLine {
            start: Location::new(self.location_x, self.location_y),
            end: Location::new(self.corner_x, self.corner_y),
            color: self.color,
            width: self.line_width,
            ..Default::default()
        });
    }
}

impl Draw for record::Rectangle {
    type Context<'a> = SchDrawCtx<'a>;

    fn draw<C: Canvas>(&self, canvas: &mut C, _ctx: &SchDrawCtx<'_>) {
        let width = self.corner.x - self.location.x;
        let height = self.corner.y - self.location.y;

        canvas.draw_rectangle(DrawRectangle {
            x: self.location.x,
            y: self.location.y,
            width,
            height,
            fill_color: self.area_color,
            stroke_color: self.color,
            stroke_width: self.line_width,
            ..Default::default()
        });
    }
}

impl Draw for record::SheetSymbol {
    type Context<'a> = SchDrawCtx<'a>;

    fn draw<C: Canvas>(&self, canvas: &mut C, _ctx: &SchDrawCtx<'_>) {
        canvas.draw_rectangle(DrawRectangle {
            x: self.location.x,
            y: self.location.y - self.y_size,
            width: self.x_size,
            height: self.y_size,
            fill_color: self.area_color,
            stroke_color: self.color,
            stroke_width: self.line_width,
            ..Default::default()
        });
    }
}

impl Draw for record::SheetEntry {
    type Context<'a> = SchDrawCtx<'a>;

    fn draw<C: Canvas>(&self, _canvas: &mut C, _ctx: &SchDrawCtx<'_>) {
        // TODO need to do something with distance_from_top
        // The parent index is the index in all the records
    }
}
impl Draw for record::PowerPort {
    type Context<'a> = SchDrawCtx<'a>;

    fn draw<C: Canvas>(&self, canvas: &mut C, ctx: &SchDrawCtx<'_>) {
        let Location { x, y } = self.location;

        let tmp1; // can't wait for lifetime extension...
        let tmp2;

        // FIXME: care about orientation
        let pairs = if self.style == 0 {
            // A sheet connector is a "power port"
            // Sheet entries look like a double chevron with a small tab.

            let lt = Location::new(self.location.x - (GW + GW_H), self.location.y + GW_H);
            let lm = Location::new(self.location.x - GW, self.location.y);
            let lb = Location::new(self.location.x - (GW + GW_H), self.location.y - GW_H);

            let rt = Location::new(self.location.x - GW, self.location.y + GW_H);
            let rm = Location::new(self.location.x - GW_H, self.location.y);
            let rb = Location::new(self.location.x - GW, self.location.y - GW_H);

            let dash_end = Location::new(self.location.x, self.location.y);

            let l1 = [lt, lm, lb];
            let l2 = [rt, rm, rb];
            let l3 = [rm, dash_end];
            let lines = [l1.as_slice(), l2.as_slice(), l3.as_slice()];

            for points in lines {
                canvas.draw_polyline(DrawPolyLine {
                    locations: points,
                    color: self.color,
                    width: THIN_LINE_WIDTH,
                    ..Default::default()
                });
            }

            return;
        } else if self.style == 2 {
            // Simple VCC bar
            let left = Location::new(x - GW_H, y + GW);
            let right = Location::new(x + GW_H, y + GW);
            let center = Location::new(x, y + GW);
            let bottom = Location::new(x, y);

            tmp1 = [[left, right], [center, bottom]];
            tmp1.as_slice()
        } else if self.style == 4 {
            // GND triple line triangle

            let top = Location::new(x, y);
            let c1 = Location::new(x, y - GW);

            let l1 = Location::new(x - GW, y - GW);
            let r1 = Location::new(x + GW, y - GW);
            let l2 = Location::new(x - (GW * 3 / 4), y - (GW + GW / 3));
            let r2 = Location::new(x + (GW * 3 / 4), y - (GW + GW / 3));
            let l3 = Location::new(x - (GW * 2 / 4), y - (GW + GW * 2 / 3));
            let r3 = Location::new(x + (GW * 2 / 4), y - (GW + GW * 2 / 3));
            let l4 = Location::new(x - (GW / 4), y - (GW * 2));
            let r4 = Location::new(x + (GW / 4), y - (GW * 2));

            tmp2 = [[top, c1], [l1, r1], [l2, r2], [l3, r3], [l4, r4]];
            tmp2.as_slice()
        } else {
            warn!(
                "unrecognized power port style {} in '{}'",
                self.style, ctx.name
            );
            return;
        };

        for [start, end] in pairs {
            canvas.draw_line(DrawLine {
                start: *start,
                end: *end,
                color: self.color,
                width: THIN_LINE_WIDTH,
                ..Default::default()
            });
        }
    }
}

impl Draw for record::Port {
    type Context<'a> = SchDrawCtx<'a>;

    fn draw<C: Canvas>(&self, canvas: &mut C, ctx: &SchDrawCtx<'_>) {
        // Ports are a rectangle with one or both end pointed
        // FIXME: need to take direction into account

        let h2 = self.height / 2;
        let mut locations = [Location::default(); 6];

        locations[0] = Location::new(self.location.x, self.location.y + h2);
        locations[1] = Location::new(self.location.x + self.width - h2, self.location.y + h2);
        locations[2] = Location::new(self.location.x + self.width, self.location.y);
        locations[3] = Location::new(self.location.x + self.width - h2, self.location.y - h2);
        locations[4] = Location::new(self.location.x, self.location.y - h2);
        locations[5] = Location::new(self.location.x, self.location.y + h2);

        canvas.draw_polygon(DrawPolygon {
            locations: &locations,
            fill_color: self.area_color,
            stroke_color: self.color,
            stroke_width: self.border_width.try_into().unwrap(),
            ..Default::default()
        });

        let font = &ctx.fonts.get_idx(self.font_id.into());
        canvas.draw_text(DrawText {
            x: self.location.x,
            y: self.location.y,
            text: &self.name,
            color: self.text_color,
            font,
            ..Default::default()
        });
    }
}

impl Draw for record::NoErc {
    type Context<'a> = SchDrawCtx<'a>;

    fn draw<C: Canvas>(&self, canvas: &mut C, ctx: &Self::Context<'_>) {
        if self.symbol.as_ref() == "Small Cross" {
            let mut line = DrawLine {
                start: Location::new(self.location.x - GW_Q, self.location.y - GW_Q),
                end: Location::new(self.location.x + GW_Q, self.location.y + GW_Q),
                color: Rgb::red(),
                width: THIN_LINE_WIDTH,
                ..Default::default()
            };
            canvas.draw_line(line);

            line.start = Location::new(self.location.x - GW_Q, self.location.y + GW_Q);
            line.end = Location::new(self.location.x + GW_Q, self.location.y - GW_Q);
            canvas.draw_line(line);
        } else {
            warn!(
                "unrecognized no ERC marker '{}' in '{}'",
                self.symbol, ctx.name
            );
        }
    }
}

impl Draw for record::NetLabel {
    type Context<'a> = SchDrawCtx<'a>;

    fn draw<C: Canvas>(&self, canvas: &mut C, ctx: &SchDrawCtx<'_>) {
        let font = &ctx.fonts.get_idx(self.font_id.into());

        canvas.draw_text(DrawText {
            x: self.location.x,
            y: self.location.y,
            text: &self.text,
            color: self.color,
            font,
            ..Default::default()
        });
    }
}

impl Draw for record::Bus {
    type Context<'a> = SchDrawCtx<'a>;

    fn draw<C: Canvas>(&self, canvas: &mut C, _ctx: &Self::Context<'_>) {
        canvas.draw_polyline(DrawPolyLine {
            locations: &self.locations,
            color: self.color,
            width: self.line_width,
            ..Default::default()
        });
    }
}

impl Draw for record::Wire {
    type Context<'a> = SchDrawCtx<'a>;

    fn draw<C: Canvas>(&self, canvas: &mut C, _ctx: &Self::Context<'_>) {
        canvas.draw_polyline(DrawPolyLine {
            locations: &self.locations,
            color: self.color,
            width: self.line_width,
            ..Default::default()
        });
    }
}

// impl Draw for record::TextFrame {}
// impl Draw for record::Junction {}
impl Draw for record::Image {
    type Context<'a> = SchDrawCtx<'a>;

    fn draw<C: Canvas>(&self, _canvas: &mut C, _ctx: &SchDrawCtx<'_>) {
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
            x: self.location.x,
            y: self.location.y,
            text: &self.text,
            font,
            ..Default::default()
        });
    }
}

// impl Draw for record::ImplementationList {}
