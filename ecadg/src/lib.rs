#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_precision_loss)]

mod app;
mod backend;
mod draw;
mod gfx;
pub use app::GuiApp;
pub use backend::open_file_sync_err;
