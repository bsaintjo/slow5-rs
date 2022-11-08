use std::marker::PhantomData;

use crate::{record::RecPtr, Slow5Error, header::Header, Record};

enum AuxType {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    Float(f32),
    Double(f64),
    Char(char),
}

impl AuxType {
    fn from_rec<R>(rec: R, field: &str) -> Result<AuxType, Slow5Error> where R: RecPtr {
        todo!()
    }
}
pub(crate) struct Aux<'a, T> {
    name: String,
    header: &'a mut Header<'a>,
    _value: PhantomData<T>,
}
trait AuxField {
    fn aux_get(&self, rec: &Record, name: &str) -> Result<Self, Slow5Error>
    where
        Self: std::marker::Sized;
}

// macro_rules! impl_auxfield {
//     ($rtype:ty, $ctype:ident) => {
//         impl AuxField for $rtype {
//             fn aux_get(&self, rec: &Record, name: &str) -> Result<Self,
// Slow5Error> {                 let mut ret = 0;
//                 let name = CString::new(name).unwrap();
//                 // TODO try to use paste! from paste crate
//                 let data = unsafe { concat_idents!(slow5_aux_get_,
// $ctype)(rec.slow5_rec, name.as_ptr(), &mut ret) };                 if ret !=
// 0 {                     Err(Slow5Error::AuxLoadFailure)
//                 } else {
//                     Ok(data)
//                 }
//             }
//         }
//     };
// }
// impl_auxfield!(u64, uint64);

// impl AuxField for u64 {
//     fn aux_get(&self, rec: &Record, name: &str) -> Result<Self, Slow5Error> {
//         let mut ret = 0;
//         let name = CString::new(name).unwrap();
//         let data = unsafe { slow5_aux_get_uint64(rec.slow5_rec,
// name.as_ptr(), &mut ret) };         if ret != 0 {
//             Err(Slow5Error::AuxLoadFailure)
//         } else {
//             Ok(data)
//         }
//     }
// }