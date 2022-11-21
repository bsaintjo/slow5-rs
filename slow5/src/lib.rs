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

use std::ffi::CString;

pub use aux::{Field, FieldType};
pub use compression::{Options, RecordCompression, SignalCompression};
pub use error::Slow5Error;
pub use header::{Header, HeaderView};
pub use reader::FileReader;
pub use record::{Record, RecordT, RecordBuilder, RecordExt, RecordIter, RecordView};
pub use writer::FileWriter;
pub use aux::RecordAuxiliaries;

pub use slow5_derive::FieldExt;

pub(crate) fn to_cstring<T: Into<Vec<u8>>>(x: T) -> Result<CString, Slow5Error> {
    CString::new(x).map_err(Slow5Error::InteriorNul)
}

pub trait FieldExt {
    fn set_header_aux_fields(header: &Header);
}