use std::ffi::NulError;

use thiserror::Error;

/// Errors from slow5 library
#[derive(Error, Debug)]
pub enum Slow5Error {
    #[error("Unable to load index")]
    NoIndex,
    #[error("IO error")]
    IOError,
    #[error("Read identifier not found in index {0}")]
    ReadIDNotInIndex(String),
    #[error("String passed with interior nul byte: {0}")]
    InteriorNul(NulError),
    #[error("Bad argument")]
    Argument,
    #[error("Record parsing error")]
    RecordParse,
    #[error("Failed to convert integer type due to overflow")]
    Conversion,
    #[error("Failed to allocate space")]
    Allocation,
    #[error("Unexpected null pointer given as argument")]
    NullArgument,
    #[error("Unknown error")]
    Unknown,
    #[error("Read ID {0} is a duplicate")]
    DuplicateReadId(String),
}
