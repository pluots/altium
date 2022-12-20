use std::{error::Error, fmt::Display};

use ini;

#[derive(Debug)]
pub enum AltiumError {
    Io(std::io::Error),
    IniFormat(ini::ParseError),
    MissingSection(String),
    UniqueId(String),
}

impl Display for AltiumError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AltiumError::IniFormat(e) => write!(f, "error parsing ini: {e}"),
            AltiumError::Io(e) => write!(f, "io error: {e}"),
            AltiumError::MissingSection(e) => write!(f, "missing required section `{e}`"),
            AltiumError::UniqueId(e) => write!(f, "bad or missing unique ID section `{e}`"),
        }
    }
}

impl Error for AltiumError {}

impl From<ini::ParseError> for AltiumError {
    fn from(value: ini::ParseError) -> Self {
        Self::IniFormat(value)
    }
}

impl From<ini::Error> for AltiumError {
    fn from(value: ini::Error) -> Self {
        match value {
            ini::Error::Io(e) => Self::Io(e),
            ini::Error::Parse(e) => Self::IniFormat(e),
        }
    }
}
