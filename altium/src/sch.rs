//! Everything related to schematic documents (`.SchDoc`) and schematic
//! libraries (`.SchLib`)

mod component;
mod params;
mod pin;
pub(crate) mod record;
mod schdoc;
mod schlib;
mod storage;

pub use component::Component;
pub use params::{Justification, SheetStyle};
pub use pin::PinError;
pub(crate) use record::SchRecord;
pub use schdoc::SchDoc;
pub use schlib::{ComponentMeta, ComponentsIter, SchLib};
