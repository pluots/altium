#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_precision_loss)]

#[allow(unused_macros)]
macro_rules! do_once {
    ($($tt:tt)*) => {{
        static DONE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        if !DONE.swap(true, std::sync::atomic::Ordering::Relaxed) {
            $($tt)*
        }
    }}
}

#[allow(unused_macros)]
macro_rules! println_once {
    ($($tt:tt)*) => {
        do_once!(println!($($tt:tt)*))
    }
}

mod app;
mod backend;
mod draw;
mod gfx;
pub use app::GuiApp;
pub use backend::open_file_sync_err;
