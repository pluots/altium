#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{env, error::Error, path::PathBuf, process::exit};

use ecadg::open_file_sync_err;

#[cfg(debug_assertions)]
// const DEFAULT_LOG_LEVEL: &str = "debug";
const DEFAULT_LOG_LEVEL: &str = "info";
#[cfg(not(debug_assertions))]
const DEFAULT_LOG_LEVEL: &str = "warn";

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, DEFAULT_LOG_LEVEL),
    );

    parse_args()?;

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "ecadg",
        native_options,
        Box::new(|cc| Box::new(ecadg::GuiApp::new(cc))),
    )
    .map_err(Into::into)
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id", // hardcode it
                web_options,
                Box::new(|cc| Box::new(ecadg::GuiApp::new(cc))),
            )
            .await
            .expect("failed to start eframe");
    });
}

const USAGE: &str = "ecadg
eCAD file viewer.

Usage:

    ecadg [files]

All files provided as arguments will be open.
";

fn parse_args() -> Result<(), Box<dyn Error>> {
    for arg in env::args_os() {
        if arg == "-h" || arg == "--help" {
            println!("{USAGE}");
            exit(0);
        }
    }

    // Any remaining arguments get opened as files
    for arg in env::args_os().skip(1).rev() {
        let path: PathBuf = arg.into();
        open_file_sync_err(path)?;
    }

    Ok(())
}
