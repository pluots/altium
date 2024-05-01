pub(crate) mod canvas;
mod svg;

pub use canvas::{Canvas, DrawImage, DrawLine, DrawPolygon, DrawRectangle, DrawText};

pub use self::svg::SvgCtx;
pub use crate::common::{Color, Location, PosHoriz, PosVert};

pub trait Draw {
    type Context<'a>;

    /// Draw this element to a SVG and return the new SVG
    ///
    /// This has a defualt implementation that does nothing for easier
    /// reusability
    #[allow(unused)]
    fn draw<C: Canvas>(&self, canvas: &mut C, ctx: &Self::Context<'_>) {}
}
