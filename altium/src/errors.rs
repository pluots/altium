use std::fmt;
use std::io;
use std::num::ParseIntError;
use std::str::Utf8Error;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    IniFormat(ini::ParseError),
    MissingSection(String),
    UniqueId(String),
    FileType(String, &'static str),
    InvalidStream(String, usize),
    RequiredSplit(String),
    Utf8(Utf8Error),
    ExpectedInt(String, ParseIntError),
    InvalidKey(String),
    ExpectedBool(String),
    ExpectedColor(Vec<u8>),
    SheetStyle(u8),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::IniFormat(e) => write!(f, "error parsing ini: {e}"),
            Error::Io(e) => write!(f, "io error: {e}"),
            Error::MissingSection(e) => write!(f, "missing required section `{e}`"),
            Error::UniqueId(e) => write!(f, "bad or missing unique ID section `{e}`"),
            Error::FileType(n, ty) => write!(f, "file `{n}` is not a valid {ty} file"),
            Error::InvalidStream(s, n) => {
                write!(f, "invalid value in stream `{s}` at position {n}")
            }
            Error::RequiredSplit(s) => {
                write!(f, "expected key-value pair but couldn't split `{s}`")
            }
            Error::Utf8(e) => write!(f, "utf8 error: {e}"),
            Error::ExpectedInt(s, e) => write!(f, "error parsing integer from `{s}`: {e}"),
            Error::InvalidKey(s) => write!(f, "invalid key found: `{s}`"),
            Error::ExpectedBool(s) => write!(f, "error parsing bool from `{s}`"),
            Error::ExpectedColor(v) => write!(f, "error parsing color from `{v:?}`"),
            Error::SheetStyle(v) => write!(f, "invalid sheet style {v}"),
        }
    }
}

impl std::error::Error for Error {}

impl Error {
    pub(crate) fn new_invalid_stream(name: &str, pos: usize) -> Self {
        Self::InvalidStream(name.to_owned(), pos)
    }

    pub(crate) fn new_invalid_key(key: &[u8]) -> Self {
        Self::InvalidKey(String::from_utf8_lossy(key).to_string())
    }
}

impl From<ini::ParseError> for Error {
    fn from(value: ini::ParseError) -> Self {
        Self::IniFormat(value)
    }
}

impl From<ini::Error> for Error {
    fn from(value: ini::Error) -> Self {
        match value {
            ini::Error::Io(e) => Self::Io(e),
            ini::Error::Parse(e) => Self::IniFormat(e),
        }
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<Utf8Error> for Error {
    fn from(value: Utf8Error) -> Self {
        Self::Utf8(value)
    }
}
