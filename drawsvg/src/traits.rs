use convert_case::Case;
use core::fmt::Error;
use core::fmt::Write;
use std::any::type_name;

pub trait WriteBuf {
    fn write_buf<W: Write>(&self, f: &mut W) -> Result<(), Error>;
}
