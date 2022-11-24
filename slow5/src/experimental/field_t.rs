use std::marker::PhantomData;

use slow5lib_sys::slow5_hdr_t;

use crate::{FieldType, Record, Slow5Error};

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

pub trait AuxFieldTExt {
    fn to_slow5_t() -> FieldType;
}

macro_rules! impl_auxfieldt {
    ($rt:ty, $ft:expr) => {
        impl AuxFieldTExt for $rt {
            fn to_slow5_t() -> FieldType {
                $ft
            }
        }
    };
}

impl_auxfieldt!(u8, FieldType::Uint8);
impl_auxfieldt!(u16, FieldType::Uint16);
impl_auxfieldt!(u32, FieldType::Uint32);
impl_auxfieldt!(u64, FieldType::Uint64);
impl_auxfieldt!(i8, FieldType::Int8);
impl_auxfieldt!(i16, FieldType::Int16);
impl_auxfieldt!(i32, FieldType::Int32);
impl_auxfieldt!(i64, FieldType::Int64);
impl_auxfieldt!(f32, FieldType::Float);
impl_auxfieldt!(f64, FieldType::Double);
impl_auxfieldt!(char, FieldType::Char);
