use std::marker::PhantomData;

use slow5lib_sys::slow5_hdr_t;

use crate::error::Slow5Error;

pub(crate) struct HeaderView<'a> {
    header: *mut slow5_hdr_t,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> HeaderView<'a> {
    pub(crate) fn new(header: *mut slow5_hdr_t, _lifetime: PhantomData<&'a ()>) -> Self {
        Self { header, _lifetime }
    }

    fn attribute(&self) -> Result<String, Slow5Error> {
        unimplemented!()
    }
}

pub(crate) struct Header<'a> {
    header: *mut slow5_hdr_t,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> Header<'a> {
    fn add_attribute(&mut self, attr: &[u8]) -> Result<(), Slow5Error> {
        unimplemented!()
    }

    fn set_attribute_read_group(&mut self) -> Result<i64, Slow5Error> {
        unimplemented!()
    }

    fn add_aux_field<T>(&mut self, aux_field: T) -> Result<(), Slow5Error>
    where
        T: AuxField,
    {
        unimplemented!()
    }
}

trait AuxField: Clone {
    fn aux_meta(&self, name: &[u8], header: Header);
}
