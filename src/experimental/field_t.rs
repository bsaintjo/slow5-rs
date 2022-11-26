use std::marker::PhantomData;

use slow5lib_sys::slow5_hdr_t;

use crate::{Record, Slow5Error};

pub struct Field<'a, T> {
    name: Vec<u8>,
    header: *mut slow5_hdr_t,
    _value: PhantomData<T>,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a, T> Field<'a, T> {
    pub(crate) fn new(name: Vec<u8>, header: *mut slow5_hdr_t) -> Self {
        Self {
            name,
            header,
            _value: PhantomData,
            _lifetime: PhantomData,
        }
    }

    pub(crate) fn name(&self) -> &[u8] {
        &self.name
    }

    pub(crate) fn header_ptr(&self) -> *mut slow5_hdr_t {
        self.header
    }

    pub(crate) fn aux_set(&self, rec: &mut Record, val: T) -> Result<(), Slow5Error> {
        todo!()
    }
}
