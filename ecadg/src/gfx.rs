//! Entrypoint for GPU rendering items

mod grid;
mod poly;
mod triangle;

// use std::sync::Arc;

use altium::sch::Component;
use eframe::egui_wgpu;
use egui::{PaintCallbackInfo, Vec2};
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
            // device: Arc::clone(device),
            triangle_ctx: triangle::TriangleCtx::init(wgpu_render_state, device),
            grid_ctx: grid::GridCtx::init(wgpu_render_state, device),
        });
}

/// Context that is created upon init and accessible via each render
struct GraphicsCtx {
    // device: Arc<wgpu::Device>,
    triangle_ctx: triangle::TriangleCtx,
    grid_ctx: grid::GridCtx,
}

/// Callback for drawing schlib items
pub struct SchLibCallback {
    scale: f32,
    center: Vec2,
    dims: Vec2,
}

impl SchLibCallback {
    /// Entrypoint for rendering a single component in a schematic library
    pub fn callback(
        rect: egui::Rect,
        _comp: &Component,
        vs: &ViewState,
        dims: Vec2,
    ) -> egui::PaintCallback {
        let cb_ctx = Self {
            scale: vs.scale,
            center: vs.center,
            dims,
        };
        egui_wgpu::Callback::new_paint_callback(rect, cb_ctx)
    }
}

impl egui_wgpu::CallbackTrait for SchLibCallback {
    fn prepare(
        &self,
        _device: &Device,
        queue: &Queue,
        _encoder: &mut CommandEncoder,
        resources: &mut CallbackResources,
    ) -> Vec<CommandBuffer> {
        let ctx: &mut GraphicsCtx = resources.get_mut().unwrap();

        ctx.triangle_ctx.prepare(queue);
        ctx.grid_ctx
            .prepare(queue, self.dims, self.scale, self.center);

        Vec::new()
    }

    fn paint<'a>(
        &'a self,
        _info: PaintCallbackInfo,
        render_pass: &mut RenderPass<'a>,
        resources: &'a CallbackResources,
    ) {
        let ctx: &GraphicsCtx = resources.get().unwrap();
        ctx.triangle_ctx.paint(render_pass);
        ctx.grid_ctx.paint(render_pass, self.scale);
    }
}
