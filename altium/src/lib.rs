#![doc = include_str!("../README.md")]
//! Tools for to reading and writing files produced by Altium Designer.
//!
//! It is very early in development, so please expect surprises if you are using
//! it!
//!
//! # Units
//!
//! Unless otherwise stated, we try to use the following conventions:
//!
//! - For integer values, 1 = 1.0 nm
//! - For floating point values, 1.0 = 1.0 m
//!
//! 1nm precision is pretty excessive for what we need. However, it allows us to represent
//! anything from surface coating up to a 2.2 x 2.2 m PCB in an `i32`, which is more than
//! sufficient for the vast majority of use cases.

// #![allow(unused)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::new_without_default)]
#![allow(clippy::struct_field_names)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::unreadable_literal)]

mod common;
mod logging;
mod parse;

#[doc(hidden)]
pub mod __private;
pub mod draw;
pub mod dwf;
pub mod error;
pub mod font;
pub mod pcb;
pub mod prj;
pub mod sch;

#[doc(inline)]
pub use common::{Location, Rgb, Rotation90, UniqueId, Visibility};
#[doc(inline)]
pub use error::{Error, ErrorKind, Result};
#[doc(inline)]
pub use pcb::{PcbDoc, PcbLib};
#[doc(inline)]
pub use prj::PrjPcb;
#[doc(inline)]
pub use sch::{SchDoc, SchLib};

/// Allow using the sealed trait if this feature is enabled. Used for the GUI crate
#[cfg(not(feature = "_sealed"))]
mod sealed {
    pub trait Sealed {}
}

#[cfg(feature = "_sealed")]
pub mod sealed {
    pub trait Sealed {}
}
