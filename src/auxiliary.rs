use std::ffi::CStr;

use libc::c_void;
use slow5lib_sys::{
    slow5_aux_get_char, slow5_aux_get_double, slow5_aux_get_enum, slow5_aux_get_float,
    slow5_aux_get_int16, slow5_aux_get_int32, slow5_aux_get_int64, slow5_aux_get_int8,
    slow5_aux_get_string, slow5_aux_get_uint16, slow5_aux_get_uint32, slow5_aux_get_uint64,
    slow5_aux_get_uint8, slow5_aux_set, slow5_aux_set_string, slow5_aux_type_SLOW5_CHAR,
    slow5_aux_type_SLOW5_DOUBLE, slow5_aux_type_SLOW5_DOUBLE_ARRAY, slow5_aux_type_SLOW5_ENUM,
    slow5_aux_type_SLOW5_FLOAT, slow5_aux_type_SLOW5_FLOAT_ARRAY, slow5_aux_type_SLOW5_INT16_T,
    slow5_aux_type_SLOW5_INT16_T_ARRAY, slow5_aux_type_SLOW5_INT32_T,
    slow5_aux_type_SLOW5_INT32_T_ARRAY, slow5_aux_type_SLOW5_INT64_T,
    slow5_aux_type_SLOW5_INT64_T_ARRAY, slow5_aux_type_SLOW5_INT8_T,
    slow5_aux_type_SLOW5_INT8_T_ARRAY, slow5_aux_type_SLOW5_STRING, slow5_aux_type_SLOW5_UINT16_T,
    slow5_aux_type_SLOW5_UINT16_T_ARRAY, slow5_aux_type_SLOW5_UINT32_T,
    slow5_aux_type_SLOW5_UINT32_T_ARRAY, slow5_aux_type_SLOW5_UINT64_T,
    slow5_aux_type_SLOW5_UINT8_T, slow5_aux_type_SLOW5_UINT8_T_ARRAY,
};

use crate::{to_cstring, FileWriter, Record, RecordExt, Slow5Error};

/// Maps between Rust types and SLOW5 C types
#[derive(Debug, Clone)]
pub enum FieldType {
    /// i8
    Int8,
    /// i16
    Int16,
    /// i32
    Int32,
    /// i64
    Int64,
    /// u8
    Uint8,
    /// u16
    Uint16,
    /// u32
    Uint32,
    /// u64
    Uint64,
    /// f32
    Float,
    /// f64
    Double,
    /// char
    Char,
    /// &str,
    Str,

    /// &[u8], not a string representation
    Uint8Array,

    /// &[u16]
    Uint16Array,

    /// &[u32]
    Uint32Array,

    /// &[u64]
    Uint64Array,

    /// &[i8]
    Int8Array,

    /// &[i16]
    Int16Array,

    /// &[i32]
    Int32Array,

    /// &[i64]
    Int64Array,

    /// &[f32]
    FloatArray,

    /// &[f64]
    DoubleArray,

    /// EnumField
    Enum(Vec<Vec<u8>>),
}

impl<B> From<Vec<B>> for FieldType
where
    B: Into<Vec<u8>>,
{
    fn from(value: Vec<B>) -> Self {
        FieldType::Enum(value.into_iter().map(|b| b.into()).collect())
    }
}

/// Wrapper around slow5lib-sys aux type
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
            FieldType::Str => slow5_aux_type_SLOW5_STRING,
            FieldType::DoubleArray => slow5_aux_type_SLOW5_DOUBLE_ARRAY,
            FieldType::FloatArray => slow5_aux_type_SLOW5_FLOAT_ARRAY,
            FieldType::Int8Array => slow5_aux_type_SLOW5_INT8_T_ARRAY,
            FieldType::Int16Array => slow5_aux_type_SLOW5_INT16_T_ARRAY,
            FieldType::Int32Array => slow5_aux_type_SLOW5_INT32_T_ARRAY,
            FieldType::Int64Array => slow5_aux_type_SLOW5_INT64_T_ARRAY,
            FieldType::Uint8Array => slow5_aux_type_SLOW5_UINT8_T_ARRAY,
            FieldType::Uint16Array => slow5_aux_type_SLOW5_UINT16_T_ARRAY,
            FieldType::Uint32Array => slow5_aux_type_SLOW5_UINT32_T_ARRAY,
            FieldType::Uint64Array => slow5_aux_type_SLOW5_INT64_T_ARRAY,
            FieldType::Enum(_) => slow5_aux_type_SLOW5_ENUM,
        })
    }
}

/// Represents the value for an enum field. This struct wraps an index into the
/// labels used for auxiiliary enum field.
///
/// The intended way to use is to index into the output from
/// [`AuxEnumlabelIter`]
///
/// [`AuxEnumLabelIter`]: crate::reader::AuxEnumLabelIter
pub struct EnumField(pub usize);

// TODO Use an associated type to separate FieldType from Enum related types
/// Helper trait to get auxiliary field values from [`Record`]
///
/// [`Record`]: crate::Record
pub trait AuxField {
    /// Get the auxiliary field with name from the Record
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
                    paste::paste!( [<slow5_aux_get_ $ctype:lower >] )(rec.ptr().ptr, name.as_ptr(), &mut ret)
                };
                if ret != 0 {
                    Err(Slow5Error::AuxLoadFailure)
                } else {
                    Ok(data)
                }
            }
        }

        impl AuxField for &[$rtype] {
            fn aux_get<B, R>(rec: &R, name: B) -> Result<Self, Slow5Error>
            where
                B: Into<Vec<u8>>,
                R: RecordExt,
            {
                use slow5lib_sys::*;
                let mut err = 0;
                let mut len = 0;
                let name: Vec<u8> = name.into();
                let name = crate::to_cstring(name)?;
                let data = unsafe {
                    paste::paste!( [<slow5_aux_get_ $ctype:lower _array>] )(rec.ptr().ptr, name.as_ptr(), &mut len, &mut err)
                };
                if err != 0 {
                    Err(Slow5Error::AuxLoadFailure)
                } else {
                    let data: &[$rtype] = unsafe { std::slice::from_raw_parts(data, len as usize) };
                    Ok(data)
                }
            }
        }
    };
}

impl_auxfield!(i8, Int8);
impl_auxfield!(i16, Int16);
impl_auxfield!(i32, Int32);
impl_auxfield!(i64, Int64);

impl_auxfield!(u8, Uint8);
impl_auxfield!(u16, Uint16);
impl_auxfield!(u32, Uint32);
impl_auxfield!(u64, Uint64);

impl_auxfield!(f32, Float);
impl_auxfield!(f64, Double);

impl AuxField for char {
    fn aux_get<B, R>(rec: &R, name: B) -> Result<Self, Slow5Error>
    where
        B: Into<Vec<u8>>,
        R: RecordExt,
    {
        let mut ret = 0;
        let name = to_cstring(name)?;
        let data = unsafe { slow5_aux_get_char(rec.ptr().ptr, name.as_ptr(), &mut ret) };
        if ret != 0 {
            Err(Slow5Error::AuxLoadFailure)
        } else {
            Ok(data as u8 as char)
        }
    }
}

impl AuxField for &str {
    fn aux_get<B, R>(rec: &R, name: B) -> Result<Self, Slow5Error>
    where
        B: Into<Vec<u8>>,
        R: RecordExt,
        Self: std::marker::Sized,
    {
        let mut err = 0;
        let mut len = 0;
        let name = to_cstring(name)?;
        let data =
            unsafe { slow5_aux_get_string(rec.ptr().ptr, name.as_ptr(), &mut len, &mut err) };
        let data = unsafe { CStr::from_ptr(data) };
        let data = data.to_str().map_err(Slow5Error::Utf8Error)?;
        Ok(data)
    }
}

impl AuxField for EnumField {
    fn aux_get<B, R>(rec: &R, name: B) -> Result<Self, Slow5Error>
    where
        B: Into<Vec<u8>>,
        R: RecordExt,
        Self: std::marker::Sized,
    {
        let mut err = 0;
        let name = to_cstring(name)?;
        let ef = unsafe { slow5_aux_get_enum(rec.ptr().ptr, name.as_ptr(), &mut err) };
        if err < 0 {
            Err(Slow5Error::Unknown)
        } else {
            Ok(EnumField(ef as usize))
        }
    }
}

/// Trait for values that we are allowed to set the values for in Records.
/// Currently only primitive types, strings, and enums are allowed to be used to
/// set auxiliary fields.
pub trait AuxFieldSetExt {
    fn aux_set<B>(
        &self,
        rec: &mut Record,
        field: B,
        writer: &mut FileWriter,
    ) -> Result<(), Slow5Error>
    where
        Self: Sized,
        B: Into<Vec<u8>>,
    {
        let name = to_cstring(field)?;
        let value_ptr = self as *const Self as *const c_void;
        let ret = unsafe {
            slow5_aux_set(
                rec.slow5_rec,
                name.as_ptr(),
                value_ptr,
                writer.header().header,
            )
        };
        writer.auxiliary_fields.push(name);
        if ret < 0 {
            Err(Slow5Error::SetAuxFieldError)
        } else {
            Ok(())
        }
    }
}

impl AuxFieldSetExt for u8 {}
impl AuxFieldSetExt for u16 {}
impl AuxFieldSetExt for u32 {}
impl AuxFieldSetExt for u64 {}
impl AuxFieldSetExt for i8 {}
impl AuxFieldSetExt for i16 {}
impl AuxFieldSetExt for i32 {}
impl AuxFieldSetExt for i64 {}
impl AuxFieldSetExt for f32 {}
impl AuxFieldSetExt for f64 {}

impl AuxFieldSetExt for &str {
    fn aux_set<B>(
        &self,
        rec: &mut Record,
        field: B,
        writer: &mut FileWriter,
    ) -> Result<(), Slow5Error>
    where
        B: Into<Vec<u8>>,
    {
        let name = to_cstring(field)?;
        let value_ptr = to_cstring(*self)?;
        let ret = unsafe {
            slow5_aux_set_string(
                rec.slow5_rec,
                name.as_ptr(),
                value_ptr.as_ptr(),
                writer.header().header,
            )
        };
        writer.auxiliary_fields.push(name);
        if ret < 0 {
            Err(Slow5Error::SetAuxFieldError)
        } else {
            Ok(())
        }
    }
}

impl AuxFieldSetExt for String {
    fn aux_set<B>(
        &self,
        rec: &mut Record,
        field: B,
        writer: &mut FileWriter,
    ) -> Result<(), Slow5Error>
    where
        B: Into<Vec<u8>>,
    {
        let name = to_cstring(field)?;
        let value_ptr = to_cstring(self.as_bytes())?;
        let ret = unsafe {
            slow5_aux_set_string(
                rec.slow5_rec,
                name.as_ptr(),
                value_ptr.as_ptr(),
                writer.header().header,
            )
        };
        writer.auxiliary_fields.push(name);
        if ret < 0 {
            Err(Slow5Error::SetAuxFieldError)
        } else {
            Ok(())
        }
    }
}

impl AuxFieldSetExt for EnumField {
    fn aux_set<B>(
        &self,
        rec: &mut Record,
        field: B,
        writer: &mut FileWriter,
    ) -> Result<(), Slow5Error>
    where
        Self: Sized,
        B: Into<Vec<u8>>,
    {
        if self.0 > (u8::MAX as usize) {
            Err(Slow5Error::TooManyLabels(self.0))
        } else {
            (self.0 as u8).aux_set(rec, field, writer)
        }
    }
}

// Seal the traits from downstream implementations
mod private {
    pub trait Sealed {}
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::FileReader;

    #[test]
    fn test_aux_get() -> anyhow::Result<()> {
        let reader = FileReader::open("examples/example2.slow5")?;
        let rec = reader.get_record("r0")?;
        let channel_number = <&str>::aux_get(&rec, "channel_number")?;
        assert_eq!(channel_number, "281");

        let rec = reader.get_record("r1")?;
        let channel_number: &str = rec.get_aux_field("channel_number")?;
        assert_eq!(channel_number, "391");

        let reader = FileReader::open("examples/example3.blow5")?;
        let rec = reader.get_record("0035aaf9-a746-4bbd-97c4-390ddc27c756")?;
        assert_eq!(rec.get_aux_field::<u64>("start_time").unwrap(), 335760788);
        assert_eq!(rec.get_aux_field::<i32>("read_number").unwrap(), 13875);

        assert!(rec.get_aux_field::<u8>("not real field").is_err());
        assert!(rec.get_aux_field::<i64>("read_number").is_err());
        Ok(())
    }
}
