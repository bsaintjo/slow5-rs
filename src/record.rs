use libc::c_char;
use slow5lib_sys::slow5_file;
use slow5lib_sys::slow5_rec_free;
use slow5lib_sys::slow5_rec_t;
use std::ffi::CStr;
use std::marker::PhantomData;

use crate::error::Slow5Error;

struct RecordBuilder {
    read_id: Vec<u8>,
    read_group: usize,
    digitisation: f64,
    offset: f64,
    range: f64,
    sampling_rate: f64,
    raw_signal: Vec<f64>,
}

impl RecordBuilder {
    fn read_id(self, read_id: &[u8]) -> Self {
        unimplemented!()
    }

    fn read_group(self, read_group: usize) -> Self {
        unimplemented!()
    }

    // TODO check float type
    fn digitisation(self, digitisation: f64) -> Self {
        unimplemented!()
    }

    fn offset(self, offset: f64) -> Self {
        unimplemented!()
    }

    fn range(self, range: f64) -> Self {
        unimplemented!()
    }

    fn sampling_rate(self, sampling_rate: f64) -> Self {
        unimplemented!()
    }

    fn raw_signal(self, raw_signal: &[f64]) -> Self {
        unimplemented!()
    }

    fn build(self) -> Record {
        unimplemented!()
    }
}

pub struct Record {
    // TODO Figure out whether to keep picoamps
    picoamps: bool,
    slow5_rec: *mut slow5_rec_t,
}

impl Record {
    pub(crate) fn new(picoamps: bool, slow5_rec: *mut slow5_rec_t) -> Self {
        Self {
            picoamps,
            slow5_rec,
        }
    }
}

impl Drop for Record {
    fn drop(&mut self) {
        unsafe {
            slow5_rec_free(self.slow5_rec);
        }
    }
}

pub struct RecordView<'a> {
    slow5_rec: *mut slow5_rec_t,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> RecordView<'a> {
    fn new(slow5_rec: *mut slow5_rec_t) -> Self {
        Self {
            slow5_rec,
            _lifetime: PhantomData,
        }
    }
}

pub trait RecordExt {
    fn read_id(&self) -> &str;
}

impl<'a> RecordExt for RecordView<'a> {
    fn read_id(&self) -> &str {
        let str_ptr: *mut c_char = unsafe { (*self.slow5_rec).read_id };
        let read_id = unsafe { CStr::from_ptr(str_ptr) };

        read_id.to_str().unwrap()
    }
}

impl RecordExt for Record {
    fn read_id(&self) -> &str {
        let str_ptr: *mut c_char = unsafe { (*self.slow5_rec).read_id };
        let read_id = unsafe { CStr::from_ptr(str_ptr) };

        read_id.to_str().unwrap()
    }
}

pub struct RecordIter<'a> {
    slow5_rec_ptr: *mut slow5_rec_t,
    slow5_file: *mut slow5_file,
    errored: bool,
    _lifetime: PhantomData<&'a ()>,
}

/// Iterator over records of a Slow5 file
impl<'a> RecordIter<'a> {
    pub(crate) fn new(slow5_rec_ptr: *mut slow5_rec_t, slow5_file: *mut slow5_file) -> Self {
        Self {
            slow5_rec_ptr,
            slow5_file,
            errored: false,
            _lifetime: PhantomData,
        }
    }
}

impl<'a> Iterator for RecordIter<'a> {
    type Item = Result<RecordView<'a>, Slow5Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = unsafe { slow5lib_sys::slow5_get_next(&mut self.slow5_rec_ptr, self.slow5_file) };
        if self.errored {
            None
        } else if ret >= 0 {
            Some(Ok(RecordView::new(self.slow5_rec_ptr)))
        } else if ret == -1 {
            None
        } else if ret == -2 {
            self.errored = true;
            Some(Err(Slow5Error::Argument))
        } else if ret == -4 {
            self.errored = true;
            Some(Err(Slow5Error::RecordParse))
        } else {
            // -5
            // for now just put everything under this.
            self.errored = true;
            Some(Err(Slow5Error::IOError))
        }
    }
}

impl<'a> Drop for RecordIter<'a> {
    fn drop(&mut self) {
        unsafe { slow5_rec_free(self.slow5_rec_ptr) }
    }
}

fn to_picoamps(raw_val: f64, digitisation: f64, offset: f64, range: f64) -> f64 {
    ((raw_val) + offset) * (range / digitisation)
}

/// Iterator over signal data from Record
pub struct SignalIter<'a> {
    picoamps: bool,
    i: u64,
    read: *mut slow5_rec_t,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> SignalIter<'a> {
    fn new(picoamps: bool, read: *mut slow5_rec_t) -> Self {
        Self {
            picoamps,
            i: 0,
            read,
            _lifetime: PhantomData,
        }
    }
}

impl<'a> Iterator for SignalIter<'a> {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.i < (*self.read).len_raw_signal {
                let mut signal = *(*self.read).raw_signal.offset(self.i as isize) as f64;
                if self.picoamps {
                    signal = to_picoamps(
                        signal,
                        (*self.read).digitisation,
                        (*self.read).offset,
                        (*self.read).range,
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

// TODO maybe combine this with RecordExt
pub trait SignalIterExt {
    fn signal_iter(&self) -> SignalIter<'_>;
}

impl SignalIterExt for Record {
    fn signal_iter(&self) -> SignalIter<'_> {
        SignalIter::new(self.picoamps, self.slow5_rec)
    }
}

impl<'a> SignalIterExt for RecordView<'a> {
    fn signal_iter(&self) -> SignalIter<'_> {
        SignalIter::new(true, self.slow5_rec)
    }
}
