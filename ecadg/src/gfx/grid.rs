#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]

use std::mem;

use bytemuck::{Pod, Zeroable};
use eframe::egui_wgpu::{self, wgpu, RenderState};
use egui::Vec2;
use egui_wgpu::wgpu::{Device, Queue, RenderPass};
use wgpu::util::DeviceExt;

const GRID_INDICES: usize = 4;

/// Data of window position used to determine layout
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C, align(16))]
pub struct GridUniformBuf {
    offset_mod_pct: Vec2,
    spacing_pct: Vec2,
}

impl Default for GridUniformBuf {
    fn default() -> Self {
        Self {
            // modulo of offset relative to spacing
            offset_mod_pct: Vec2::new(0.0, 0.0),
            // spacing as a percent from -1.0..1.0
            spacing_pct: Vec2 { x: 0.2, y: 0.2 },
        }
    }
}

/// Data that can be selected for each run of the pipeline
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C, align(16))]
pub struct GridInstanceBuf {
    /// Multiplier of spacing to make major and minor grids
    spacing_mult: f32,
    saturation: f32,
    /// 0 for horizontal, 1 for vertical
    is_vert: u32,
    _padding: [u32; 1],
}

impl GridInstanceBuf {
    // const MINOR_HORIZ_IDX: u8 = 0;
    // const MINOR_VERT_IDX: u8 = 1;
    // const MAJOR_HORIZ_IDX: u8 = 2;
    // const MAJOR_VERT_IDX: u8 = 3;

    const MAJOR_SPACING_MULT: f32 = 1.0;
    const MINOR_SPACING_MULT: f32 = 0.1;

    /// Horizontal & vertical for major and minor
    fn all() -> [Self; GRID_INDICES] {
        const MAJOR_SATURATION: f32 = 1.0;
        const MINOR_SATURATION: f32 = 0.4;

        // First two are minor, second two are major. We do this so major overwrites minor
        [
            Self {
                spacing_mult: Self::MINOR_SPACING_MULT,
                saturation: MINOR_SATURATION,
                is_vert: 0,
                _padding: Default::default(),
            },
            Self {
                spacing_mult: Self::MINOR_SPACING_MULT,
                saturation: MINOR_SATURATION,
                is_vert: 1,
                _padding: Default::default(),
            },
            Self {
                spacing_mult: Self::MAJOR_SPACING_MULT,
                saturation: MAJOR_SATURATION,
                is_vert: 0,
                _padding: Default::default(),
            },
            Self {
                spacing_mult: Self::MAJOR_SPACING_MULT,
                saturation: MAJOR_SATURATION,
                is_vert: 1,
                _padding: Default::default(),
            },
        ]
    }

    /// Make the vertex buffer layout for this type
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We'll have to reassemble the mat4 in the shader.
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32,
                },
                wgpu::VertexAttribute {
                    offset: 4,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32,
                },
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Uint32,
                },
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Uint32,
                },
            ],
        }
    }
}

pub struct GridCtx {
    uniform_buffer: wgpu::Buffer,
    uniform_buffer_data: GridUniformBuf,
    instance_buffer: wgpu::Buffer,
    _instance_buffer_data: [GridInstanceBuf; 4],
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

impl GridCtx {
    pub fn init(render_state: &RenderState, device: &Device) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("grid.wgsl"));
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("grid_uniform"),
            contents: bytemuck::bytes_of(&GridUniformBuf::default()), // 16 bytes aligned!
            // Mapping at creation (as done by the create_buffer_init utility) doesn't require us to to add the MAP_WRITE usage
            // (this *happens* to workaround this bug )
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let instance_buffer_data = GridInstanceBuf::all();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("grid_instance"),
            contents: bytemuck::bytes_of(&instance_buffer_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("grid_uniform_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(
                        u64::try_from(mem::size_of::<GridUniformBuf>())
                            .unwrap()
                            .try_into()
                            .unwrap(),
                    ),
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("grid_bind_group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("grid_pl_layout"),
            // bind_group_layouts: &[],
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("grid_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                // buffers: &[],
                buffers: &[GridInstanceBuf::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(render_state.target_format.into())],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self {
            uniform_buffer,
            bind_group,
            pipeline,
            instance_buffer,
            _instance_buffer_data: instance_buffer_data,
            uniform_buffer_data: GridUniformBuf::default(),
        }
    }

    /// Set up buffers to be ready to draw
    /// scale is m/px, center is in px
    pub fn prepare(&mut self, queue: &Queue, window_dims: Vec2, scale: f32, center: Vec2) {
        /// spacing for major grid in m. 10mm currently
        const MAJOR_SPACING_M: f32 = 10e-3;
        // spacing in pixels
        let sp_px = MAJOR_SPACING_M / scale;

        // spacing as percent
        // TODO: find out why these need to be reversed to get accurate results, I have no clue
        let sp_pct_x = sp_px / window_dims.y;
        let sp_pct_y = sp_px / window_dims.x;

        let offset_pct_x = (center.x / window_dims.x) % sp_pct_x;
        let offset_pct_y = (center.y / window_dims.y) % sp_pct_y;

        dbg!(offset_pct_x, sp_pct_x, offset_pct_y, sp_pct_y);

        let uniform = GridUniformBuf {
            offset_mod_pct: Vec2 {
                x: offset_pct_x,
                y: offset_pct_y,
            },
            spacing_pct: Vec2 {
                x: sp_pct_x,
                y: sp_pct_y,
            },
        };

        let buf = bytemuck::bytes_of(&uniform);
        queue.write_buffer(&self.uniform_buffer, 0, buf);

        self.uniform_buffer_data = uniform;
    }

    /// Draw needed lines
    pub fn paint<'a>(&'a self, render_pass: &mut RenderPass<'a>, scale: f32) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.instance_buffer.slice(..));

        if scale < 2e-3 {
            if scale < 200e-6 {
                // drawing minor
                let x_lines = (2.0 / self.uniform_buffer_data.spacing_pct.x).ceil()
                    / GridInstanceBuf::MINOR_SPACING_MULT;
                let y_lines = (2.0 / self.uniform_buffer_data.spacing_pct.y).ceil()
                    / GridInstanceBuf::MINOR_SPACING_MULT;
                debug_assert!(x_lines > 0.0);
                debug_assert!(y_lines > 0.0);

                render_pass.draw(0..(x_lines as u32) * 2, 0..1);
                render_pass.draw(0..(y_lines as u32) * 2, 1..2);
            }

            // drawing major
            let x_lines = (2.0 / self.uniform_buffer_data.spacing_pct.x).ceil()
                / GridInstanceBuf::MAJOR_SPACING_MULT;
            let y_lines = (2.0 / self.uniform_buffer_data.spacing_pct.y).ceil()
                / GridInstanceBuf::MAJOR_SPACING_MULT;

            debug_assert!(x_lines > 0.0);
            debug_assert!(y_lines > 0.0);
            render_pass.draw(0..(x_lines as u32) * 2, 2..3);
            render_pass.draw(0..(y_lines as u32) * 2, 3..4);
        }
    }
}
