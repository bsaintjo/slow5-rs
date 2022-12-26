//! Module implement a FileReader, FileWriter, Record, etc. that are generic
//! over a type representing auxiliary fields.
mod header;
pub mod reader;
pub mod record;

use std::ffi::CString;

pub use header::Header;
pub use reader::FileReader;
use slow5::Slow5Error;
pub use slow5_derive::FieldExt;

/// Represents a trait for auxiliary types that set the header field.
/// Usually automatically implemented using the FieldExt derive macro.
pub trait FieldExt {
    /// Set the auxiliary fields for a header. Types representing auxiliary
    /// fields implement this so FileWriter<A> will set the auxiliary fields
    /// at initiliazation.
    fn set_header_aux_fields(header: &Header<Self>)
    where
        Self: Sized;
}

impl FieldExt for () {
    fn set_header_aux_fields(_header: &Header<Self>) {}
}

pub(crate) fn to_cstring<T: Into<Vec<u8>>>(x: T) -> Result<CString, Slow5Error> {
    CString::new(x)
}