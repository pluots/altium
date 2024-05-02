//! Parameter types stored in a schematic

use serde::{Deserialize, Serialize};

use crate::{
    common::{PosHoriz, PosVert},
    parse::{FromUtf8, ParseUtf8},
    ErrorKind,
};

/// Sheet paper style,
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum SheetStyle {
    /// 292.1 mm x 193.0 mm sheet (1150x760 mil)
    #[default]
    A4 = 0,
    /// 393.7 mm x 281.9 mm sheet (1550x1110 mil)
    A3 = 1,
    /// 566.4 mm x 398.8 mm sheet (2230x1570 mil)
    A2 = 2,
    /// 800.1 mm x 566.4 mm sheet (3150x2230 mil)
    A1 = 3,
    /// 1132.8 mm x 800.1 mm sheet (4460x3150 mil)
    A0 = 4,
    /// 241.3 mm x 190.5 mm sheet (950x750 mil)
    A = 5,
    /// 381.0 mm x 241.3 mm sheet (1500x950 mil)
    B = 6,
    /// 508.0 mm x 381.0 mm sheet (2000x1500 mil)
    C = 7,
    /// 812.8 mm x 508.0 mm sheet (3200x2000 mil)
    D = 8,
    /// 1066.8 mm x 812.8 mm sheet (4200x3200 mil)
    E = 9,
    /// 279.4 mm x 215.9 mm sheet (1100x850 mil)
    Letter = 10,
    /// 355.6 mm x 215.9 mm sheet (1400x850 mil)
    Legal = 11,
    /// 431.8 mm x 279.4 mm sheet (1700x1100 mil)
    Tabloid = 12,
    /// 251.5 mm x 200.7 mm sheet (990x790 mil)
    OrCadA = 13,
    /// 391.2 mm x 251.5 mm sheet (1540x990 mil)
    OrCadB = 14,
    /// 523.2 mm x 396.2 mm sheet (2060x1560 mil)
    OrCadC = 15,
    /// 828.0 mm x 523.2 mm sheet (3260x2060 mil)
    OrCadD = 16,
    /// 1087.1 mm x 833.1 mm sheet (4280x3280 mil)
    OrCadE = 17,
}

impl TryFrom<u8> for SheetStyle {
    type Error = ErrorKind;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let res = match value {
            x if x == Self::A4 as u8 => Self::A4,
            x if x == Self::A3 as u8 => Self::A3,
            x if x == Self::A2 as u8 => Self::A2,
            x if x == Self::A1 as u8 => Self::A1,
            x if x == Self::A0 as u8 => Self::A0,
            x if x == Self::A as u8 => Self::A,
            x if x == Self::B as u8 => Self::B,
            x if x == Self::C as u8 => Self::C,
            x if x == Self::D as u8 => Self::D,
            x if x == Self::E as u8 => Self::E,
            x if x == Self::Letter as u8 => Self::Letter,
            x if x == Self::Legal as u8 => Self::Legal,
            x if x == Self::Tabloid as u8 => Self::Tabloid,
            x if x == Self::OrCadA as u8 => Self::OrCadA,
            x if x == Self::OrCadB as u8 => Self::OrCadB,
            x if x == Self::OrCadC as u8 => Self::OrCadC,
            x if x == Self::OrCadD as u8 => Self::OrCadD,
            x if x == Self::OrCadE as u8 => Self::OrCadE,
            _ => return Err(ErrorKind::SheetStyle(value)),
        };

        Ok(res)
    }
}

impl FromUtf8<'_> for SheetStyle {
    fn from_utf8(buf: &[u8]) -> Result<Self, ErrorKind> {
        let num: u8 = buf.parse_as_utf8()?;
        num.try_into()
    }
}

/// Allowed text alignments in a schematic
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum Justification {
    #[default]
    BottomLeft = 0,
    BottomCenter = 1,
    BottomRight = 2,
    CenterLeft = 3,
    CenterCenter = 4,
    CenterRight = 5,
    TopLeft = 6,
    TopCenter = 7,
    TopRight = 8,
}

impl TryFrom<u8> for Justification {
    type Error = ErrorKind;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let res = match value {
            x if x == Self::BottomLeft as u8 => Self::BottomLeft,
            x if x == Self::BottomCenter as u8 => Self::BottomCenter,
            x if x == Self::BottomRight as u8 => Self::BottomRight,
            x if x == Self::CenterLeft as u8 => Self::CenterLeft,
            x if x == Self::CenterCenter as u8 => Self::CenterCenter,
            x if x == Self::CenterRight as u8 => Self::CenterRight,
            x if x == Self::TopLeft as u8 => Self::TopLeft,
            x if x == Self::TopCenter as u8 => Self::TopCenter,
            x if x == Self::TopRight as u8 => Self::TopRight,
            _ => return Err(ErrorKind::Justification(value)),
        };

        Ok(res)
    }
}

impl FromUtf8<'_> for Justification {
    fn from_utf8(buf: &[u8]) -> Result<Self, ErrorKind> {
        let num: u8 = buf.parse_as_utf8()?;
        num.try_into()
    }
}

impl From<Justification> for (PosHoriz, PosVert) {
    fn from(value: Justification) -> Self {
        match value {
            Justification::BottomLeft => (PosHoriz::Left, PosVert::Bottom),
            Justification::BottomCenter => (PosHoriz::Center, PosVert::Bottom),
            Justification::BottomRight => (PosHoriz::Right, PosVert::Bottom),
            Justification::CenterLeft => (PosHoriz::Left, PosVert::Middle),
            Justification::CenterCenter => (PosHoriz::Center, PosVert::Middle),
            Justification::CenterRight => (PosHoriz::Right, PosVert::Middle),
            Justification::TopLeft => (PosHoriz::Left, PosVert::Top),
            Justification::TopCenter => (PosHoriz::Center, PosVert::Top),
            Justification::TopRight => (PosHoriz::Right, PosVert::Top),
        }
    }
}
