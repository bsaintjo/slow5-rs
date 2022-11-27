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

use std::ffi::{CStr, CString};

pub use aux::FieldType;
pub use compression::{RecordCompression, SignalCompression};
pub use error::Slow5Error;
pub use header::Header;
pub use reader::FileReader;
pub use record::{to_picoamps, to_raw_signal, Record, RecordBuilder, RecordExt, RecordIter};
pub use writer::{FileWriter, WriteOptions};

pub(crate) fn to_cstring<T: Into<Vec<u8>>>(x: T) -> Result<CString, Slow5Error> {
    CString::new(x).map_err(Slow5Error::InteriorNul)
}

pub(crate) fn to_cstr<T: AsRef<[u8]>>(x: &T) -> Result<&CStr, Slow5Error> {
    CStr::from_bytes_with_nul(x.as_ref()).map_err(Slow5Error::NulError)
}
