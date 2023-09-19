//! A pin

use core::fmt;
use std::str::{self, Utf8Error};

use altium_macros::FromRecord;
use log::warn;
use num_enum::TryFromPrimitive;

use super::SchRecord;
use crate::common::{Location, Rotation, Visibility};
use crate::error::AddContext;
use crate::parse::ParseUtf8;
use crate::parse::{FromRecord, FromUtf8};
use crate::{ErrorKind, UniqueId};

/// Representation of a pin
///
/// Altium stores pins as binary in the schematic libraries but text in the
/// schematic documents, so we need to parse both.
#[derive(Clone, Debug, Default, PartialEq, FromRecord)]
#[from_record(id = 2, record_variant = Pin)]
pub struct SchPin {
    pub(super) formal_type: u8,
    pub(super) owner_index: u8,
    pub(super) owner_part_id: u8,
    pub(super) description: Box<str>,
    // #[from_record(rename = b"PinDesignator")]
    pub(super) designator: Box<str>,
    pub(super) name: Box<str>,
    pub(super) location_x: i32,
    pub(super) location_y: i32,
    pub(super) electrical: ElectricalType,
    #[from_record(rename = b"PinLength")]
    pub(super) length: u32,
    #[from_record(rename = b"SwapIDPart")]
    pub(super) swap_id_part: Box<str>,
    pub(super) designator_vis: Visibility,
    pub(super) name_vis: Visibility,
    pub(super) rotation: Rotation,
    #[from_record(rename = b"PinPropagationDelay")]
    pub(super) propegation_delay: f32,
    pub(super) unique_id: UniqueId,
}

impl SchPin {
    pub(crate) fn parse(buf: &[u8]) -> Result<SchRecord, PinError> {
        // 6 bytes unknown
        let [_, _, _, _, _, _, rest @ ..] = buf else {
            return Err(PinError::TooShort(buf.len(), "initial group"));
        };
        // 6 more bytes unknown - symbols
        let [_, _, _, _, _, _, rest @ ..] = rest else {
            return Err(PinError::TooShort(rest.len(), "second group"));
        };

        let (description, rest) = sized_buf_to_utf8(rest, "description")?;

        // TODO: ty_info
        let [formal_type, ty_info, rot_hide, l0, l1, x0, x1, y0, y1, rest @ ..] = rest else {
            return Err(PinError::TooShort(rest.len(), "position extraction"));
        };

        assert_eq!(
            *formal_type, 1,
            "expected formal type of 1 but got {formal_type}"
        );
        let (rotation, des_vis, name_vis) = get_rotation_and_hiding(*rot_hide);
        let length = u16::from_le_bytes([*l0, *l1]);
        let location_x = i16::from_le_bytes([*x0, *x1]);
        let location_y = i16::from_le_bytes([*y0, *y1]);

        let [_, _, _, _, rest @ ..] = rest else {
            return Err(PinError::TooShort(rest.len(), "remaining buffer"));
        };

        let (name, rest) = sized_buf_to_utf8(rest, "name")?;
        let (designator, rest) = sized_buf_to_utf8(rest, "designator")?;

        if !matches!(rest, [_, 0x03, b'|', b'&', b'|']) {
            warn!("unexpected rest: {rest:02x?}");
        }

        let retval = Self {
            formal_type: *formal_type,
            owner_index: 0,
            owner_part_id: 0,
            description: description.into(),
            designator: designator.into(),
            name: name.into(),
            location_x: i32::from(location_x),
            location_y: i32::from(location_y),
            length: u32::from(length),
            // location_x: i32::from(location_x) * 10,
            // location_y: i32::from(location_y) * 10,
            // length: u32::from(length) * 10,
            designator_vis: des_vis,
            name_vis,
            rotation,
            ..Default::default()
        };

        Ok(SchRecord::Pin(retval))
    }

    /// Nonconnecting point of this pin
    pub(crate) fn location(&self) -> Location {
        Location {
            x: self.location_x,
            y: self.location_y,
        }
    }

    /// Altium stores the position of the pin at its non-connecting end. Which
    /// seems dumb. This provides the connecting end.
    pub(crate) fn location_conn(&self) -> Location {
        let x_orig = self.location_x;
        let y_orig = self.location_y;
        let len = i32::try_from(self.length).unwrap();

        let (x, y) = match self.rotation {
            Rotation::R0 => (x_orig + len, y_orig),
            Rotation::R90 => (x_orig, y_orig + len),
            Rotation::R180 => (x_orig - len, y_orig),
            Rotation::R270 => (x_orig, y_orig - len),
        };
        Location { x, y }
    }
}

fn sized_buf_to_utf8<'a>(
    buf: &'a [u8],
    loc: &'static str,
) -> Result<(&'a str, &'a [u8]), PinError> {
    let [text_len_u8, rest @ ..] = buf else {
        return Err(PinError::TooShort(buf.len(), loc));
    };
    let text_len: usize = (*text_len_u8).into();
    let text = str::from_utf8(
        rest.get(..text_len)
            .ok_or(PinError::DescrTooShort(*text_len_u8, rest.len()))?,
    )
    .map_err(PinError::NonUtf8Designator)?;
    Ok((text, &rest[text_len..]))
}

/// Given a byte representing rotation and hiding, extract that info
///
/// Returns `(rotation, designator_vis, name_vis)`
fn get_rotation_and_hiding(val: u8) -> (Rotation, Visibility, Visibility) {
    const ROT_MASK: u8 = 0b00000011;
    const VIS_DES_MASK: u8 = 0b00001000;
    const VIS_NAME_MASK: u8 = 0b00010000;

    let rotation = match val & ROT_MASK {
        x if x == Rotation::R0 as u8 => Rotation::R0,
        x if x == Rotation::R90 as u8 => Rotation::R90,
        x if x == Rotation::R180 as u8 => Rotation::R180,
        x if x == Rotation::R270 as u8 => Rotation::R270,
        _ => unreachable!("2-bit patterns covered"),
    };

    let des_vis = if (val & VIS_DES_MASK) == 0 {
        Visibility::Hidden
    } else {
        Visibility::Visible
    };

    let name_vis = if (val & VIS_NAME_MASK) == 0 {
        Visibility::Hidden
    } else {
        Visibility::Visible
    };

    (rotation, des_vis, name_vis)
}

#[repr(u8)]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub enum ElectricalType {
    #[default]
    Input = 0,
    Id = 1,
    Output = 2,
    OpenCollector = 3,
    Passive = 4,
    HighZ = 5,
    OpenEmitter = 6,
    Power = 7,
}

impl FromUtf8<'_> for ElectricalType {
    fn from_utf8(buf: &[u8]) -> Result<Self, ErrorKind> {
        let num: u8 = buf.parse_as_utf8()?;
        num.try_into()
    }
}

impl TryFrom<u8> for ElectricalType {
    type Error = ErrorKind;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            x if x == Self::Input as u8 => Ok(Self::Input),
            x if x == Self::Id as u8 => Ok(Self::Id),
            x if x == Self::Output as u8 => Ok(Self::Output),
            x if x == Self::OpenCollector as u8 => Ok(Self::OpenCollector),
            x if x == Self::Passive as u8 => Ok(Self::Passive),
            x if x == Self::HighZ as u8 => Ok(Self::HighZ),
            x if x == Self::OpenEmitter as u8 => Ok(Self::OpenEmitter),
            x if x == Self::Power as u8 => Ok(Self::Power),
            _ => Err(ErrorKind::ElectricalType(value)),
        }
    }
}

/// Errors related specifically to pin parsing
#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum PinError {
    /// `(length, position)`
    TooShort(usize, &'static str),
    DescrTooShort(u8, usize),
    NonUtf8Designator(Utf8Error),
}

impl fmt::Display for PinError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PinError::TooShort(v, c) => write!(f, "buffer length {v} too short at {c}"),
            PinError::DescrTooShort(a, b) => write!(f, "description needs {a} bytes but got {b}"),
            PinError::NonUtf8Designator(e) => e.fmt(f),
        }
    }
}
