use std::marker::PhantomData;

use slow5lib_sys::slow5_hdr_t;

use crate::{header::Header, Slow5Error, aux::AuxField, RecordExt};

use super::record::Record;

pub(crate) struct Field<'a, T> {
    name: String,
    header: &'a mut Header<'a>,
    _value: PhantomData<T>,
}

impl<'a, T> Field<'a, T> {
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn header_ptr(&self) -> *mut slow5_hdr_t {
        self.header.header
    }
    // TODO easy to implement but is it worth having?
    pub fn aux_getR<R>(&self, rec: &R) -> Result<T, Slow5Error>
    where
        T: AuxField,
        R: RecordExt,
    {
        T::aux_get(rec, self.name.as_str())
    }
}