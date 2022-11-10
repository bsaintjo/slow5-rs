use slow5lib_sys::{slow5_aux_get_char, slow5_hdr_t};
use std::{ffi::CString, marker::PhantomData};

use slow5lib_sys::{
    slow5_aux_get_double, slow5_aux_get_float, slow5_aux_get_int16, slow5_aux_get_int32,
    slow5_aux_get_int64, slow5_aux_get_int8, slow5_aux_get_uint16, slow5_aux_get_uint32,
    slow5_aux_get_uint64, slow5_aux_get_uint8,
};

use crate::{header::Header, Record, Slow5Error};

pub struct Field<'a, T> {
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
    pub fn aux_get(&self, rec: &Record) -> Result<T, Slow5Error>
    where
        T: AuxField,
    {
        T::aux_get(rec, self.name.as_str())
    }
}
pub trait AuxField {
    fn aux_get<B>(rec: &Record, name: B) -> Result<Self, Slow5Error>
    where
        B: Into<Vec<u8>>,
        Self: std::marker::Sized;
}

macro_rules! impl_auxfield {
    ($rtype:ty, $ctype:ident) => {
        impl AuxField for $rtype {
            fn aux_get<B>(rec: &Record, name: B) -> Result<Self, Slow5Error> where B: Into<Vec<u8>> {
                let mut ret = 0;
                let name: Vec<u8> = name.into();
                let name = CString::new(name).map_err(Slow5Error::InteriorNul)?;
                let data = unsafe {
                    paste::paste!( [<slow5_aux_get_ $ctype>] )(rec.slow5_rec, name.as_ptr(), &mut ret)
                };
                if ret != 0 {
                    Err(Slow5Error::AuxLoadFailure)
                } else {
                    Ok(data)
                }
            }
        }
    };
}

impl_auxfield!(i8, int8);
impl_auxfield!(i16, int16);
impl_auxfield!(i32, int32);
impl_auxfield!(i64, int64);

impl_auxfield!(u8, uint8);
impl_auxfield!(u16, uint16);
impl_auxfield!(u32, uint32);
impl_auxfield!(u64, uint64);

impl_auxfield!(f32, float);
impl_auxfield!(f64, double);

impl AuxField for char {
    fn aux_get<B>(rec: &Record, name: B) -> Result<Self, Slow5Error>
    where
        B: Into<Vec<u8>>,
    {
        let mut ret = 0;
        let name = CString::new(name.into()).unwrap();
        let data = unsafe { slow5_aux_get_char(rec.slow5_rec, name.as_ptr(), &mut ret) };
        if ret != 0 {
            Err(Slow5Error::AuxLoadFailure)
        } else {
            Ok(data as u8 as char)
        }
    }
}
