#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![cfg_attr(doc_cfg, feature(doc_cfg))]

mod auxiliary;
mod compression;
mod error;
mod header;
mod log;
mod reader;
mod record;
mod writer;

use std::ffi::CString;

pub use auxiliary::{AuxField, AuxFieldSetExt, EnumField, FieldType};
pub use compression::{RecordCompression, SignalCompression};
pub use error::Slow5Error;
pub use header::{AuxNamesIter, Header, HeaderExt};
pub use reader::{AuxEnumLabelIter, FileReader, ReadIdIter};
pub use record::{to_picoamps, to_raw_signal, Record, RecordBuilder, RecordExt, RecordIter};
pub use writer::{FileWriter, WriteOptions};

pub(crate) fn to_cstring<T: Into<Vec<u8>>>(x: T) -> Result<CString, Slow5Error> {
    CString::new(x).map_err(Slow5Error::InteriorNul)
}

#[cfg(doctest)]
doc_comment::doctest!("../README.md", readme);
