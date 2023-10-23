//! Everything related to schematic documents (`.SchDoc`) and schematic
//! libraries (`.SchLib`)

mod component;
mod params;
mod pin;
mod schdoc;
mod schlib;
#[doc(hidden)]
pub mod storage;

pub mod record;

pub use component::Component;
pub use params::{Justification, SheetStyle};
#[doc(inline)]
pub use pin::{ElectricalType, PinError, SchPin};
#[doc(inline)]
pub use record::{SchDrawCtx, SchRecord};
pub use schdoc::SchDoc;
pub use schlib::{ComponentMeta, ComponentsIter, SchLib};
#[doc(inline)]
pub use storage::Storage;
