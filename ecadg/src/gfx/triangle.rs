#![allow(unused)]
//! Just a demo triangle

use std::num::NonZeroU64;

use eframe::egui_wgpu::{self, wgpu, RenderState};
use egui_wgpu::wgpu::{Device, Queue, RenderPass};
use wgpu::util::DeviceExt;

pub struct TriangleCtx {
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

impl TriangleCtx {
    pub fn init(render_state: &RenderState, device: &Device) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("triangle.wgsl"));
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("triangle_ubuf"),
            contents: bytemuck::cast_slice(&[0.0_f32; 4]), // 16 bytes aligned!
            // Mapping at creation (as done by the create_buffer_init utility) doesn't require us to to add the MAP_WRITE usage
            // (this *happens* to workaround this bug )
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("triangle_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: NonZeroU64::new(16),
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("triangle_bind_group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("triangle_pl_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("triangle_pipeline"),
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
        }
    }

    pub fn prepare(&self, queue: &Queue) {
        let tri_buf = bytemuck::cast_slice(&[1.0f32, 0.0, 0.0, 0.0]);
        queue.write_buffer(&self.uniform_buffer, 0, tri_buf);
    }

    pub fn paint<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}
