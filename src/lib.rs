#![doc = include_str!("../README.md")]

extern crate slow5lib_sys;

use std::ffi::CStr;
use std::ffi::CString;
use std::ffi::NulError;
use std::os::unix::prelude::OsStrExt;
use std::path::Path;
use std::ptr::null_mut;

use cstr::cstr;
use thiserror::Error;

fn to_cstring(x: &[u8]) -> Result<CString, Slow5Error> {
    CString::new(x).map_err(Slow5Error::InteriorNul)
}

/// Builder for reading Slow5 files
pub struct Builder {
    picoamps: bool,
    aux: bool,
}

impl Builder {
    fn new(picoamps: bool, aux: bool) -> Self {
        Self { picoamps, aux }
    }

    /// Set whether raw signal measurements should be converted into picocamps
    pub fn picoamps(self, picoamps: bool) -> Self {
        Builder { picoamps, ..self }
    }

    #[doc(hidden)]
    fn aux(self, aux: bool) -> Self {
        Builder { aux, ..self }
    }

    /// Returns a Slow5 from the file path, will return an error if file_path
    /// contains a interior nul byte or it is unable to load the index
    pub fn open<P>(&self, file_path: P) -> Result<Slow5, Slow5Error>
    where
        P: AsRef<Path>,
    {
        let file_path = file_path.as_ref().as_os_str().as_bytes();
        let file_path = to_cstring(file_path)?;
        let mode = cstr!("r");
        let sp = unsafe { slow5lib_sys::slow5_open(file_path.as_ptr(), mode.as_ptr()) };
        let ret = unsafe { slow5lib_sys::slow5_idx_load(sp) };
        if ret < 0 {
            Err(Slow5Error::NoIndex)
        } else {
            Ok(Slow5::new(self.picoamps, self.aux, sp))
        }
    }
}

impl Default for Builder {
    fn default() -> Self {
        Builder::new(false, false)
    }
}

fn to_picoamps(raw_val: f64, digitisation: f64, offset: f64, range: f64) -> f64 {
    ((raw_val) + offset) * (range / digitisation)
}

/// Slow5 file, obtain from Builder::open
pub struct Slow5 {
    picoamps: bool,
    aux: bool,
    slow5_file: *mut slow5lib_sys::slow5_file_t,
}

impl Slow5 {
    fn new(picoamps: bool, aux: bool, slow5_file: *mut slow5lib_sys::slow5_file_t) -> Self {
        Self {
            picoamps,
            aux,
            slow5_file,
        }
    }

    /// Return iterator over each read in the slow5 file.
    pub fn seq_reads(&mut self) -> ReadIter {
        let rec: *mut slow5lib_sys::slow5_rec_t = null_mut();
        ReadIter::new(rec, self)
    }

    // TODO: test if this needs to be &mut self
    // slow5_get takes a *mut slow5_file_t but not sure if
    // that means I necessarily need to make this mutating
    /// Returns slow5 read with the corresponding read identifier, will return
    /// error if read_id contains an interior nul byte or IO error occurs
    pub fn get_read(&mut self, read_id: &[u8]) -> Result<Slow5Record, Slow5Error> {
        let mut slow_rec_ptr: *mut slow5lib_sys::slow5_rec_t = null_mut();
        let read_id = to_cstring(read_id)?;
        let rec_mut_ptr: *mut *mut slow5lib_sys::slow5_rec_t = &mut slow_rec_ptr;
        let ret =
            unsafe { slow5lib_sys::slow5_get(read_id.as_ptr(), rec_mut_ptr, self.slow5_file) };
        if ret < 0 {
            // TODO: Return appropriate error based on return code
            Err(Slow5Error::IOError)
        } else {
            Ok(Slow5Record::new(self.picoamps, slow_rec_ptr))
        }
    }

    #[doc(hidden)]
    fn get_header_names(&self, read_group: usize) {
        unimplemented!()
    }

    #[doc(hidden)]
    fn get_aux_names(&self) {
        unimplemented!()
    }

    #[doc(hidden)]
    fn get_aux_types(&self) {
        unimplemented!()
    }
}

impl Drop for Slow5 {
    fn drop(&mut self) {
        unsafe {
            slow5lib_sys::slow5_idx_unload(self.slow5_file);
            slow5lib_sys::slow5_close(self.slow5_file);
        }
    }
}

/// Iterator over each read in a Slow5
pub struct ReadIter<'a> {
    rec: *mut slow5lib_sys::slow5_rec_t,
    slow5: &'a mut Slow5,
}

impl<'a> ReadIter<'a> {
    fn new(rec: *mut slow5lib_sys::slow5_rec_t, slow5: &'a mut Slow5) -> Self {
        Self { rec, slow5 }
    }
}

impl<'a> Iterator for ReadIter<'a> {
    type Item = Result<Slow5Rec, Slow5Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = unsafe { slow5lib_sys::slow5_get_next(&mut self.rec, self.slow5.slow5_file) };
        if ret >= 0 {
            Some(Ok(Slow5Rec::new(self.slow5.picoamps, self.rec)))
        } else if ret == -1 {
            None
        } else {
            // TODO: Give out correct error based on return code
            // for now just put everything under this.
            Some(Err(Slow5Error::IOError))
        }
    }
}

impl<'a> Drop for ReadIter<'a> {
    fn drop(&mut self) {
        unsafe {
            slow5lib_sys::slow5_rec_free(self.rec);
        }
    }
}

/// Errors from slow5 library
#[derive(Error, Debug)]
pub enum Slow5Error {
    #[error("Unable to load index")]
    NoIndex,
    #[error("IO error")]
    IOError,
    #[error("Read identifier not found in index {0}")]
    ReadIDNotInIndex(String),
    #[error("String passed with interior nul byte: {0}")]
    InteriorNul(NulError),
}

/// Represents an owned SLOW5 record
pub struct Slow5Record {
    picoamps: bool,
    slow5_rec: *mut slow5lib_sys::slow5_rec_t,
}

impl Slow5Record {
    fn new(picoamps: bool, slow5_rec: *mut slow5lib_sys::slow5_rec_t) -> Self {
        Self {
            picoamps,
            slow5_rec,
        }
    }

    /// When iterating over signal, set whether to covert signal into picoamps,
    /// a convience function if you set picoamps in Builder but want to change
    /// to raw signal later on.
    pub fn picoamps(self, picoamps: bool) -> Self {
        Self { picoamps, ..self }
    }

    /// Return read identifier of Slow5Read
    pub fn read_id(&self) -> &[u8] {
        unsafe { CStr::from_ptr((*self.slow5_rec).read_id).to_bytes() }
    }

    fn as_borrowed(&self) -> Slow5Rec {
        Slow5Rec::from_owned(self)
    }

    // /// Return iterator over signal measurements
    pub fn signal_iter(&self) -> SignalIter {
        SignalIter::new(0, self.as_borrowed())
    }
}

impl Drop for Slow5Record {
    fn drop(&mut self) {
        unsafe {
            slow5lib_sys::slow5_rec_free(self.slow5_rec);
        }
    }
}

/// Representation of a borrowed Slow5Record
///
/// [^note]: This does not impl Drop and only used for functions that take care of the
/// deallocation.
pub struct Slow5Rec {
    picoamps: bool,
    slow5_rec: *mut slow5lib_sys::slow5_rec_t,
}

impl Slow5Rec {
    fn new(picoamps: bool, slow5_rec: *mut slow5lib_sys::slow5_rec_t) -> Self {
        Self {
            picoamps,
            slow5_rec,
        }
    }

    fn from_owned(rec: &Slow5Record) -> Slow5Rec {
        Slow5Rec::new(rec.picoamps, rec.slow5_rec)
    }

    pub fn signal_iter(self) -> SignalIter {
        SignalIter::new(0, self)
    }
}

/// Iterator over signal from Slow5Read
pub struct SignalIter {
    i: u64,
    read: Slow5Rec,
}

impl SignalIter {
    fn new(i: u64, read: Slow5Rec) -> Self {
        Self { i, read }
    }
}

impl Iterator for SignalIter {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.i < (*self.read.slow5_rec).len_raw_signal {
                let mut signal = *(*self.read.slow5_rec).raw_signal.offset(self.i as isize) as f64;
                if self.read.picoamps {
                    signal = to_picoamps(
                        signal,
                        (*self.read.slow5_rec).digitisation,
                        (*self.read.slow5_rec).offset,
                        (*self.read.slow5_rec).range,
                    );
                }
                self.i += 1;
                Some(signal as f64)
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Builder;

    #[test]
    fn test_builder_setters() {
        let builder = Builder::default();
        assert!(!builder.picoamps);
        assert!(!builder.aux);

        let builder = builder.picoamps(true);
        assert!(builder.picoamps);
        assert!(!builder.aux);

        let builder = builder.aux(true);
        assert!(builder.picoamps);
        assert!(builder.aux);

        let builder = builder.aux(false);
        assert!(builder.picoamps);
        assert!(!builder.aux);
    }
}
