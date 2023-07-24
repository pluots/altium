//! Tools for to reading and writing files produced by Altium Designer.
//!
//! It is very early in development, so please expect surprises if you are using
//! it!

#![allow(unused)]
#![warn(clippy::pedantic)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::missing_panics_doc)]

mod common;
mod draw;
mod logging;
mod parse;

#[doc(hidden)]
pub mod __private;
pub mod dwf;
pub mod errors;
pub mod font;
pub mod pcb;
pub mod prj;
pub mod sch;

#[doc(inline)]
pub use common::UniqueId;
#[doc(inline)]
pub use errors::{Error, ErrorKind};
#[doc(inline)]
pub use pcb::{PcbDoc, PcbLib};
#[doc(inline)]
pub use prj::PrjPcb;
#[doc(inline)]
pub use sch::{SchDoc, SchLib};

#[cfg(doctest)]
mod readme_tests {
    #[doc = include_str!("../../README.md")]
    struct MainReadMe;
}
