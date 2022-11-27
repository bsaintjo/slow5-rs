
pub mod reader;
pub mod record;
pub mod header;

pub use header::Header;
pub use reader::FileReader;
pub use slow5_derive::FieldExt;

pub trait FieldExt {
    fn set_header_aux_fields(header: &Header<Self>) where Self: Sized;
}

impl FieldExt for () {
    fn set_header_aux_fields(_header: &Header<Self>) {}
}
