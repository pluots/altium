use std::sync::Arc;
use std::{path::PathBuf, sync::atomic::Ordering::SeqCst};

use egui::{ScrollArea, TextStyle, Ui};
use log::debug;

use crate::backend::{
    open_file_async,
    GlobalQueue,
    SchDocTab,
    SchLibTab,
    TabData,
    TabDataInner,
    ViewState,
    HAS_FRESH_DATA,
};
#[cfg(feature = "_debug")]
use crate::backend::{rect_disp, vec_disp};
use crate::gfx;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct GuiApp {
    #[serde(skip)]
    tabs: Vec<TabData>,

    #[serde(skip)]
    errors: Vec<String>,

    recent_files: Vec<PathBuf>,

    /// Index of the active tab
    #[serde(skip)]
    active_tab: Option<usize>,
}

impl GuiApp {
    /// Called once before the first frame.
    #[must_use]
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        log::warn!("NEW");
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        gfx::init_graphics(cc);

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        GuiApp::default()
    }
}

impl eframe::App for GuiApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    #[allow(clippy::too_many_lines)]
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if HAS_FRESH_DATA.load(SeqCst) {
            // If there is fresh data, grab it
            GlobalQueue::drain(&mut self.tabs, &mut self.recent_files, &mut self.errors);
            // dbg!(&self.errors);
            // FIXME: deduplicate stored tabs based on path
        }

        // If no tab is selected, select the first one
        if self.active_tab.is_none() && !self.tabs.is_empty() {
            self.active_tab = Some(0);
        } else if self.tabs.is_empty() {
            self.active_tab = None;
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            debug!("picked file {}", path.display());
                            open_file_async(path);
                        }
                    }

                    ui.menu_button("Open Recent", |ui| {
                        for path in &self.recent_files {
                            if ui.button(path.display().to_string()).clicked() {
                                open_file_async(path.clone());
                            }
                        }
                    });

                    #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("bottom panel");
                });
            });
        });

        egui::SidePanel::left("left_panel").show(ctx, |ui| make_left_panel(self, ui));

        egui::CentralPanel::default().show(ctx, |ui| make_center_panel(self, ui));

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally choose either panels OR windows.");
            });
        }
    }
}

fn make_left_panel(app: &mut GuiApp, ui: &mut Ui) {
    ui.heading("Side Panel");

    if let Some(tab_idx) = app.active_tab {
        let tab = &mut app.tabs[tab_idx];

        match &mut tab.inner {
            TabDataInner::SchLib(tab) => make_left_panel_schlib(ui, tab),
            TabDataInner::SchDoc(tab) => make_left_panel_schdoc(ui, tab),
        }
    }

    ui.heading("errors");

    for error in &app.errors {
        ui.label(error);
    }

    egui::warn_if_debug_build(ui);
}

fn make_left_panel_schlib(ui: &mut Ui, tab: &mut SchLibTab) {
    // Search bar
    ui.horizontal(|ui| ui.text_edit_singleline(&mut tab.search_query));
    // FIXME: use search query
    tab.hide_items.clear();
    if !tab.search_query.is_empty() {
        let sq_lc = tab.search_query.to_lowercase();
        for (idx, c) in tab.components.iter().enumerate() {
            if !c.name().to_ascii_lowercase().contains(&sq_lc) {
                tab.hide_items.push(idx);
            }
        }
    }

    ScrollArea::vertical().auto_shrink([false; 2]).show_rows(
        ui,
        ui.text_style_height(&TextStyle::Body),
        tab.components.len(),
        |ui, row_range| {
            for row_idx in row_range {
                if tab.hide_items.contains(&row_idx) {
                    continue;
                }
                ui.selectable_value(
                    &mut tab.active_component,
                    row_idx,
                    tab.components[row_idx].name(),
                );
                // ui.label(tab.components[row].name());
            }
        },
    );
}

#[allow(unused)]
#[allow(clippy::needless_pass_by_ref_mut)]
fn make_left_panel_schdoc(ui: &Ui, tab: &mut SchDocTab) {}

/// The central main content panel the region left after adding `TopPanel`'s and `SidePanel`'s
#[allow(clippy::needless_pass_by_ref_mut)]
fn make_center_panel(app: &mut GuiApp, ui: &mut Ui) {
    // tab row
    ui.horizontal(|ui| {
        for (idx, tab) in app.tabs.iter().enumerate() {
            ui.selectable_value(&mut app.active_tab, Some(idx), &*tab.title);
        }
    });

    // main content
    let (rect, response) = ui.allocate_at_least(ui.available_size(), egui::Sense::click_and_drag());
    let Some(tab_idx) = app.active_tab else {
        return;
    };

    let tabdata = &mut app.tabs[tab_idx];

    let hovered = response.hovered();
    let view_state = &mut tabdata.view_state;
    view_state.rect = rect;
    if hovered {
        view_state.update_dragged_by(response.drag_delta());
        ui.input(|istate| view_state.update_with_input_state(istate));
    }

    #[cfg(feature = "_debug")]
    ui.label(format!(
        "view_state: {view_state:?}; vp: {}; hovered: {hovered}; vs offset_gfx: {}; pos world: {}",
        rect_disp(view_state.world_viewport()),
        vec_disp(view_state.offset_gfx()),
        vec_disp(view_state.px_to_world(view_state.latest_pos.unwrap_or_default().to_vec2()))
    ));

    match &mut tabdata.inner {
        TabDataInner::SchLib(tab) => make_center_panel_schlib(ui, rect, tab, view_state),
        TabDataInner::SchDoc(tab) => make_center_panel_schdoc(ui, rect, tab, view_state),
    }
}

#[allow(clippy::needless_pass_by_ref_mut)]
fn make_center_panel_schlib(ui: &mut Ui, rect: egui::Rect, tab: &SchLibTab, vs: &ViewState) {
    let comp = Arc::clone(&tab.components[tab.active_component]);
    ui.label(format!("rect: {rect:?}, vs: {vs:?}"));
    egui::Frame::canvas(ui.style()).show(ui, |ui| {
        ui.painter()
            .add(crate::gfx::SchLibCallback::callback(comp, vs))
    });
}

#[allow(clippy::needless_pass_by_ref_mut)]
fn make_center_panel_schdoc(
    _ui: &mut Ui,
    _rect: egui::Rect,
    _tab: &mut SchDocTab,
    _vs: &mut ViewState,
) {
}
