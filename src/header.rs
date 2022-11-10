use std::{
    ffi::{CStr, CString},
    marker::PhantomData,
};

use libc::c_char;
use slow5lib_sys::{slow5_get_aux_names, slow5_hdr_get, slow5_hdr_t};

use crate::{aux::Field, error::Slow5Error};

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
    /// let attr = header.attribute("run_id", 0).unwrap();
    /// assert_eq!(attr, "d6e473a6d513ec6bfc150c60fd4556d72f0e6d18");
    /// ```
    // TODO how to handle allocated string from slow5_hdr_get
    pub fn attribute<S: Into<Vec<u8>>>(
        &self,
        attr: S,
        read_group: u32,
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

pub struct Header<'a> {
    pub(crate) header: *mut slow5_hdr_t,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> Header<'a> {
    pub(crate) fn new(header: *mut slow5_hdr_t) -> Self {
        Self {
            header,
            _lifetime: PhantomData,
        }
    }

    fn get_attribute(&self, read_group: u32) -> Result<&[u8], Slow5Error> {
        todo!()
    }

    fn add_attribute(&mut self, attr: &[u8]) -> Result<(), Slow5Error> {
        unimplemented!()
    }

    fn set_attribute(
        &mut self,
        attr: &[u8],
        value: &[u8],
        read_group: u32,
    ) -> Result<(), Slow5Error> {
        unimplemented!()
    }

    /// Return iterator over auxiliary field names
    pub fn aux_names_iter(&self) -> Result<AuxNamesIter, Slow5Error> {
        let mut num_aux = 0;
        let auxs = unsafe { slow5_get_aux_names(self.header, &mut num_aux) };
        if auxs.is_null() || num_aux == 0 {
            Err(Slow5Error::AuxNameIterError)
        } else {
            Ok(AuxNamesIter::new(0, num_aux, auxs))
        }
    }

    /// Add auxiliary field to header, and return a [`Field`] that can be
    /// used for setting the auxiliary field of [`crate::Record`].
    pub(crate) fn add_aux_field<B, T>(&'a mut self, name: B) -> Result<Field<'a, T>, Slow5Error>
    where
        B: Into<Vec<u8>>,
    {
        todo!();
    }
}

/// Iterator over auxiliary field names of a [`Header`], usually using [`aux_names_iter`]
/// 
/// [`aux_names_iter`]: crate::Header::aux_names_iter
pub struct AuxNamesIter<'a> {
    idx: u64,
    num_aux: u64,
    auxs: *mut *mut c_char,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> AuxNamesIter<'a> {
    fn new(idx: u64, num_aux: u64, auxs: *mut *mut c_char) -> Self {
        Self {
            idx,
            num_aux,
            auxs,
            _lifetime: PhantomData,
        }
    }
}

impl<'a> Iterator for AuxNamesIter<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.num_aux {
            let aux_name = unsafe { self.auxs.offset(self.idx as isize) };
            let aux_name = unsafe { CStr::from_ptr(*aux_name) };
            Some(aux_name.to_bytes())
        } else {
            None
        }
    }
}
