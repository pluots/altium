//! Everything related to schematic documents (`.SchDoc`) and schematic
//! libraries (`.SchLib`)

mod component;
mod params;
mod pin;
mod schdoc;
mod schlib;

pub(crate) mod record;

pub mod storage;

pub use component::Component;
pub use params::{Justification, SheetStyle};
pub use pin::PinError;
pub(crate) use record::{SchDrawCtx, SchRecord};
pub use schdoc::{SchDoc, SchDocRecords};
pub use schlib::{ComponentMeta, ComponentsIter, SchLib};
