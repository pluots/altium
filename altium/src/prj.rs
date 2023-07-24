//! `.PrjPcb` configuration

#![allow(unused)]

mod parse;
mod prjcfg;

#[cfg(test)]
mod tests;

pub use prjcfg::{Document, PrjPcb};
