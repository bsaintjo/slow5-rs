#![doc = include_str!("../README.md")]
// #![warn(missing_docs)]

mod aux;
mod compression;
mod error;
mod experimental;
mod header;
mod reader;
mod record;
mod writer;

pub mod typed;

use std::ffi::CString;

pub use aux::FieldType;
pub use compression::{Options, RecordCompression, SignalCompression};
pub use error::Slow5Error;
pub use header::{Header, HeaderView};
pub use reader::FileReader;
pub use record::{Record, RecordBuilder, RecordExt, RecordIter};
pub use writer::FileWriter;

pub(crate) fn to_cstring<T: Into<Vec<u8>>>(x: T) -> Result<CString, Slow5Error> {
    CString::new(x).map_err(Slow5Error::InteriorNul)
}
