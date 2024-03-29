use std::{ffi::NulError, path::PathBuf, str::Utf8Error};

use thiserror::Error;

/// Errors from slow5 library
#[derive(Error, Debug)]
#[non_exhaustive]
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
    InteriorNul(#[from] NulError),

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

    /// Failed to add read group
    #[error("Failed to add new read group {0}")]
    FailedAddReadGroup(u32),

    /// Set number of read groups lower than inferred
    #[error("Inferred number of read groups {1} lower than n, {0}")]
    NumReadGroups(u32, u32),

    /// Failed to get Record from reader
    #[error("Failed to get record, read id not in FileReader")]
    GetRecordFailed,

    /// Failed to convert to UTF8
    #[error("Failed to convert to UTF8 {0}")]
    Utf8Error(#[from] Utf8Error),

    /// Compression was set but output was SLOW5. Only BLOW5 files are allowed
    /// to set compression options
    #[error("Compression was set but output is SLOW5.")]
    Slow5CompressionError,

    /// Invalid file path, the extension must be "slow5" or "blow5"
    #[error("Invalid file path: {0}")]
    InvalidFilePath(String),

    /// Failed to add attribute to header
    #[error("Failed to add attribute, error code {0}")]
    AddAttributeError(i32),

    /// Failed to set attribute to header
    #[error("Failed to set attribute, error code {0}")]
    SetAttributeError(i32),

    /// Failed to set auxiliary field
    #[error("Failed to set auxiliary field, error code {0}")]
    AddAuxFieldError(i32),

    /// Number of labels for an auxiliary enum must be less than u8::MAX
    #[error("Number of labels for an auxiliary enum must be less than u8::MAX, got {0}")]
    TooManyLabels(usize),

    /// Index given in EnumField is larger than number of labels
    #[error("Enum index out of range")]
    EnumOutOfRange,

    /// The attribute was not found within the header
    ///
    /// Common cases include:
    /// * Misspelled the name of the attribute
    /// * Did not add the auxiliary field in WriteOptions before creating file
    #[error("Attribute name was not found in header")]
    MissingAttribute,

    /// Type requested or given doesn't match type in SLOW5 file
    #[error("Invalid input, type mismatch")]
    AuxTypeMismatch,
}
