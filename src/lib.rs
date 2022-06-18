#![doc = include_str!("../README.md")]

pub mod error;
mod header;
pub mod reader;
pub mod record;
mod writer;

pub use error::Slow5Error;
pub use reader::FileReader;
pub use record::Record;
pub use record::RecordExt;
pub use record::RecordView;
pub use record::SignalIter;
pub use record::SignalIterExt;
