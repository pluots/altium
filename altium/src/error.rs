//! Error types used throughout this crate

use std::borrow::Cow;
use std::cmp::min;
use std::fmt;
use std::io;
use std::num::{ParseFloatError, ParseIntError};
use std::str::Utf8Error;

use crate::sch::PinError;

/// The main result type used by this crate
pub type Result<T, E = Error> = core::result::Result<T, E>;

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

    /// A trace of where this error came from.
    ///
    /// This is a library trace rather than a full frame stacktrace, but it is
    /// useful for debugging.
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
        self.kind.fmt(f)?;

        if !self.trace.is_empty() {
            f.write_str(". context:")?;

            for (idx, frame) in self.trace.iter().enumerate() {
                write!(f, "\n  {idx:2}: {}", frame.description())?;
            }
            f.write_str("\n")?;
        }

        Ok(())
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

/// A raw error without context. See [`Error`] for more information.
#[derive(Debug)]
#[non_exhaustive]
pub enum ErrorKind {
    BufferTooShort(usize, TruncBuf<u8>),
    ElectricalType(u8),
    ExpectedBool(String),
    ExpectedColor(TruncBuf<u8>),
    ExpectedFloat(String, ParseFloatError),
    ExpectedInt(String, ParseIntError),
    ExpectedNul(TruncBuf<u8>),
    FileType(String, &'static str),
    Image(Box<image::ImageError>),
    IniFormat(ini::ParseError),
    InvalidHeader(Box<str>, &'static str),
    InvalidKey(Box<str>),
    InvalidStorageData(TruncBuf<u8>),
    InvalidStream(Box<str>, usize),
    InvalidUniqueId(TruncBuf<u8>),
    Io(io::Error),
    Justification(u8),
    MissingSection(String),
    MissingUniqueId,
    Overflow(i64, i64, char),
    Pin(PinError),
    ReadOnlyState(u8),
    RequiredSplit(String),
    SheetStyle(u8),
    Utf8(Utf8Error, String),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::IniFormat(e) => write!(f, "error parsing ini: {e}"),
            ErrorKind::ElectricalType(e) => write!(f, "invalid electrical type {e}"),
            ErrorKind::Io(e) => write!(f, "io error: {e}"),
            ErrorKind::MissingSection(e) => write!(f, "missing required section `{e}`"),
            ErrorKind::MissingUniqueId => write!(f, "bad or missing unique ID section"),
            ErrorKind::InvalidUniqueId(e) => {
                write!(
                    f,
                    "invalid unique ID section `{e}` (len {}, `{}`)",
                    e.orig_len,
                    e.as_str()
                )
            }
            ErrorKind::FileType(n, ty) => write!(f, "file `{n}` is not a valid {ty} file"),
            ErrorKind::InvalidStream(s, n) => {
                write!(f, "invalid value in stream `{s}` at position {n}")
            }
            ErrorKind::RequiredSplit(s) => {
                write!(f, "expected key-value pair but couldn't split `{s}`")
            }
            ErrorKind::InvalidStorageData(e) => {
                write!(f, "invalid storage data near `{e:x}`")
            }
            ErrorKind::Utf8(e, s) => write!(f, "utf8 error: {e} at '{s}'"),
            ErrorKind::InvalidHeader(e, v) => write!(f, "invalid header '{e}'; expected `{v}`"),
            ErrorKind::ExpectedInt(s, e) => write!(f, "error parsing integer from `{s}`: {e}"),
            ErrorKind::ExpectedFloat(s, e) => write!(f, "error parsing float from `{s}`: {e}"),
            ErrorKind::InvalidKey(s) => write!(f, "invalid key found: `{s}`"),
            ErrorKind::ExpectedBool(s) => write!(f, "error parsing bool from `{s}`"),
            ErrorKind::ExpectedColor(v) => write!(f, "error parsing color from `{v:x}`"),
            ErrorKind::SheetStyle(v) => write!(f, "invalid sheet style {v}"),
            ErrorKind::ReadOnlyState(v) => write!(f, "invalid readonly state {v}"),
            ErrorKind::Justification(v) => write!(f, "invalid justification state {v}"),
            ErrorKind::Pin(v) => write!(f, "error parsing pin: {v}"),
            ErrorKind::BufferTooShort(v, b) => write!(
                f,
                "buffer too short: expected at least {v} elements but got {} near {b:x}",
                b.len()
            ),
            ErrorKind::Image(e) => write!(f, "image error: {e}"),
            ErrorKind::ExpectedNul(e) => write!(f, "expected nul near {e}"),
            ErrorKind::Overflow(a, b, op) => write!(f, "overflow at {a} {op} {b}"),
        }
    }
}

impl std::error::Error for ErrorKind {}

impl ErrorKind {
    pub(crate) fn new_invalid_stream(name: &str, pos: usize) -> Self {
        Self::InvalidStream(name.into(), pos)
    }

    #[allow(unused)]
    pub(crate) fn new_invalid_key(key: &[u8]) -> Self {
        Self::InvalidKey(String::from_utf8_lossy(key).into())
    }

    pub(crate) fn new_invalid_header(header: &[u8], expected: &'static str) -> Self {
        Self::InvalidHeader(String::from_utf8_lossy(header).into(), expected)
    }
}

impl From<ini::ParseError> for ErrorKind {
    fn from(value: ini::ParseError) -> Self {
        Self::IniFormat(value)
    }
}

impl From<ini::ParseError> for Error {
    fn from(value: ini::ParseError) -> Self {
        Self {
            kind: ErrorKind::from(value).into(),
            trace: Vec::new(),
        }
    }
}

impl From<ini::Error> for ErrorKind {
    fn from(value: ini::Error) -> Self {
        match value {
            ini::Error::Io(e) => Self::Io(e),
            ini::Error::Parse(e) => Self::IniFormat(e),
        }
    }
}

impl From<ini::Error> for Error {
    fn from(value: ini::Error) -> Self {
        Self {
            kind: ErrorKind::from(value).into(),
            trace: Vec::new(),
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

impl From<image::ImageError> for ErrorKind {
    fn from(value: image::ImageError) -> Self {
        Self::Image(Box::new(value))
    }
}

/// This trait lets us throw random context on errors and make them brand new
pub(crate) trait AddContext: Sized {
    type WithContext;

    /// Convert a type to an `Error`
    fn context<C: Into<Box<str>>>(self, ctx: C) -> Self::WithContext;

    /// Add context that is lazily evaluated
    fn or_context<F, C>(self, f: F) -> Self::WithContext
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

/// A subslice of a buffer for nicer error messages
#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct TruncBuf<T> {
    buf: Box<[T]>,
    orig_len: usize,
    at_end: bool,
}

impl<T: Clone + Copy> TruncBuf<T> {
    /// Truncate a buffer to 16 elements and box them. Useful for reporting errors
    /// on buffers that may be too large
    // FIXME: version: bounds loosened to `Clone`-only in 1.71
    // <https://github.com/rust-lang/rust/pull/103406>
    pub(crate) fn truncate(buf: &[T]) -> Box<[T]> {
        buf[..min(buf.len(), 16)].into()
    }

    pub(crate) fn truncate_end(buf: &[T]) -> Box<[T]> {
        buf[buf.len().saturating_sub(16)..].into()
    }

    /// Print the leftmost elements
    pub(crate) fn new(buf: &[T]) -> Self {
        Self {
            buf: Self::truncate(buf),
            orig_len: buf.len(),
            at_end: false,
        }
    }

    /// Print the rightmost elements
    pub(crate) fn new_end(buf: &[T]) -> Self {
        Self {
            buf: Self::truncate_end(buf),
            orig_len: buf.len(),
            at_end: true,
        }
    }

    /// Length of the original buffer
    pub(crate) fn len(&self) -> usize {
        self.orig_len
    }
}

impl TruncBuf<u8> {
    pub(crate) fn as_str(&self) -> Cow<str> {
        String::from_utf8_lossy(&self.buf)
    }
}

impl<T: Clone + Copy> From<&[T]> for TruncBuf<T> {
    fn from(value: &[T]) -> Self {
        Self::new(value)
    }
}

impl<T: fmt::Debug> fmt::Display for TruncBuf<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.at_end {
            f.debug_list().entry(&{ .. }).entries(&*self.buf).finish()
        } else {
            f.debug_list().entries(&*self.buf).entry(&{ .. }).finish()
        }
    }
}

impl<T: fmt::LowerHex> fmt::LowerHex for TruncBuf<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;

        if self.at_end {
            write!(f, "..")?;
            for val in &*self.buf {
                write!(f, ", {val:02x}")?;
            }
            write!(f, "]")
        } else {
            for val in &*self.buf {
                write!(f, "{val:02x}, ")?;
            }
            write!(f, "..]")
        }
    }
}
