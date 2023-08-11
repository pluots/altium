//! GPU rendering

use std::num::NonZeroU64;
use std::sync::Arc;

use altium::sch::Component;
use eframe::egui_wgpu::{self, wgpu};
use egui::{Context, PaintCallbackInfo};
use egui_wgpu::wgpu::{CommandBuffer, CommandEncoder, Device, Queue, RenderPass};
use egui_wgpu::CallbackResources;
use wgpu::util::DeviceExt;

use crate::backend::ViewState;

/// Random shader copied from a tutorial, does not yet work
const SHADER_TMP: &str = r#"
struct VertexOut {
    @location(0) color: vec4<f32>,
    @builtin(position) position: vec4<f32>,
};

struct Uniforms {
    @size(16) angle: f32, // pad to 16 bytes
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

var<private> v_positions: array<vec2<f32>, 3> = array<vec2<f32>, 3>(
    vec2<f32>(0.0, 1.0),
    vec2<f32>(1.0, -1.0),
    vec2<f32>(-1.0, -1.0),
);

var<private> v_colors: array<vec4<f32>, 3> = array<vec4<f32>, 3>(
    vec4<f32>(1.0, 0.0, 0.0, 1.0),
    vec4<f32>(0.0, 1.0, 0.0, 1.0),
    vec4<f32>(0.0, 0.0, 1.0, 1.0),
);

@vertex
fn vs_main(@builtin(vertex_index) v_idx: u32) -> VertexOut {
    var out: VertexOut;

    out.position = vec4<f32>(v_positions[v_idx], 0.0, 1.0);
    out.position.x = out.position.x * cos(uniforms.angle);
    out.color = v_colors[v_idx];

    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    return in.color;
}
"#;

const SHADER: &str = include_str!("../gsl/background.wgsl");

// Called once
pub fn init(cc: &eframe::CreationContext<'_>) {
    let wgpu_render_state = cc
        .wgpu_render_state
        .as_ref()
        .expect("failed to prepare render state");

    let device = &wgpu_render_state.device;

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("shader1"),
        source: wgpu::ShaderSource::Wgsl(SHADER.into()),
    });

    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("uniform_buffer1"),
        contents: bytemuck::cast_slice(&[0.0_f32; 4]), // 16 bytes aligned!
        // Mapping at creation (as done by the create_buffer_init utility) doesn't require us to to add the MAP_WRITE usage
        // (this *happens* to workaround this bug )
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
    });

    // let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
    //     label: Some("bindgrouplayout"),
    //     entries: &[wgpu::BindGroupLayoutEntry {
    //         binding: 0,
    //         visibility: wgpu::ShaderStages::VERTEX,
    //         ty: wgpu::BindingType::Buffer {
    //             ty: wgpu::BufferBindingType::Uniform,
    //             has_dynamic_offset: false,
    //             min_binding_size: NonZeroU64::new(16),
    //         },
    //         count: None,
    //     }],
    // });

    // let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
    //     label: Some("bindgroup"),
    //     layout: &bind_group_layout,
    //     entries: &[wgpu::BindGroupEntry {
    //         binding: 0,
    //         resource: uniform_buffer.as_entire_binding(),
    //     }],
    // });

    let texture_size = wgpu::Extent3d {
        width: 1,
        height: 1,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("maintexture"),
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Bgra8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::RENDER_ATTACHMENT,
        // usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("mainlayout"),
        bind_group_layouts: &[/* &bind_group_layout */],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("mainpipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu_render_state.target_format.into())],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    wgpu_render_state
        .renderer
        .write()
        .callback_resources
        .insert(WindowCtx {
            device: Arc::clone(device),
            // bind_group,
            uniform_buffer,
            texture,
            pipeline,
        });
}

/// Context that is created upon init and accessible via each render
struct WindowCtx {
    device: Arc<wgpu::Device>,
    // pipeline: wgpu::RenderPipeline,
    // bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    texture: wgpu::Texture,
    pipeline: wgpu::RenderPipeline,
}

/// Callback for drawing schlib items
struct SchLibCallback {}

impl egui_wgpu::CallbackTrait for SchLibCallback {
    fn prepare(
        &self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        resources: &mut CallbackResources,
    ) -> Vec<CommandBuffer> {
        let ctx: &WindowCtx = resources.get().unwrap();
        // let view = ctx
        //     .texture
        //     .create_view(&wgpu::TextureViewDescriptor::default());

        // let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        //     label: Some("background"),
        //     color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        //         view: &view,
        //         resolve_target: None,
        //         ops: wgpu::Operations {
        //             load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
        //             store: wgpu::StoreOp::Store,
        //         },
        //     })],
        //     depth_stencil_attachment: None,
        //     ..Default::default()
        // });

        // rpass.set_pipeline(&ctx.pipeline);
        // rpass.draw(0..3, 0..1);
        queue.write_buffer(
            &ctx.uniform_buffer,
            0,
            bytemuck::cast_slice(&[4.0f32, 0.0, 0.0, 0.0]),
        );
        // queue.write_texture(
        //     wgpu::ImageCopyTexture {
        //         texture: &ctx.texture,
        //         mip_level: 0,
        //         origin: wgpu::Origin3d::default(),
        //         aspect: wgpu::TextureAspect::All,
        //     },
        //     &[100u8; 4],
        //     wgpu::ImageDataLayout {
        //         offset: 0,
        //         bytes_per_row: Some(32),
        //         rows_per_image: None,
        //     },
        //     wgpu::Extent3d {
        //         width: 1,
        //         height: 1,
        //         depth_or_array_layers: 1,
        //     },
        // );

        Vec::new()
    }

    fn paint<'a>(
        &'a self,
        info: PaintCallbackInfo,
        render_pass: &mut RenderPass<'a>,
        resources: &'a CallbackResources,
    ) {
        let ctx: &WindowCtx = resources.get().unwrap();
        render_pass.set_pipeline(&ctx.pipeline);
        // render_passc
        render_pass.draw(0..3, 0..1);
        // todo!()
    }
}

/// Entrypoint for schlib rendering
pub fn paint_schlib_component(ui: &mut egui::Ui, comp: &Component, vs: ViewState) {
    let (rect, response) =
        ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());

    ui.painter().add(egui_wgpu::Callback::new_paint_callback(
        rect,
        SchLibCallback {},
    ));
}
