//! Tools related to drawing objects

pub(crate) mod canvas;
mod svg;

pub use canvas::{
    Canvas,
    DrawImage,
    DrawLine,
    DrawPolygon,
    DrawRectangle,
    DrawText,
    LineCap,
    LineJoin,
};

pub use self::svg::SvgCtx;
pub use crate::common::{Location, PosHoriz, PosVert, Rgb};

/// Implementors of this trait can draw themselves to a canvas.
pub trait Draw {
    /// Additional context needed to draw
    type Context<'a>;

    /// Draw this element to a [`Canvas`].
    ///
    /// This has a defualt implementation that does nothing for easier
    /// reusability
    #[allow(unused)]
    fn draw<C: Canvas>(&self, canvas: &mut C, ctx: &Self::Context<'_>) {}
}
