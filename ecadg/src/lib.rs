#![allow(unused)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod app;
mod backend;
mod draw;
mod graphics;
pub use app::GuiApp;
pub use backend::open_file_sync_err;
