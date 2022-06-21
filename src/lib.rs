#![doc = include_str!("../README.md")]
// #![warn(missing_docs)]

mod error;
mod header;
mod reader;
mod record;
mod writer;

use std::ffi::CString;

pub use error::Slow5Error;
pub use reader::FileReader;
pub use record::Record;
pub use record::RecordExt;
pub use record::RecordIter;
pub use record::RecordView;
pub use record::SignalIter;
pub use record::SignalIterExt;

pub(crate) fn to_cstring<T: Into<Vec<u8>>>(x: T) -> Result<CString, Slow5Error> {
    CString::new(x).map_err(Slow5Error::InteriorNul)
}
