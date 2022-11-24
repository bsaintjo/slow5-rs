use std::{ffi::NulError, path::PathBuf};

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
    #[error("Failed to write header to SLOW5 file")]
    HeaderWriteFailed,
    #[error("Failed to load auxiliary field")]
    AuxLoadFailure,
    #[error("Error setting compression method")]
    CompressionError,
    #[error("Either no slow index or read ID list not in index")]
    ReadIdIterError,
    #[error("Error in getting list of aux field names")]
    AuxNameIterError,
    #[error("Error setting auxiliary field")]
    SetAuxFieldError,
    #[error("Input file path does not exist {0}")]
    IncorrectPath(PathBuf),
    #[error("Error getting attribute, attribute doesn't exist or read_group is out of range")]
    AttributeError,
}
