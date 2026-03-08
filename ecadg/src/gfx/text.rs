use std::sync::{Arc, Mutex, MutexGuard, OnceLock, RwLock, RwLockReadGuard};

use altium::{draw::PosHoriz, Location};
use eframe::{
    egui_wgpu::{wgpu, RenderState},
    wgpu::{Queue, RenderPass},
};
use glyphon::{
    cosmic_text::Align,
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
use log::debug;
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
    font_system: FontSystem,
    swash_cache: SwashCache,
    atlas: TextAtlas,
    viewport: Viewport,
    renderer: TextRenderer,
    text_buffers: Vec<glyphon::Buffer>,
    buf_count: usize,
    text_areas: Vec<TextAreaBuilder>,
    vs: ViewState,
}

/// Info that goes into a text area except with an index rather than the buffer.
#[derive(Debug)]
struct TextAreaBuilder {
    buf_idx: usize,
    loc: Location,
    scale: f32,
    default_color: Color,
}

impl TextCtx {
    pub fn init(render_state: &RenderState, device: &Device) -> Self {
        let queue = &render_state.queue;
        // let mut font_system = get_font_system();
        let font_system = FontSystem::new();

        let swash_cache = SwashCache::new();
        let cache = Cache::new(&device);
        let viewport = Viewport::new(&device, &cache);
        debug!("{:#?}", &viewport);
        let mut atlas = TextAtlas::new(device, queue, &cache, render_state.target_format);
        let renderer =
            TextRenderer::new(&mut atlas, &device, wgpu::MultisampleState::default(), None);

        // buffer.set_size(&mut font_system, 500.0, 500.0);
        // // buffer.set_size(&mut font_system, physical_width, physical_height);

        // buffer.set_text(
        //     &mut font_system,
        //     "Hello world! 👋\nThis is rendered with 🦅 glyphon 🦁\nThe text below should be partially clipped.\na b c d e f g h i j k l m n o p q r s t u v w x y z",
        //     Attrs::new().family(Family::SansSerif), Shaping::Advanced);
        // buffer.shape_until_scroll(&mut font_system, false);

        Self {
            font_system,
            swash_cache,
            atlas,
            viewport,
            renderer,
            text_buffers: Vec::new(),
            buf_count: 0,
            text_areas: Vec::new(),
            vs: ViewState::default(),
        }
    }

    /// Clear the buffers for redraw. Must be called before calling [`prepare`]!
    pub fn clear(&mut self) {
        self.buf_count = 0;
        self.text_areas.clear();
        // self.text_areas.clear();
        // Leave text_buffers since that contains allocated objects
    }

    pub fn prepare<'a>(&mut self, device: &Device, queue: &Queue, vs: ViewState) {
        self.atlas.trim();
        self.viewport.update(
            queue,
            Resolution {
                width: vs.rect.width() as u32,
                height: vs.rect.height() as u32,
            },
        );

        let text_areas = self.text_areas.iter().map(|ta| {
            debug!(
                "{:?}",
                self.text_buffers[ta.buf_idx].lines.get(0).map(|l| l.text())
            );
            debug!("{:?}", &ta);
            // let gfx = vs.local_to_gfx(ta.loc);
            let gfx = vs.world_to_px(ta.loc);
            debug!("{:?}", &gfx,);
            TextArea {
                buffer: &self.text_buffers[ta.buf_idx],
                left: gfx.x,
                top: gfx.y,
                scale: ta.scale,
                bounds: TextBounds {
                    left: 0,
                    top: 0,
                    right: vs.rect.width() as i32,
                    bottom: vs.rect.height() as i32,
                },
                default_color: ta.default_color,
                custom_glyphs: &[],
            }
        });

        self.renderer
            .prepare(
                device,
                queue,
                &mut get_font_system(),
                &mut self.atlas,
                &self.viewport,
                text_areas,
                &mut self.swash_cache,
            )
            .unwrap();
    }

    pub fn paint(&self, render_pass: &mut RenderPass<'static>) {
        self.renderer
            .render(&self.atlas, &self.viewport, render_pass)
            .unwrap();
    }

    pub fn push_text(&mut self, item: altium::draw::DrawText) {
        let altium::draw::DrawText {
            loc,
            text,
            font,
            anchor_h,
            anchor_v,
            color,
            rotation,
        } = item;

        let buf_idx = self.buf_count;
        self.buf_count += 1;
        if self.text_buffers.len() < self.buf_count {
            let buf = glyphon::Buffer::new(&mut self.font_system, Metrics::new(30.0, 42.0));
            self.text_buffers.push(buf);
        }

        let mut buf = self.text_buffers[buf_idx].borrow_with(&mut self.font_system);
        let align = match anchor_h {
            PosHoriz::Left => Align::Left,
            PosHoriz::Center => Align::Center,
            PosHoriz::Right => Align::Right,
        };
        buf.set_text(
            text,
            &Attrs::new().family(Family::SansSerif),
            Shaping::Advanced,
            Some(align),
        );

        let ta = TextAreaBuilder {
            buf_idx,
            loc,
            scale: 1.0,
            default_color: Color::rgba(color.r, color.g, color.b, 255),
        };

        self.text_areas.push(ta);
    }
}
