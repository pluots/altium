use std::sync::{Arc, Mutex, MutexGuard, OnceLock, RwLock, RwLockReadGuard};

use eframe::{
    egui_wgpu::{wgpu, RenderState},
    wgpu::{Queue, RenderPass},
};
use glyphon::{
    Attrs,
    Cache,
    Family,
    FontSystem,
    Resolution,
    Shaping,
    SwashCache,
    TextArea,
    TextAtlas,
    TextRenderer,
    Viewport,
};
use wgpu::Device;

use crate::backend::ViewState;

// TODO: rwlock?
/// Global font list
static FONT_SYSTEM: OnceLock<Mutex<FontSystem>> = OnceLock::new();

fn get_font_system() -> MutexGuard<'static, FontSystem> {
    FONT_SYSTEM
        .get_or_init(|| Mutex::new(FontSystem::new()))
        .lock()
        .unwrap()
}

pub struct TextCtx {
    swash_cache: SwashCache,
    cache: Cache,
    atlas: TextAtlas,
    viewport: Viewport,
    renderer: TextRenderer,
    buffer: Arc<RwLock<glyphon::Buffer>>,
}

impl TextCtx {
    pub fn init(render_state: &RenderState, device: &Device) -> Self {
        let mut font_system = get_font_system();

        let swash_cache = SwashCache::new();
        let cache = Cache::new(&device);
        let viewport = Viewport::new(&device, &cache);
        let mut buffer = glyphon::Buffer::new(&mut font_system, glyphon::Metrics::new(30.0, 42.0));
        let mut atlas = TextAtlas::new(
            &device,
            &render_state.queue,
            &cache,
            render_state.target_format,
        );
        let renderer =
            TextRenderer::new(&mut atlas, &device, wgpu::MultisampleState::default(), None);

        buffer.set_size(&mut font_system, 500.0, 500.0);
        // buffer.set_size(&mut font_system, physical_width, physical_height);

        buffer.set_text(
            &mut font_system,
            "Hello world! 👋\nThis is rendered with 🦅 glyphon 🦁\nThe text below should be partially clipped.\na b c d e f g h i j k l m n o p q r s t u v w x y z",
            Attrs::new().family(Family::SansSerif), Shaping::Advanced);
        buffer.shape_until_scroll(&mut font_system, false);

        Self {
            swash_cache,
            cache,
            atlas,
            viewport,
            renderer,
            buffer: Arc::new(RwLock::new(buffer)),
        }
    }

    pub fn prepare<'a>(
        &mut self,
        device: &Device,
        queue: &Queue,
        vs: ViewState,
        text_areas: impl IntoIterator<Item = TextArea<'a>>,
    ) {
        // let mut viewport = dbg!(Viewport::new(&device, &self.cache));
        self.viewport.update(
            queue,
            Resolution {
                width: vs.rect.width() as u32,
                height: vs.rect.height() as u32,
            },
        );

        // dbg!(&self.viewport);
        dbg!(&self.viewport);
        self.renderer
            .prepare(
                device,
                queue,
                &mut get_font_system(),
                &mut self.atlas,
                &self.viewport,
                // &viewport,
                text_areas,
                &mut self.swash_cache,
            )
            .unwrap();

        //     glyphon_renderer
        // .prepare(
        //     device,
        //     queue,
        //     Resolution {
        //         width: screen_descriptor.size_in_pixels[0],
        //         height: screen_descriptor.size_in_pixels[1],
        //     },
        //     text_areas,
        // )
        // .unwrap();
    }

    pub fn paint<'pass>(&'pass self, render_pass: &mut RenderPass<'pass>) {
        eprintln!("PAINT TEXT");
        self.renderer
            .render(&self.atlas, &self.viewport, render_pass)
            .unwrap();
    }

    pub fn buffer(&self) -> Arc<RwLock<glyphon::Buffer>> {
        Arc::clone(&self.buffer)
    }
}
