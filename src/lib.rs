#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

mod auxiliary;
mod compression;
mod error;
mod header;
mod reader;
mod record;
mod writer;

use std::ffi::CString;

pub use auxiliary::{AuxField, EnumField, FieldType};
pub use compression::{RecordCompression, SignalCompression};
pub use error::Slow5Error;
pub use header::{AuxNamesIter, Header, HeaderExt};
pub use reader::{FileReader, ReadIdIter};
pub use record::{to_picoamps, to_raw_signal, Record, RecordBuilder, RecordExt, RecordIter};
pub use writer::{FileWriter, WriteOptions};

pub(crate) fn to_cstring<T: Into<Vec<u8>>>(x: T) -> Result<CString, Slow5Error> {
    CString::new(x).map_err(Slow5Error::InteriorNul)
}

#[cfg(doctest)]
doc_comment::doctest!("../README.md", readme);
