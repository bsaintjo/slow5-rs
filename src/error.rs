use std::{
    ffi::{FromBytesWithNulError, NulError},
    path::PathBuf,
};

use thiserror::Error;

/// Errors from slow5 library
#[derive(Error, Debug)]
pub enum Slow5Error {
    /// No index was loaded for SLOW5 file
    #[error("Unable to load index")]
    NoIndex,

    /// Input/output error
    #[error("IO error")]
    IOError,

    /// Read ID not found in index
    #[error("Read identifier not found in index {0}")]
    ReadIDNotInIndex(String),

    /// Interior NUL byte inside String
    #[error("String passed with interior nul byte: {0}")]
    InteriorNul(NulError),

    /// Incorrect argument
    #[error("Bad argument")]
    Argument,

    /// Failed to parse Record
    #[error("Record parsing error")]
    RecordParse,

    /// Failed to convert numeric value
    #[error("Failed to convert integer type due to overflow")]
    Conversion,

    /// Faild to allocate memory
    #[error("Failed to allocate space")]
    Allocation,

    /// Unexpected null pointer argument
    #[error("Unexpected null pointer given as argument")]
    NullArgument,

    /// Unknown error, report if you see this
    #[error("Unknown error")]
    Unknown,

    /// Duplicate Read ID found
    #[error("Read ID {0} is a duplicate")]
    DuplicateReadId(String),

    /// Failed to write header to SLOW5 file
    #[error("Failed to write header to SLOW5 file")]
    HeaderWriteFailed,

    /// Failed to load auxiliary field
    #[error("Failed to load auxiliary field")]
    AuxLoadFailure,

    /// Error setting compression method
    #[error("Error setting compression method")]
    CompressionError,

    /// Either no SLOW5 index or read ID list not in index
    #[error("Either no SLOW5 index or read ID list not in index")]
    ReadIdIterError,

    /// Error in getting list of auxiliary field names
    #[error("Error in getting list of auxiliary field names")]
    AuxNameIterError,

    /// Error setting auxiliary field
    #[error("Error setting auxiliary field")]
    SetAuxFieldError,

    /// File path does not exist
    #[error("Input file path does not exist {0}")]
    IncorrectPath(PathBuf),

    /// Error getting attribute
    #[error("Error getting attribute, attribute doesn't exist or read_group is out of range")]
    AttributeError,

    /// NUL not in correct place
    #[error("NUL not in correct place ")]
    NulError(FromBytesWithNulError),

    /// Failed to add read group
    #[error("Failed to add new read group {0}")]
    FailedAddReadGroup(u32),

    /// Set number of read groups lower than inferred
    #[error("Inferred number of read groups {1} lower than n, {0}")]
    NumReadGroups(u32, u32),

    /// Failed to get Record from reader
    #[error("Failed to get record, read id not in FileReader")]
    GetRecordFailed,
}
