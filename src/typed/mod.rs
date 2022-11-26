use crate::Header;

pub mod reader;
pub mod record;

pub use slow5_derive::FieldExt;

pub trait FieldExt {
    fn set_header_aux_fields(header: &Header);
}

impl FieldExt for () {
    fn set_header_aux_fields(_header: &Header) {}
}
