#![allow(dead_code, unused_imports)]
#![allow(clippy::similar_names)]

use std::mem;

use alt_to_lyon::ToLyonTy;
use altium::draw::{Canvas, Draw};
use bytemuck::{Pod, Zeroable};
use eframe::egui_wgpu::{self, wgpu, RenderState};
use egui::Vec2;
use egui_wgpu::wgpu::{Device, Queue, RenderPass};
use log::{debug, info};
use lyon::{
    geom::{euclid::Point2D, Box2D},
    math::{point, vector, Angle},
    path::{traits::SvgPathBuilder, Path},
    tessellation::{
        BuffersBuilder,
        FillOptions,
        FillTessellator,
        FillVertex,
        FillVertexConstructor,
        StrokeOptions,
        StrokeTessellator,
        StrokeVertex,
        StrokeVertexConstructor,
        VertexBuffers,
    },
};

use super::text::TextCtx;
// use wgpu::util::DeviceExt;
use crate::backend::{loc_to_p2d, v_to_p2d, ViewState, M_PER_NM, NM_PER_M};

/// Number of samples for anti-aliasing. Set to 1 to disable
// TODO
const SAMPLE_COUNT: u32 = 1;
// const SAMPLE_COUNT: u32 = 4;
const PRIM_BUFFER_LEN: usize = 256;

const TOLERANCE: f32 = 0.0002;
const FILL_OPTIONS: FillOptions = FillOptions::DEFAULT.with_tolerance(TOLERANCE);
// const DEFAULT_STROKE
/// Stroke options, in world coordinates
const STROKE_OPTIONS: StrokeOptions = StrokeOptions::DEFAULT
    .with_line_width(10000.0)
    .with_line_cap(lyon::path::LineCap::Square)
    .with_tolerance(100.0);
const DEFAULT_BUFFER_LEN: u64 = 1024 * 1024;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Globals {
    resolution: [f32; 2],
    scroll_offset: [f32; 2],
    scale: f32,
    _pad: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Primitive {
    color: [f32; 4],
    translate: [f32; 2],
    z_index: i32,
    width: f32,
    angle: f32,
    scale: f32,
    _pad: [f32; 2],
}

impl Default for Primitive {
    fn default() -> Self {
        Self {
            color: [0.0; 4],
            translate: [0.0; 2],
            z_index: 0,
            width: 0.0,
            angle: 0.0,
            scale: 1.0,
            _pad: [0.0; 2],
        }
    }
}

/// Instance buffer for tesselation
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Pod, Zeroable)]
struct TessVertex {
    position: [f32; 2],
    /// Offset direction from the position, if needed
    normal: [f32; 2],
    color: [f32; 4],
    /// Multiplier by the offser
    stroke_width: f32,
}

impl TessVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 0x8,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 0x10,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 0x20,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        }
    }
}

pub struct TessCtx {
    fill_vbo: wgpu::Buffer,
    stroke_vbo: wgpu::Buffer,
    fill_ibo: wgpu::Buffer,
    stroke_ibo: wgpu::Buffer,
    fill_geometry: VertexBuffers<TessVertex, u16>,
    stroke_geometry: VertexBuffers<TessVertex, u16>,
    /// Quantities to actually write if we need to pad them
    prims_ubo: wgpu::Buffer,
    globals_ubo: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
    /// Primitives as calculated by the CPU
    cpu_primitives: Box<[Primitive]>,
    fill_tess: FillTessellator,
    stroke_tess: StrokeTessellator,
    view_state: ViewState,
}

impl TessCtx {
    pub fn init(render_state: &RenderState, device: &Device) -> Self {
        debug!("init tessellation shader");
        let shader = device.create_shader_module(wgpu::include_wgsl!("tessellated.wgsl"));
        let fill_geometry: VertexBuffers<TessVertex, u16> = VertexBuffers::new();
        let stroke_geometry: VertexBuffers<TessVertex, u16> = VertexBuffers::new();

        let globals_buffer_byte_size = mem::size_of::<Globals>() as u64;
        let prim_buffer_byte_size = (PRIM_BUFFER_LEN * mem::size_of::<Primitive>()) as u64;

        let fill_vbo = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("tess_fill_vbo"),
            size: DEFAULT_BUFFER_LEN,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let fill_ibo = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("tess_fill_ibo"),
            size: DEFAULT_BUFFER_LEN,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let stroke_vbo = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("tess_stroke_vbo"),
            size: DEFAULT_BUFFER_LEN,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let stroke_ibo = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("tess_stroke_ibo"),
            size: DEFAULT_BUFFER_LEN,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let prims_ubo = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("tess_prims_uniform_buff"),
            size: prim_buffer_byte_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let globals_ubo = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("tess_globals_uniform_buff"),
            size: globals_buffer_byte_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("tess_uniform_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(globals_buffer_byte_size),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(prim_buffer_byte_size),
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("tess_bind_group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(globals_ubo.as_entire_buffer_binding()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(prims_ubo.as_entire_buffer_binding()),
                },
            ],
        });

        let _depth_stencil_state = Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Greater,
            stencil: wgpu::StencilState {
                front: wgpu::StencilFaceState::IGNORE,
                back: wgpu::StencilFaceState::IGNORE,
                read_mask: 0,
                write_mask: 0,
            },
            bias: wgpu::DepthBiasState::default(),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("tess_pl_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("tess_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[TessVertex::desc()],
                // buffers: &[TessVertex::desc(), TessVertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                // targets: &[Some(wgpu::ColorTargetState {
                //     format: wgpu::TextureFormat::Bgra8Unorm,
                //     blend: None,
                //     write_mask: wgpu::ColorWrites::ALL,
                // })],
                targets: &[Some(render_state.target_format.into())],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                // topology: wgpu::PrimitiveTopology::LineList,
                topology: wgpu::PrimitiveTopology::TriangleList,
                polygon_mode: wgpu::PolygonMode::Fill,
                front_face: wgpu::FrontFace::Ccw,
                // cull_mode: Some(wgpu::Face::Back),
                ..wgpu::PrimitiveState::default()
            },
            depth_stencil: None,
            // depth_stencil: depth_stencil_state,
            multisample: wgpu::MultisampleState {
                count: SAMPLE_COUNT,
                mask: u64::MAX,
                alpha_to_coverage_enabled: false,
            },
            // multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self {
            fill_vbo,
            stroke_vbo,
            fill_ibo,
            stroke_ibo,
            fill_geometry,
            stroke_geometry,
            prims_ubo,
            globals_ubo,
            bind_group,
            pipeline,
            cpu_primitives: [Primitive::default(); PRIM_BUFFER_LEN].into(),
            fill_tess: FillTessellator::new(),
            stroke_tess: StrokeTessellator::new(),
            view_state: ViewState::default(),
        }
    }

    /// True if we need to update the fill buffer and run its shaders
    fn render_fill(&self) -> bool {
        debug_assert!(
            !(self.fill_geometry.indices.is_empty() ^ self.fill_geometry.vertices.is_empty())
        );

        !self.fill_geometry.indices.is_empty()
    }

    /// True if we need to update the stroke buffer and run its shaders
    fn render_stroke(&self) -> bool {
        debug_assert!(
            !(self.stroke_geometry.indices.is_empty() ^ self.stroke_geometry.vertices.is_empty())
        );

        !self.stroke_geometry.indices.is_empty()
    }

    fn needs_render(&self) -> bool {
        self.render_fill() || self.render_stroke()
    }

    /// Clear the buffers for redraw. Must be called before calling [`prepare`]!
    pub fn clear(&mut self) {
        self.fill_geometry.clear();
        self.stroke_geometry.clear();
    }

    /// Set up buffers to be ready to draw
    pub fn prepare(&mut self, queue: &Queue, vs: ViewState) {
        self.view_state = vs;

        if !self.needs_render() {
            debug!("skipping tessellation prepare");
            return;
        }

        let uniform = Globals {
            resolution: [vs.rect.width(), vs.rect.height()],
            scale: vs.scale,
            scroll_offset: vs.offset.into(),
            _pad: 0.0,
        };

        queue.write_buffer(&self.globals_ubo, 0, bytemuck::bytes_of(&uniform));
        queue.write_buffer(
            &self.prims_ubo,
            0,
            bytemuck::cast_slice(&self.cpu_primitives),
        );

        if self.render_fill() {
            with_aligned_buf(&mut self.fill_geometry.vertices, |buf| {
                queue.write_buffer(&self.fill_vbo, 0, buf);
            });
            with_aligned_buf(&mut self.fill_geometry.indices, |buf| {
                queue.write_buffer(&self.fill_ibo, 0, buf);
            });
        }

        if self.render_stroke() {
            with_aligned_buf(&mut self.stroke_geometry.vertices, |buf| {
                queue.write_buffer(&self.stroke_vbo, 0, buf);
            });
            with_aligned_buf(&mut self.stroke_geometry.indices, |buf| {
                queue.write_buffer(&self.stroke_ibo, 0, buf);
            });
        }
    }

    /// Draw needed triangles
    pub fn paint<'a>(&'a self, render_pass: &mut RenderPass<'a>, _vs: ViewState) {
        if !self.needs_render() {
            debug!("skipping tessellation paint");
            return;
        }

        render_pass.insert_debug_marker("debug1");
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.insert_debug_marker("debug2");

        if self.render_fill() {
            render_pass.insert_debug_marker("debug fill");
            render_pass.set_vertex_buffer(0, self.fill_vbo.slice(..));
            render_pass.set_index_buffer(self.fill_ibo.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(
                0..self.fill_geometry.indices.len().try_into().unwrap(),
                0,
                0..1,
            );
        }

        if self.render_stroke() {
            render_pass.insert_debug_marker("debug stroke");
            render_pass.set_vertex_buffer(0, self.stroke_vbo.slice(..));
            render_pass.set_index_buffer(self.stroke_ibo.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(
                0..self.stroke_geometry.indices.len().try_into().unwrap(),
                0,
                0..1,
            );
        }
    }
}

/// Draw methods
impl TessCtx {
    pub fn draw_line(&mut self, item: altium::draw::DrawLine) {
        self.draw_polyline(altium::draw::DrawPolyLine {
            locations: &[item.start, item.end],
            color: item.color,
            width: item.width,
            start_cap: item.start_cap,
            end_cap: item.end_cap,
            line_join: item.line_join,
        });
    }

    pub fn draw_polyline(&mut self, item: altium::draw::DrawPolyLine) {
        let mut builder = Path::builder();
        let Some(first) = item.locations.first() else {
            return;
        };

        builder.begin(point(first.x_f32(), first.y_f32()));

        for node in &item.locations[1..] {
            builder.line_to(point(node.x_f32(), node.y_f32()));
        }

        builder.end(false);
        let path = builder.build();

        self.stroke_tess
            .tessellate_path(
                &path,
                &STROKE_OPTIONS
                    .with_line_width(item.width as f32)
                    .with_start_cap(item.start_cap.to_lyon_ty())
                    .with_end_cap(item.end_cap.to_lyon_ty())
                    .with_line_join(item.line_join.to_lyon_ty()),
                &mut BuffersBuilder::new(
                    &mut self.stroke_geometry,
                    WithColor(item.color.as_float_rgba()),
                ),
            )
            .unwrap();
    }

    pub fn draw_arc(&mut self, _item: altium::draw::DrawArc) {
        // TODO: figure out <https://github.com/nical/lyon/issues/909>
        // let mut builder = Path::builder().with_svg();
        // // builder.move_to(loc_to_p2d(item.center));
        // // builder.arc_to(
        // //     vector(500000.0, 500000.0),
        // //     , 0.0, , )
        // dbg!(&item);

        // builder.arc(
        //     // lyon::math::point(1000.0, 1000.0),
        //     loc_to_p2d(item.center),
        //     // vector(item.x_radius as f32, item.y_radius as f32) * 100.0,
        //     vector(500000.0, 500000.0),
        //     Angle::radians(std::f32::consts::FRAC_PI_4),
        //     Angle::radians(std::f32::consts::FRAC_PI_4 * 2.0),
        //     // Angle::radians(std::f32::consts::),
        //     // Angle::radians(item.end_angle - item.start_angle),
        //     // Angle::radians(item.end_angle),
        //     // Angle::radians(item.start_angle),
        //     // Angle::radians(item.end_angle),
        //     // Angle::radians(item.start_angle + 3.0 * std::f32::consts::FRAC_PI_4),
        // );

        // let path = builder.build();

        // self.stroke_tess
        //     .tessellate_path(
        //         &path,
        //         &STROKE_OPTIONS
        //             .with_line_width(10000.0)
        //             // .with_line_width(item.width as f32 / 4.0)
        //             // .with_line_width(item.width as f32)
        //             .with_start_cap(lyon::path::LineCap::Square)
        //             .with_end_cap(lyon::path::LineCap::Round)
        //             // .with_start_cap(item.start_cap.to_lyon_ty())
        //             // .with_end_cap(item.end_cap.to_lyon_ty())
        //             .with_line_join(item.line_join.to_lyon_ty()),
        //         &mut BuffersBuilder::new(
        //             &mut self.stroke_geometry,
        //             WithColor(item.color.as_float_rgba()),
        //         ),
        //     )
        //     .unwrap();
    }

    pub fn draw_polygon(&mut self, item: altium::draw::DrawPolygon) {
        let Some((first_loc, locations)) = item.locations.split_first() else {
            return;
        };

        let mut builder = Path::builder();
        builder.begin(point(first_loc.x_f32(), first_loc.y_f32()));
        for loc in locations {
            builder.line_to(point(loc.x_f32(), loc.y_f32()));
        }

        builder.close();
        let path = builder.build();

        self.fill_tess
            .tessellate_path(
                &path,
                &FILL_OPTIONS,
                &mut BuffersBuilder::new(
                    &mut self.fill_geometry,
                    WithColor(item.fill_color.as_float_rgba()),
                ),
            )
            .unwrap();

        self.stroke_tess
            .tessellate_path(
                &path,
                &STROKE_OPTIONS
                    .with_line_width(item.stroke_width as f32)
                    .with_start_cap(item.start_cap.to_lyon_ty())
                    .with_end_cap(item.end_cap.to_lyon_ty())
                    .with_line_join(item.line_join.to_lyon_ty()),
                &mut BuffersBuilder::new(
                    &mut self.stroke_geometry,
                    WithColor(item.stroke_color.as_float_rgba()),
                ),
            )
            .unwrap();
    }

    pub fn draw_rectangle(&mut self, item: altium::draw::DrawRectangle) {
        let min_x = item.x as f32;
        let min_y = item.y as f32;
        let max_x = min_x + item.width as f32;
        let max_y = min_y + item.height as f32;
        let rect = Box2D::new(Point2D::new(min_x, min_y), Point2D::new(max_x, max_y));

        self.fill_tess
            .tessellate_rectangle(
                &rect,
                &FILL_OPTIONS,
                &mut BuffersBuilder::new(
                    &mut self.fill_geometry,
                    WithColor(item.fill_color.as_float_rgba()),
                ),
            )
            .unwrap();

        self.stroke_tess
            .tessellate_rectangle(
                &rect,
                &STROKE_OPTIONS
                    .with_line_width(item.stroke_width as f32)
                    .with_start_cap(item.start_cap.to_lyon_ty())
                    .with_end_cap(item.end_cap.to_lyon_ty())
                    .with_line_join(item.line_join.to_lyon_ty()),
                &mut BuffersBuilder::new(
                    &mut self.stroke_geometry,
                    WithColor(item.stroke_color.as_float_rgba()),
                ),
            )
            .unwrap();
    }
}

/// This vertex constructor forwards the positions and normals provided by the
/// tessellators and add a shape id.
pub struct WithColor([f32; 4]);

impl FillVertexConstructor<TessVertex> for WithColor {
    fn new_vertex(&mut self, vertex: FillVertex) -> TessVertex {
        TessVertex {
            position: vertex.position().to_array(),
            normal: [0.0, 0.0],
            // prim_id: self.0,
            color: self.0,
            stroke_width: 1.0,
        }
    }
}

impl StrokeVertexConstructor<TessVertex> for WithColor {
    fn new_vertex(&mut self, vertex: StrokeVertex) -> TessVertex {
        TessVertex {
            position: vertex.position_on_path().to_array(),
            normal: vertex.normal().to_array(),
            // prim_id: self.0,
            color: self.0,
            stroke_width: vertex.line_width(),
        }
    }
}

/// Temporarily extend a buffer to be a multiple of [`wgpu::COPY_BUFFER_ALIGNMENT`] for use
/// as a slice
///
/// This is used because wgpu requires a buffer aligned to `COPY_BUFFER_ALIGNMENT` but we don't
/// want to zero-pad our tessellation buffers then forget about it (and wind up with extra
/// vertices / indices). So provide an aligned buffer only within the scope of a closure.
fn with_aligned_buf<T, F>(buf: &mut Vec<T>, f: F)
where
    T: Default + bytemuck::NoUninit,
    F: FnOnce(&[u8]),
{
    let t_size: wgpu::BufferAddress = std::mem::size_of::<T>().try_into().unwrap();
    let len: wgpu::BufferAddress = buf.len().try_into().unwrap();
    let len_bytes = len * t_size;
    // Next value that will meet the alignment
    let target_len_bytes = len_bytes.next_multiple_of(wgpu::COPY_BUFFER_ALIGNMENT);
    let to_add = (target_len_bytes - len_bytes).div_ceil(t_size);

    // push temporary elements to meet the needed alignment
    for _ in 0..to_add {
        buf.push(T::default());
    }

    f(bytemuck::cast_slice(buf));

    // Remove the temporary elements so we can continue appending to the buffer later
    buf.truncate(buf.len() - usize::try_from(to_add).unwrap());
}

mod alt_to_lyon {
    pub trait ToLyonTy<LyonTy> {
        fn to_lyon_ty(&self) -> LyonTy;
    }

    impl ToLyonTy<lyon::path::LineCap> for altium::draw::LineCap {
        fn to_lyon_ty(&self) -> lyon::path::LineCap {
            use altium::draw::LineCap;
            match self {
                LineCap::Butt => lyon::path::LineCap::Butt,
                LineCap::Square => lyon::path::LineCap::Square,
                LineCap::Round => lyon::path::LineCap::Round,
            }
        }
    }

    impl ToLyonTy<lyon::path::LineJoin> for altium::draw::LineJoin {
        fn to_lyon_ty(&self) -> lyon::path::LineJoin {
            use altium::draw::LineJoin;
            match self {
                LineJoin::Miter => lyon::path::LineJoin::Miter,
                LineJoin::MiterClip => lyon::path::LineJoin::MiterClip,
                LineJoin::Round => lyon::path::LineJoin::Round,
                LineJoin::Bevel => lyon::path::LineJoin::Bevel,
            }
        }
    }
}
