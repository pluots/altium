//! Entrypoint for GPU rendering items

mod canvas;
mod grid;
mod origin;
mod poly;
mod tessellated;
mod text;
mod triangle;

use std::sync::Arc;

use altium::font::FontCollection;
use altium::sch::{self, Component, SchDrawCtx, SchRecord};
use eframe::egui_wgpu;
use egui::PaintCallbackInfo;
use egui_wgpu::wgpu::{CommandBuffer, CommandEncoder, Device, Queue, RenderPass};
use egui_wgpu::CallbackResources;

use crate::backend::{SchDocTab, ViewState};

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
            canvas: canvas::CanvasCtx::init(wgpu_render_state, device),
        });
}

/// Context that is created upon init and accessible via each render
struct GraphicsCtx {
    #[allow(dead_code)]
    triangle: triangle::TriangleCtx,
    grid: grid::GridCtx,
    origin: origin::OriginCtx,
    canvas: canvas::CanvasCtx,
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
        device: &Device,
        queue: &Queue,
        _desc: &egui_wgpu::ScreenDescriptor,
        _encoder: &mut CommandEncoder,
        resources: &mut CallbackResources,
    ) -> Vec<CommandBuffer> {
        let ctx: &mut GraphicsCtx = resources.get_mut().unwrap();

        ctx.grid.prepare(queue, self.view_state);
        ctx.canvas
            .prepare(device, queue, self.view_state, self.comp.as_ref(), &());
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

        ctx.grid.paint(render_pass, self.view_state);
        ctx.canvas.paint(render_pass, self.view_state);
        ctx.origin.paint(render_pass, self.view_state);
    }
}

/// Callback for drawing schlib items
pub struct SchDocCallback {
    view_state: ViewState,
    records: Arc<[SchRecord]>,
    storage: Arc<sch::Storage>,
    fonts: Arc<FontCollection>,
    name: Arc<str>,
}

impl SchDocCallback {
    /// Entrypoint for rendering a single component in a schematic library
    pub fn callback(tab: &SchDocTab, vs: &ViewState) -> egui::PaintCallback {
        let cb_ctx = Self {
            view_state: *vs,
            records: Arc::clone(&tab.records),
            fonts: Arc::clone(&tab.fonts),
            storage: Arc::clone(&tab.storage),
            name: Arc::clone(&tab.name),
        };
        egui_wgpu::Callback::new_paint_callback(vs.rect, cb_ctx)
    }
}

impl egui_wgpu::CallbackTrait for SchDocCallback {
    fn prepare(
        &self,
        device: &Device,
        queue: &Queue,
        _desc: &egui_wgpu::ScreenDescriptor,
        _encoder: &mut CommandEncoder,
        resources: &mut CallbackResources,
    ) -> Vec<CommandBuffer> {
        let ctx: &mut GraphicsCtx = resources.get_mut().unwrap();

        let sch_ctx = SchDrawCtx {
            fonts: &self.fonts,
            storage: &self.storage,
            name: &self.name,
        };

        ctx.grid.prepare(queue, self.view_state);
        ctx.canvas.prepare(
            device,
            queue,
            self.view_state,
            &self.records.as_ref(),
            &sch_ctx,
        );
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

        ctx.grid.paint(render_pass, self.view_state);
        ctx.canvas.paint(render_pass, self.view_state);
        ctx.origin.paint(render_pass, self.view_state);
    }
}
