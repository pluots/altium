//! Error types used throughout this crate

use std::cmp::min;
use std::fmt;
use std::fmt::Write;
use std::io;
use std::num::ParseIntError;
use std::str::Utf8Error;

use crate::sch::PinError;

/// Our main error type is an error ([`ErrorKind`]) plus some context for what
/// caused it, a quasi-backtrace.
pub struct Error {
    kind: Box<ErrorKind>,
    trace: Vec<Frame>,
}

impl Error {
    /// The original error that caused this problem
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    /// A trace of where this error came from (useful for debugging)
    pub fn trace(&self) -> &[Frame] {
        &self.trace
    }
}

/// A single step of context
pub struct Frame {
    inner: Box<str>,
}

impl Frame {
    /// This step's description
    pub fn description(&self) -> &str {
        &self.inner
    }

    fn new<T: Into<Box<str>>>(s: T) -> Self {
        Self { inner: s.into() }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}. context:", self.kind)?;
        for (idx, frame) in self.trace.iter().enumerate() {
            write!(f, "  {idx:2}: {}", frame.description())?;
        }
        Ok(())
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

/// A raw error caused somewhere along the file
#[derive(Debug)]
#[non_exhaustive]
pub enum ErrorKind {
    Io(io::Error),
    IniFormat(Box<ini::ParseError>),
    MissingSection(String),
    MissingUniqueId(String),
    InvalidUniqueId(Box<[u8]>),
    FileType(String, &'static str),
    InvalidStream(String, usize),
    RequiredSplit(String),
    Utf8(Utf8Error),
    ExpectedInt(String, ParseIntError),
    InvalidKey(String),
    ExpectedBool(String),
    ExpectedColor(Box<u8>),
    SheetStyle(u8),
    ReadOnlyState(u8),
    Justification(u8),
    Pin(PinError),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::IniFormat(e) => write!(f, "error parsing ini: {e}"),
            ErrorKind::Io(e) => write!(f, "io error: {e}"),
            ErrorKind::MissingSection(e) => write!(f, "missing required section `{e}`"),
            ErrorKind::MissingUniqueId(e) => write!(f, "bad or missing unique ID section `{e}`"),
            ErrorKind::InvalidUniqueId(e) => {
                write!(f, "invalid unique ID section `{}`", TruncBuf::new(e))
            }
            ErrorKind::FileType(n, ty) => write!(f, "file `{n}` is not a valid {ty} file"),
            ErrorKind::InvalidStream(s, n) => {
                write!(f, "invalid value in stream `{s}` at position {n}")
            }
            ErrorKind::RequiredSplit(s) => {
                write!(f, "expected key-value pair but couldn't split `{s}`")
            }
            ErrorKind::Utf8(e) => write!(f, "utf8 error: {e}"),
            ErrorKind::ExpectedInt(s, e) => write!(f, "error parsing integer from `{s}`: {e}"),
            ErrorKind::InvalidKey(s) => write!(f, "invalid key found: `{s}`"),
            ErrorKind::ExpectedBool(s) => write!(f, "error parsing bool from `{s}`"),
            ErrorKind::ExpectedColor(v) => write!(f, "error parsing color from `{v:?}`"),
            ErrorKind::SheetStyle(v) => write!(f, "invalid sheet style {v}"),
            ErrorKind::ReadOnlyState(v) => write!(f, "invalid readonly state {v}"),
            ErrorKind::Justification(v) => write!(f, "invalid justification state {v}"),
            ErrorKind::Pin(v) => write!(f, "error parsing pin: {v}"),
        }
    }
}

impl std::error::Error for ErrorKind {}

impl ErrorKind {
    pub(crate) fn new_invalid_stream(name: &str, pos: usize) -> Self {
        Self::InvalidStream(name.to_owned(), pos)
    }

    pub(crate) fn new_invalid_key(key: &[u8]) -> Self {
        Self::InvalidKey(String::from_utf8_lossy(key).to_string())
    }
}

impl From<ini::ParseError> for ErrorKind {
    fn from(value: ini::ParseError) -> Self {
        Self::IniFormat(Box::new(value))
    }
}

impl From<ini::Error> for ErrorKind {
    fn from(value: ini::Error) -> Self {
        match value {
            ini::Error::Io(e) => Self::Io(e),
            ini::Error::Parse(e) => Self::IniFormat(Box::new(e)),
        }
    }
}

impl From<io::Error> for ErrorKind {
    fn from(value: io::Error) -> Self {
        ErrorKind::Io(value)
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self {
            kind: Box::new(value.into()),
            trace: Vec::new(),
        }
    }
}

impl From<Utf8Error> for ErrorKind {
    fn from(value: Utf8Error) -> Self {
        Self::Utf8(value)
    }
}

impl From<ErrorKind> for Error {
    fn from(value: ErrorKind) -> Self {
        Self {
            kind: Box::new(value),
            trace: Vec::new(),
        }
    }
}

impl From<PinError> for Error {
    fn from(value: PinError) -> Self {
        Error {
            kind: Box::new(ErrorKind::Pin(value)),
            trace: Vec::new(),
        }
    }
}

/// This trait lets us throw random context on errors and make them brand new
pub(crate) trait AddContext: Sized {
    type WithContext;

    /// Convert a type to an `Error`
    fn context<C: Into<Box<str>>>(self, ctx: C) -> Self::WithContext;

    /// Add context that is lazily evaluated
    fn then_context<F, C>(self, f: F) -> Self::WithContext
    where
        F: FnOnce() -> C,
        C: Into<Box<str>>,
    {
        self.context(f())
    }
}

impl AddContext for Error {
    type WithContext = Self;

    fn context<C: Into<Box<str>>>(mut self, ctx: C) -> Self::WithContext {
        self.trace.push(Frame::new(ctx.into()));
        self
    }
}

impl<T> AddContext for Result<T, Error> {
    type WithContext = Self;

    fn context<C: Into<Box<str>>>(self, ctx: C) -> Self::WithContext {
        self.map_err(|mut e| {
            e.trace.push(Frame::new(ctx));
            e
        })
    }
}

impl AddContext for ErrorKind {
    type WithContext = Error;

    /// Convert `ErrorKind` to Error
    fn context<C: Into<Box<str>>>(self, ctx: C) -> Self::WithContext {
        Error {
            kind: Box::new(self),
            trace: vec![Frame::new(ctx)],
        }
    }
}

impl<T> AddContext for Result<T, ErrorKind> {
    type WithContext = Result<T, Error>;

    /// Convert `ErrorKind` to Error
    fn context<C: Into<Box<str>>>(self, ctx: C) -> Self::WithContext {
        self.map_err(|e| Error {
            kind: Box::new(e),
            trace: vec![Frame::new(ctx)],
        })
    }
}

/// Helper type for errors related to buffers
pub(crate) struct TruncBuf<'a>(&'a [u8]);

impl<'a> TruncBuf<'a> {
    /// Truncate a buffer to 16 elements and box them. Useful for reporting errors
    /// on buffers that may be too large
    // FIXME: version: bounds loosened to `Clone`-only in 1.71
    // <https://github.com/rust-lang/rust/pull/103406>
    pub fn truncate<T: Clone + Copy>(buf: &[T]) -> Box<[T]> {
        buf[..min(buf.len(), 16)].into()
    }

    pub fn new(buf: &'a [u8]) -> Self {
        Self(buf)
    }
}

impl fmt::Display for TruncBuf<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.0).entry(&"...").finish()
    }
}
