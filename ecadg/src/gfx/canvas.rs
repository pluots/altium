//! Content for our main canvas
//!
//! Contains a tessellator and a text engine.

use altium::draw::{Canvas, Draw};
use eframe::{
    egui_wgpu::RenderState,
    wgpu::{Device, Queue, RenderPass},
};

use super::{tessellated::TessCtx, text::TextCtx};
use crate::backend::ViewState;

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
        self.text.clear();
        item.draw(self, item_ctx);
        self.tess.prepare(queue, vs);
        self.text.prepare(device, queue, vs);
    }

    pub fn paint(&self, render_pass: &mut RenderPass<'static>, vs: ViewState) {
        self.tess.paint(render_pass, vs);
        self.text.paint(render_pass);
    }
}

impl altium::sealed::Sealed for CanvasCtx {}

/// Pass different records off to various other renderers
impl Canvas for CanvasCtx {
    fn draw_text(&mut self, item: altium::draw::DrawText) {
        self.text.push_text(item);
    }

    fn draw_line(&mut self, item: altium::draw::DrawLine) {
        self.tess.draw_line(item);
    }

    fn draw_polyline(&mut self, item: altium::draw::DrawPolyLine) {
        self.tess.draw_polyline(item);
    }

    fn draw_arc(&mut self, item: altium::draw::DrawArc) {
        self.tess.draw_arc(item);
    }

    fn draw_polygon(&mut self, item: altium::draw::DrawPolygon) {
        self.tess.draw_polygon(item);
    }

    fn draw_rectangle(&mut self, item: altium::draw::DrawRectangle) {
        self.tess.draw_rectangle(item);
    }

    fn draw_image(&mut self, _item: altium::draw::DrawImage) {
        // todo!()
    }
}
