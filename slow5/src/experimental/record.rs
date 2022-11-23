#![allow(dead_code)]
//! Alternate API following a pattern more similarly to PathBuf/&Path and
//! OsString/&OsStr Record is an owned type and Rec represents the borrowed
//! type, and you can only get &Rec
use std::{ffi::CStr, mem::transmute, ops::Deref};

use libc::{c_char, c_void};
use slow5lib_sys::{slow5_aux_set, slow5_rec_free, slow5_rec_t};

use super::field_t::Field;
use crate::{aux::AuxField, Slow5Error};

macro_rules! rec_getter {
    ($field:ident, $ftype:ty) => {
        pub fn $field(&self) -> $ftype {
            unsafe { (*self.rec_ptr.0).$field }
        }
    };
}

struct RecPtr(*mut slow5_rec_t);

pub struct Record {
    rec_ptr: RecPtr,
}

impl Record {
    fn set_aux_field<T>(&mut self, aux: &Field<T>, value: &T) -> Result<(), Slow5Error>
    where
        T: AuxField,
    {
        let value: *const c_void = unsafe { transmute(&value) };
        let name = aux.name().as_ptr() as *const c_char;
        let ret = unsafe { slow5_aux_set(self.rec_ptr.0, name, value, aux.header_ptr()) };
        if ret < 0 {
            Err(Slow5Error::SetAuxFieldError)
        } else {
            Ok(())
        }
    }
}

impl Deref for Record {
    type Target = Rec;

    fn deref(&self) -> &Self::Target {
        Rec::from_inner(&self.rec_ptr)
    }
}

impl AsRef<Rec> for Record {
    fn as_ref(&self) -> &Rec {
        self
    }
}

impl Drop for Record {
    fn drop(&mut self) {
        unsafe { slow5_rec_free(self.rec_ptr.0) }
    }
}

pub struct Rec {
    rec_ptr: RecPtr,
}

impl Rec {
    pub fn new<R: AsRef<Rec>>(r: &R) -> &Rec {
        r.as_ref()
    }

    fn from_inner(inner: &RecPtr) -> &Rec {
        unsafe { &*(inner as *const RecPtr as *const Rec) }
    }

    fn ptr(&self) -> *mut slow5_rec_t {
        self.rec_ptr.0
    }

    pub fn read_id(&self) -> &[u8] {
        let str_ptr: *mut c_char = unsafe { (*self.ptr()).read_id };
        let read_id = unsafe { CStr::from_ptr(str_ptr) };

        read_id.to_bytes()
    }

    rec_getter!(digitisation, f64);
    rec_getter!(offset, f64);
    rec_getter!(range, f64);
    rec_getter!(sampling_rate, f64);
    rec_getter!(read_group, u32);
    rec_getter!(len_raw_signal, u64);
}
