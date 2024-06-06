//! Content for our main canvas
//!
//! Contains a tessellator and a text engine

use std::sync::atomic::*;

use altium::draw::{Canvas, Draw};
use eframe::{
    egui_wgpu::RenderState,
    wgpu::{Device, Queue, RenderPass},
};
use glyphon::{Color, TextArea, TextBounds};

use super::{tessellated::TessCtx, text::TextCtx};
use crate::backend::ViewState;
static A: AtomicU32 = AtomicU32::new(0);

pub struct CanvasCtx {
    tess: TessCtx,
    text: TextCtx,
}
impl CanvasCtx {
    pub fn init(render_state: &RenderState, device: &Device) -> Self {
        Self {
            tess: TessCtx::init(render_state, device),
            text: TextCtx::init(render_state, device),
        }
    }

    pub fn prepare<D: Draw>(
        &mut self,
        device: &Device,
        queue: &Queue,
        vs: ViewState,
        item: &D,
        item_ctx: &D::Context<'_>,
    ) {
        self.tess.clear();
        let mut draw = CanvasDraw {
            canvas: self,
            text_areas: Vec::new(),
        };

        item.draw(&mut draw, item_ctx);
        let text_areas = draw.text_areas;
        let buf = self.text.buffer();
        // dbg!(buf.read().unwrap());
        let text_areas = [
            // TextArea {
            //     buffer: &buf.read().unwrap(),
            //     left: 700.0,
            //     top: 400.0,
            //     // left: 10.0,
            //     // top: 10.0,
            //     scale: 1.0,
            //     bounds: TextBounds {
            //         left: -1234,
            //         top: -12345,
            //         right: 600,
            //         bottom: 160,
            //     },
            //     default_color: Color::rgb(255, 255, 255),
            // },
            TextArea {
                buffer: &buf.read().unwrap(),
                left: 0.0,
                top: 0.0,
                // left: 10.0,
                // top: 10.0,
                scale: 1.0,
                bounds: TextBounds {
                    left: 0,
                    top: 0,
                    right: 132450,
                    bottom: 12345,
                },
                default_color: Color::rgb(255, 255, 255),
            },
            // TextArea {
            //     buffer: &buf.read().unwrap(),
            //     left: 700.0,
            //     top: 400.0,
            //     // left: 10.0,
            //     // top: 10.0,
            //     scale: 1.0,
            //     bounds: TextBounds {
            //         left: 0,
            //         top: 0,
            //         right: 600,
            //         bottom: 160,
            //     },
            //     default_color: Color::rgb(255, 255, 255),
            // },
            // TextArea {
            //     buffer: &buf.read().unwrap(),
            //     left: 700.0,
            //     top: 400.0,
            //     // left: 10.0,
            //     // top: 10.0,
            //     scale: 1.0,
            //     bounds: TextBounds {
            //         left: 0,
            //         top: 0,
            //         right: 600,
            //         bottom: 160,
            //     },
            //     default_color: Color::rgb(255, 255, 255),
            // },
        ];

        self.tess.prepare(queue, vs);
        self.text.prepare(device, queue, vs, text_areas);
    }

    pub fn paint<'a>(&'a self, render_pass: &mut RenderPass<'a>, vs: ViewState) {
        self.tess.paint(render_pass, vs);
        self.text.paint(render_pass);
    }
}

/// Add context to our canvas
struct CanvasDraw<'c, 't> {
    canvas: &'c mut CanvasCtx,
    text_areas: Vec<TextArea<'t>>,
}

impl altium::sealed::Sealed for CanvasDraw<'_, '_> {}

/// Pass different records off to various other renderers
impl Canvas for CanvasDraw<'_, '_> {
    fn draw_text(&mut self, _item: altium::draw::DrawText) {
        // self.text_ctx.p

        // todo!()
    }

    fn draw_line(&mut self, item: altium::draw::DrawLine) {
        self.canvas.tess.draw_line(item);
    }

    fn draw_polyline(&mut self, item: altium::draw::DrawPolyLine) {
        self.canvas.tess.draw_polyline(item);
    }

    fn draw_arc(&mut self, item: altium::draw::DrawArc) {
        self.canvas.tess.draw_arc(item);
    }

    fn draw_polygon(&mut self, item: altium::draw::DrawPolygon) {
        self.canvas.tess.draw_polygon(item);
    }

    fn draw_rectangle(&mut self, item: altium::draw::DrawRectangle) {
        self.canvas.tess.draw_rectangle(item);
    }

    fn draw_image(&mut self, _item: altium::draw::DrawImage) {
        // todo!()
    }
}
