#![allow(unused)]

use core::fmt;

use traits::WriteBuf;
mod traits;

pub struct Svg {
    attributes: Vec<SvgAttribute>,
    elements: Vec<Element>,
}

impl WriteBuf for Svg {
    fn write_buf<W: fmt::Write>(&self, f: &mut W) -> Result<(), fmt::Error> {
        write!(f, "<")?;
        write!(f, r#"xmlns="http://www.w3.org/2000/svg">"#)?;
    }
}

pub enum SvgAttribute {
    ViewBox(ViewBox),
    Width(Length),
    Height(Length),
}

pub struct ViewBox {
    min_x: Number,
    min_y: Number,
    width: Number,
    height: Number,
}

pub enum Number {
    F(f32),
    I(i32),
}

pub enum Length {
    Em(Number),
    #[default]
    Px(Number),
    Mm(Number),
    Pt(Number),
    Pct(Number),
}

pub enum Element {
    Text(Text),
}

pub struct Text {
    attributes: Vec<SvgAttribute>,
    content: String,
}

impl Text {
    fn new(text: &str) -> Self {
        Self {
            attributes: Vec::new(),
            content: text.to_owned(),
        }
    }
}
