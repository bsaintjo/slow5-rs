use std::{
    ffi::{CStr, CString},
    marker::PhantomData,
};

use libc::c_char;
use slow5lib_sys::{
    slow5_aux_add, slow5_get_aux_names, slow5_hdr_add, slow5_hdr_get, slow5_hdr_set, slow5_hdr_t,
};

use crate::{
    aux::{AuxField, FieldType},
    error::Slow5Error,
    to_cstring,
};

/// Represents a SLOW5 header generic over the auxiliary fields
pub struct Header<'a, A> {
    pub(crate) header: *mut slow5_hdr_t,
    _lifetime: PhantomData<&'a ()>,
    _aux: PhantomData<A>,
}

impl<'a, A> Header<'a, A> {
    pub(crate) fn new(header: *mut slow5_hdr_t) -> Self {
        Self {
            header,
            _lifetime: PhantomData,
            _aux: PhantomData,
        }
    }

    /// Get the value of an attribute in a read group
    /// ```
    /// use slow5::typed::FileReader;
    ///
    /// let slow5: FileReader<()> = FileReader::open("examples/example.slow5").unwrap();
    /// let header = slow5.header();
    /// let attr = header.get_attribute("run_id", 0).unwrap();
    /// assert_eq!(attr, b"d6e473a6d513ec6bfc150c60fd4556d72f0e6d18");
    /// ```
    pub fn get_attribute<S: Into<Vec<u8>>>(
        &self,
        attr: S,
        read_group: u32,
    ) -> Result<&[u8], Slow5Error> {
        let attr = CString::new(attr).unwrap();
        let rg_value = unsafe { slow5_hdr_get(attr.as_ptr(), read_group, self.header) };
        if !rg_value.is_null() {
            let cstr = unsafe { CStr::from_ptr(rg_value) };
            Ok(cstr.to_bytes())
        } else {
            Err(Slow5Error::Unknown)
        }
    }

    /// Add attribute to header.
    pub fn add_attribute<B>(&mut self, attr: B) -> Result<(), Slow5Error>
    where
        B: Into<Vec<u8>>,
    {
        let attr = to_cstring(attr.into())?;
        let ret = unsafe { slow5_hdr_add(attr.as_ptr(), self.header) };
        if ret < 0 {
            Err(Slow5Error::Unknown)
        } else {
            Ok(())
        }
    }

    /// Set value for given attribute and read group
    pub fn set_attribute<B, C>(
        &mut self,
        attr: B,
        value: C,
        read_group: u32,
    ) -> Result<(), Slow5Error>
    where
        B: Into<Vec<u8>>,
        C: Into<Vec<u8>>,
    {
        let attr = to_cstring(attr.into())?;
        let value = to_cstring(value.into())?;
        let ret = unsafe { slow5_hdr_set(attr.as_ptr(), value.as_ptr(), read_group, self.header) };
        if ret < 0 {
            Err(Slow5Error::Unknown)
        } else {
            Ok(())
        }
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

    /// Add auxiliary field to header used for setting the auxiliary field
    /// of [`crate::typed::record::RecordT`].
    pub fn add_aux_field<B>(&mut self, name: B, field_type: FieldType) -> Result<(), Slow5Error>
    where
        B: Into<Vec<u8>>,
    {
        let name = to_cstring(name)?;
        let ret = unsafe { slow5_aux_add(name.as_ptr(), field_type.to_slow5_t().0, self.header) };
        if ret < 0 {
            Err(Slow5Error::Unknown)
        } else {
            Ok(())
        }
    }

    /// Add auxiliary field to header used for setting the auxiliary field
    /// of [`crate::typed::record::RecordT`]. Infer the type T from type arguments instead of
    /// directly passing a [`FieldType`].
    pub fn add_aux_field_t<B, T>(&'a self, name: B) -> Result<(), Slow5Error>
    where
        B: Into<Vec<u8>> + Clone,
        T: AuxField,
    {
        let cname = to_cstring(name)?;
        let field_type = T::to_slow5_t();
        let ret = unsafe { slow5_aux_add(cname.as_ptr(), field_type.to_slow5_t().0, self.header) };
        if ret < 0 {
            Err(Slow5Error::Unknown)
        } else {
            Ok(())
        }
    }
}

/// Iterator over auxiliary field names of a [`Header`], usually using
/// [`aux_names_iter`]
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
