#[derive(Debug, PartialEq)]
pub struct UniqueId([u8; 8]);

impl UniqueId {
    pub fn from_slice<S: AsRef<[u8]>>(buf: S) -> Option<Self> {
        buf.as_ref().try_into().ok().map(|v| Self(v))
    }
}
