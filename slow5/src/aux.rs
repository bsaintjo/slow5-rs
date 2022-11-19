use std::{ffi::CString, marker::PhantomData};

use slow5lib_sys::{
    slow5_aux_get_char, slow5_aux_get_double, slow5_aux_get_float, slow5_aux_get_int16,
    slow5_aux_get_int32, slow5_aux_get_int64, slow5_aux_get_int8, slow5_aux_get_uint16,
    slow5_aux_get_uint32, slow5_aux_get_uint64, slow5_aux_get_uint8, slow5_aux_type_SLOW5_CHAR,
    slow5_aux_type_SLOW5_DOUBLE, slow5_aux_type_SLOW5_FLOAT, slow5_aux_type_SLOW5_INT16_T,
    slow5_aux_type_SLOW5_INT32_T, slow5_aux_type_SLOW5_INT64_T, slow5_aux_type_SLOW5_INT8_T,
    slow5_aux_type_SLOW5_UINT16_T, slow5_aux_type_SLOW5_UINT32_T, slow5_aux_type_SLOW5_UINT64_T,
    slow5_aux_type_SLOW5_UINT8_T, slow5_hdr_t,
};

use crate::{header::Header, record::RecordT, Record, RecordExt, Slow5Error};

#[derive(Debug, Clone, Copy)]
pub enum FieldType {
    Int8,
    Int16,
    Int32,
    Int64,
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    Float,
    Double,
    Char,
}

pub(crate) struct Slow5AuxType(pub(crate) u32);

impl FieldType {
    pub(crate) fn to_slow5_t(&self) -> Slow5AuxType {
        Slow5AuxType(match self {
            FieldType::Int8 => slow5_aux_type_SLOW5_INT8_T,
            FieldType::Int16 => slow5_aux_type_SLOW5_INT16_T,
            FieldType::Int32 => slow5_aux_type_SLOW5_INT32_T,
            FieldType::Int64 => slow5_aux_type_SLOW5_INT64_T,
            FieldType::Uint8 => slow5_aux_type_SLOW5_UINT8_T,
            FieldType::Uint16 => slow5_aux_type_SLOW5_UINT16_T,
            FieldType::Uint32 => slow5_aux_type_SLOW5_UINT32_T,
            FieldType::Uint64 => slow5_aux_type_SLOW5_UINT64_T,
            FieldType::Float => slow5_aux_type_SLOW5_FLOAT,
            FieldType::Double => slow5_aux_type_SLOW5_DOUBLE,
            FieldType::Char => slow5_aux_type_SLOW5_CHAR,
        })
    }
}

#[derive(Clone)]
pub struct Field<'a> {
    name: Vec<u8>,
    header: &'a Header<'a>,
    field_t: FieldType,
}

impl<'a> Field<'a> {
    pub(crate) fn new<B>(name: B, header: &'a Header<'a>, field_t: FieldType) -> Self
    where
        B: Into<Vec<u8>>,
    {
        Self {
            name: name.into(),
            header,
            field_t,
        }
    }

    pub fn field_t(&self) -> FieldType {
        self.field_t
    }

    pub fn name(&self) -> &[u8] {
        &self.name
    }

    pub(crate) fn header_ptr(&self) -> *mut slow5_hdr_t {
        self.header.header
    }
    // // TODO easy to implement but is it worth having?
    // pub fn aux_get<T>(&self, rec: &Record) -> Result<T, Slow5Error>
    // where
    //     T: AuxField,
    // {
    //     T::aux_get(rec, self.name.clone())
    // }
}
pub trait AuxField {
    fn aux_get<B, R>(rec: &R, name: B) -> Result<Self, Slow5Error>
    where
        B: Into<Vec<u8>>,
        R: RecordExt,
        Self: std::marker::Sized;
}

macro_rules! impl_auxfield {
    ($rtype:ty, $ctype:ident) => {
        impl AuxField for $rtype {
            fn aux_get<B, R>(rec: &R, name: B) -> Result<Self, Slow5Error>
            where
                B: Into<Vec<u8>>,
                R: RecordExt,
            {
                let mut ret = 0;
                let name: Vec<u8> = name.into();
                let name = crate::to_cstring(name)?;
                let data = unsafe {
                    paste::paste!( [<slow5_aux_get_ $ctype>] )(rec.ptr().ptr, name.as_ptr(), &mut ret)
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
    fn aux_get<B, R>(rec: &R, name: B) -> Result<Self, Slow5Error>
    where
        B: Into<Vec<u8>>,
        R: RecordExt,
    {
        let mut ret = 0;
        let name = CString::new(name.into()).unwrap();
        let data = unsafe { slow5_aux_get_char(rec.ptr().ptr, name.as_ptr(), &mut ret) };
        if ret != 0 {
            Err(Slow5Error::AuxLoadFailure)
        } else {
            Ok(data as u8 as char)
        }
    }
}

pub struct RecordAuxiliaries<'a, A> {
    rec: &'a RecordT<A>,
}

impl<'a, A> RecordAuxiliaries<'a, A> {
    pub(crate) fn new(rec: &'a RecordT<A>) -> Self {
        Self { rec }
    }
}
