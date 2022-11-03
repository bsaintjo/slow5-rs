use std::{
    ffi::{CStr, CString},
    marker::PhantomData,
};

use slow5lib_sys::{slow5_hdr_get, slow5_hdr_t};

use crate::error::Slow5Error;

/// Get an immutable access to the headers of a SLOW5 file.
pub struct HeaderView<'a> {
    header: *mut slow5_hdr_t,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> HeaderView<'a> {
    pub(crate) fn new(header: *mut slow5_hdr_t, _lifetime: PhantomData<&'a ()>) -> Self {
        Self { header, _lifetime }
    }

    /// Get the value of an attribute in a read group
    /// ```
    /// use slow5::FileReader;
    ///
    /// let slow5 = FileReader::open("examples/example.slow5").unwrap();
    /// let header = slow5.header();
    /// let attr = header.attribute(0, "run_id").unwrap();
    /// # assert_eq!(attr, "d6e473a6d513ec6bfc150c60fd4556d72f0e6d18");
    /// ```
    // TODO how to handle allocated string from slow5_hdr_get
    pub fn attribute<S: Into<Vec<u8>>>(
        &self,
        read_group: u32,
        attr: S,
    ) -> Result<&str, Slow5Error> {
        let attr = CString::new(attr).unwrap();
        let rg_value = unsafe { slow5_hdr_get(attr.as_ptr(), read_group, self.header) };
        if !rg_value.is_null() {
            let cstr = unsafe { CStr::from_ptr(rg_value) };
            Ok(cstr.to_str().unwrap())
        } else {
            Err(Slow5Error::Unknown)
        }
    }
}

pub(crate) struct Header<'a> {
    header: *mut slow5_hdr_t,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> Header<'a> {
    pub(crate) fn new(header: *mut slow5_hdr_t) -> Self {
        Self {
            header,
            _lifetime: PhantomData,
        }
    }

    fn add_attribute(&mut self, attr: &[u8]) -> Result<(), Slow5Error> {
        unimplemented!()
    }

    fn set_attribute_read_group(&mut self) -> Result<i64, Slow5Error> {
        unimplemented!()
    }

    pub(crate) fn add_aux_field<S, T>(&'a mut self, name: S) -> Result<Aux<'a, T>, Slow5Error>
    where
        S: Into<String>,
    {
        todo!();
        Ok(Aux {
            name: name.into(),
            header: self,
            _value: PhantomData,
        })
    }
}

pub(crate) struct Aux<'a, T> {
    name: String,
    header: &'a mut Header<'a>,
    _value: PhantomData<T>,
}

trait HeaderExt {}

trait AuxField: Clone {
    fn aux_meta(&self, name: &[u8], header: Header);
}
