use std::{path::PathBuf, sync::atomic::Ordering::SeqCst};

use egui::{plot::Plot, ScrollArea, TextStyle, Ui};
use regex::Regex;

use crate::{
    backend::{
        open_file,
        GlobalQueue,
        SchDocTab,
        SchLibTab,
        TabData,
        TabDataInner,
        HAS_FRESH_DATA,
    },
    draw::PlotUiWrapper,
};

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

// impl Default for GuiApp {
//     fn default() -> Self {
//         Self {
//             active_tab: None,
//             tabs: vec![],
//             recent_files: vec![],
//             errors: vec![],
//         }
//     }
// }

impl GuiApp {
    /// Called once before the first frame.
    #[must_use]
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

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
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
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
                        dbg!("open file clicked");
                        if let Some(path) = dbg!(rfd::FileDialog::new().pick_file()) {
                            open_file(path);
                        }
                    }

                    ui.menu_button("Open Recent", |ui| {
                        for path in &self.recent_files {
                            if ui.button(path.display().to_string()).clicked() {
                                open_file(path.clone());
                            }
                        }
                    });

                    #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
                    if ui.button("Quit").clicked() {
                        frame.close();
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

#[allow(clippy::needless_pass_by_ref_mut)]
fn make_left_panel_schdoc(ui: &Ui, tab: &mut SchDocTab) {}

#[allow(clippy::needless_pass_by_ref_mut)]
fn make_center_panel(app: &mut GuiApp, ui: &mut Ui) {
    // The central panel the region left after adding TopPanel's and SidePanel's

    ui.horizontal(|ui| {
        for (idx, tab) in app.tabs.iter().enumerate() {
            ui.selectable_value(&mut app.active_tab, Some(idx), &*tab.title);
        }
    });

    let Some(tab_idx) = app.active_tab else {
        return;
    };

    let tab = &mut app.tabs[tab_idx];

    match &mut tab.inner {
        TabDataInner::SchLib(tab) => make_center_panel_schlib(ui, tab),
        TabDataInner::SchDoc(tab) => make_center_panel_schdoc(ui, tab),
    }
}

#[allow(clippy::needless_pass_by_ref_mut)]
fn make_center_panel_schlib(ui: &mut Ui, tab: &mut SchLibTab) {
    let comp = &tab.components[tab.active_component];

    let plot = default_plot();
    let _resp = plot.show(ui, |plot_ui| {
        comp.draw(&mut PlotUiWrapper(plot_ui));
    });
}

#[allow(clippy::needless_pass_by_ref_mut)]
fn make_center_panel_schdoc(ui: &mut Ui, tab: &mut SchDocTab) {
    let plot = default_plot();
    let _resp = plot.show(ui, |plot_ui| {
        tab.records.draw(&mut PlotUiWrapper(plot_ui));
    });
}

fn default_plot() -> Plot {
    Plot::new("main_plot")
        .allow_zoom(true)
        .data_aspect(1.0)
        .show_x(false)
        .show_y(false)
}
