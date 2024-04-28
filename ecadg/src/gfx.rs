//! Entrypoint for GPU rendering items

mod grid;
mod origin;
mod poly;
mod tessellated;
mod triangle;

// use std::sync::Arc;

use std::sync::Arc;

use altium::sch::Component;
use eframe::egui_wgpu;
use egui::PaintCallbackInfo;
use egui_wgpu::wgpu::{CommandBuffer, CommandEncoder, Device, Queue, RenderPass};
use egui_wgpu::CallbackResources;

use crate::backend::ViewState;

// Called once
pub fn init_graphics(cc: &eframe::CreationContext<'_>) {
    let wgpu_render_state = cc
        .wgpu_render_state
        .as_ref()
        .expect("failed to prepare render state");

    let device = &wgpu_render_state.device;

    wgpu_render_state
        .renderer
        .write()
        .callback_resources
        .insert(GraphicsCtx {
            triangle: triangle::TriangleCtx::init(wgpu_render_state, device),
            grid: grid::GridCtx::init(wgpu_render_state, device),
            origin: origin::OriginCtx::init(wgpu_render_state, device),
            tess: tessellated::TessCtx::init(wgpu_render_state, device),
        });
}

/// Context that is created upon init and accessible via each render
struct GraphicsCtx {
    triangle: triangle::TriangleCtx,
    grid: grid::GridCtx,
    origin: origin::OriginCtx,
    tess: tessellated::TessCtx,
}

/// Callback for drawing schlib items
pub struct SchLibCallback {
    view_state: ViewState,
    comp: Arc<Component>,
}

impl SchLibCallback {
    /// Entrypoint for rendering a single component in a schematic library
    pub fn callback(comp: Arc<Component>, vs: &ViewState) -> egui::PaintCallback {
        let cb_ctx = Self {
            view_state: *vs,
            comp,
        };
        egui_wgpu::Callback::new_paint_callback(vs.rect, cb_ctx)
    }
}

impl egui_wgpu::CallbackTrait for SchLibCallback {
    fn prepare(
        &self,
        _device: &Device,
        queue: &Queue,
        _desc: &egui_wgpu::ScreenDescriptor,
        _encoder: &mut CommandEncoder,
        resources: &mut CallbackResources,
    ) -> Vec<CommandBuffer> {
        let ctx: &mut GraphicsCtx = resources.get_mut().unwrap();

        ctx.triangle.prepare(queue);
        ctx.grid.prepare(queue, self.view_state);
        ctx.tess
            .prepare(queue, self.view_state, self.comp.as_ref(), &());
        ctx.origin.prepare(queue, self.view_state);

        Vec::new()
    }

    fn paint<'a>(
        &'a self,
        _info: PaintCallbackInfo,
        render_pass: &mut RenderPass<'a>,
        resources: &'a CallbackResources,
    ) {
        let ctx: &GraphicsCtx = resources.get().unwrap();
        // ctx.triangle.paint(render_pass);
        ctx.grid.paint(render_pass, self.view_state);
        ctx.tess.paint(render_pass, self.view_state);
        ctx.origin.paint(render_pass, self.view_state);
    }
}
