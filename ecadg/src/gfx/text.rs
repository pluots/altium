use eframe::{
    egui_wgpu::{RenderState, ScreenDescriptor},
    wgpu::{Device, MultisampleState, Queue, RenderPass},
};
use glyphon::{
    Attrs,
    Cache,
    Color,
    ColorMode,
    Family,
    FontSystem,
    Metrics,
    Resolution,
    Shaping,
    SwashCache,
    TextArea,
    TextAtlas,
    TextBounds,
    TextRenderer,
    Viewport,
};

use crate::backend::ViewState;

pub struct TextCtx {
    // device: wgpu::Device,
    // queue: wgpu::Queue,
    // surface: wgpu::Surface<'static>,
    // surface_config: SurfaceConfiguration,
    // Arc?
    font_system: FontSystem,
    swash_cache: SwashCache,
    viewport: Viewport,
    atlas: TextAtlas,
    text_renderer: TextRenderer,
    text_buffer: glyphon::Buffer,
    // // Make sure that the winit window is last in the struct so that
    // // it is dropped after the wgpu surface is dropped, otherwise the
    // // program may crash when closed. This is probably a bug in wgpu.
    // window: Arc<Window>,
}

impl TextCtx {
    pub fn init(render_state: &RenderState, device: &Device) -> Self {
        let queue = &render_state.queue;
        // device.
        // create_render_pipeline(desc)
        // device.create_sur
        // Set up text renderer
        let mut font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = Cache::new(device);
        let viewport = Viewport::new(device, &cache);
        // let mut atlas = TextAtlas::new(device, queue, &cache, render_state.target_format);
        let mut atlas = TextAtlas::with_color_mode(
            device,
            queue,
            &cache,
            render_state.target_format,
            ColorMode::Web,
        );
        let text_renderer =
            TextRenderer::new(&mut atlas, &device, MultisampleState::default(), None);
        let text_buffer = glyphon::Buffer::new(&mut font_system, Metrics::new(30.0, 42.0));

        Self {
            font_system,
            swash_cache,
            viewport,
            atlas,
            text_renderer,
            text_buffer,
        }
    }

    // pub
    /// Set up buffers to be ready to draw
    pub fn prepare(&mut self, device: &Device, queue: &Queue, desc: &ScreenDescriptor) {
        self.atlas.trim();
        // let physical_width = (physical_size.width as f64 * scale_factor) as f32;
        // let physical_height = (physical_size.height as f64 * scale_factor) as f32;

        self.text_buffer.set_size(
            &mut self.font_system,
            None,
            None, // Some(physical_width),
                  // Some(physical_height),
                  // Some()
        );
        self.text_buffer.set_text(
            &mut self.font_system,
            "Hello world! 👋\nThis is rendered with 🦅 glyphon 🦁\nThe text below \
            should be partially clipped.\na b c d e f g h i j k l m n o p q r s t u v w x y z",
            &Attrs::new().family(Family::SansSerif),
            Shaping::Advanced,
            None,
        );
        self.text_buffer
            .shape_until_scroll(&mut self.font_system, false);

        self.viewport.update(
            queue,
            Resolution {
                width: desc.size_in_pixels[0],
                height: desc.size_in_pixels[1],
            },
        );
        self.text_renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                // text_areas,
                [TextArea {
                    buffer: &self.text_buffer,
                    left: 10.0,
                    top: 10.0,
                    scale: 1.0,
                    bounds: TextBounds {
                        left: 0,
                        top: 0,
                        right: 600,
                        bottom: 160,
                    },
                    default_color: Color::rgb(255, 255, 255),
                    custom_glyphs: &[],
                }],
                &mut self.swash_cache,
            )
            .unwrap();

        // let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });

        // let hdims = vs.px_to_gfx(Vec2::new(ORIGIN_LENGTH_PX, ORIGIN_WIDTH_PX));
        // let vdims = vs.px_to_gfx(Vec2::new(ORIGIN_WIDTH_PX, ORIGIN_LENGTH_PX));
        // let uniform = OriginUniformBuf {
        //     offset: vs.offset_gfx(),
        //     hdims,
        //     vdims,
        // };

        // let buf = bytemuck::bytes_of(&uniform);
        // queue.write_buffer(&self.uniform_buffer, 0, buf);

        // self.uniform_buffer_data = uniform;
    }

    /// Draw needed triangles
    pub fn paint(&self, render_pass: &mut RenderPass<'static>, _vs: ViewState) {
        self.text_renderer
            .render(&self.atlas, &self.viewport, render_pass)
            .unwrap();

        // render_pass.set_pipeline(&self.pipeline);
        // render_pass.set_bind_group(0, &self.bind_group, &[]);
        // render_pass.draw(0..12, 0..1);
    }
}
