#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]

use std::mem;

use bytemuck::{Pod, Zeroable};
use eframe::egui_wgpu::{self, wgpu, RenderState};
use egui::Vec2;
use egui_wgpu::wgpu::{Device, Queue, RenderPass};
use wgpu::util::DeviceExt;

use crate::backend::ViewState;

const ORIGIN_LENGTH_PX: f32 = 60.0;
const ORIGIN_WIDTH_PX: f32 = 2.0;

/// Data of window position used to determine layout
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
struct OriginUniformBuf {
    /// Offset of the origin from (0, 0), in GPU ranges
    offset: Vec2,
    /// Dimensions of the horizontal stroke in GPU ranges (-1.0..1.0)
    hdims: Vec2,
    /// Dimensions of the vertical stroke in GPU ranges (-1.0..1.0)
    vdims: Vec2,
}

impl Default for OriginUniformBuf {
    fn default() -> Self {
        Self {
            offset: Vec2::ZERO,
            hdims: Vec2::ZERO,
            vdims: Vec2::ZERO,
        }
    }
}

pub struct OriginCtx {
    uniform_buffer: wgpu::Buffer,
    uniform_buffer_data: OriginUniformBuf,
    instance_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

impl OriginCtx {
    pub fn init(render_state: &RenderState, device: &Device) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("origin.wgsl"));
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("origin_uniform"),
            contents: bytemuck::bytes_of(&OriginUniformBuf::default()), // 16 bytes aligned!
            // Mapping at creation (as done by the create_buffer_init utility) doesn't require us to to add the MAP_WRITE usage
            // (this *happens* to workaround this bug)
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("origin_instance"),
            contents: &[],
            usage: wgpu::BufferUsages::VERTEX,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("origin_uniform_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(
                        u64::try_from(mem::size_of::<OriginUniformBuf>())
                            .unwrap()
                            .try_into()
                            .unwrap(),
                    ),
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("origin_bind_group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("origin_pl_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("origin_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(render_state.target_format.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self {
            uniform_buffer,
            bind_group,
            pipeline,
            instance_buffer,
            uniform_buffer_data: OriginUniformBuf::default(),
        }
    }

    /// Set up buffers to be ready to draw
    pub fn prepare(&mut self, queue: &Queue, vs: ViewState) {
        let hdims = vs.px_to_gfx(Vec2::new(ORIGIN_LENGTH_PX, ORIGIN_WIDTH_PX));
        let vdims = vs.px_to_gfx(Vec2::new(ORIGIN_WIDTH_PX, ORIGIN_LENGTH_PX));
        let uniform = OriginUniformBuf {
            offset: vs.offset_gfx(),
            hdims,
            vdims,
        };

        let buf = bytemuck::bytes_of(&uniform);
        queue.write_buffer(&self.uniform_buffer, 0, buf);

        self.uniform_buffer_data = uniform;
    }

    /// Draw needed triangles
    pub fn paint<'a>(&'a self, render_pass: &mut RenderPass<'a>, _vs: ViewState) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.instance_buffer.slice(..));
        render_pass.draw(0..12, 0..1);
    }
}
