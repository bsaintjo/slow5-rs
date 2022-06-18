use slow5lib_sys::slow5_get;
use slow5lib_sys::slow5_rec_t;
use std::ffi::CString;
use std::marker::PhantomData;
use std::mem::size_of;

use slow5lib_sys::slow5_file;
use slow5lib_sys::slow5_hdr_t;

use cstr::cstr;

use crate::error::Slow5Error;
use crate::header::HeaderView;
use crate::record::Record;
use crate::record::RecordIter;

fn to_cstring<T: Into<Vec<u8>>>(x: T) -> Result<CString, Slow5Error> {
    CString::new(x).map_err(Slow5Error::InteriorNul)
}

pub struct FileReader {
    slow5_file: *mut slow5_file,
}

impl FileReader {
    fn new(slow5_file: *mut slow5_file) -> Self {
        Self { slow5_file }
    }

    // TODO Change to AsRef<Path>
    pub fn open<T: Into<Vec<u8>>>(filename: T) -> Result<Self, Slow5Error> {
        let filename = to_cstring(filename)?;
        let mode = cstr!("r");
        let slow5_file = unsafe { slow5lib_sys::slow5_open(filename.as_ptr(), mode.as_ptr()) };
        let ret = unsafe { slow5lib_sys::slow5_idx_load(slow5_file) };
        if ret == -1 {
            Err(Slow5Error::NoIndex)
        } else {
            Ok(FileReader::new(slow5_file))
        }
    }

    fn header(&self) -> HeaderView<'_> {
        let header: *mut slow5_hdr_t = unsafe { (*self.slow5_file).header };
        HeaderView::new(header, PhantomData)
    }

    pub fn records(&mut self) -> RecordIter<'_> {
        let slow5_rec_ptr =
            unsafe { libc::calloc(1, size_of::<slow5_rec_t>()) as *mut slow5_rec_t };
        RecordIter::new(slow5_rec_ptr, self.slow5_file)
    }

    pub fn get_record<T: Into<Vec<u8>>>(&self, read_id: T) -> Result<Record, Slow5Error> {
        let mut slow5_rec =
            unsafe { libc::calloc(1, size_of::<slow5_rec_t>()) as *mut slow5_rec_t };
        let read_id = to_cstring(read_id)?;
        let ret = unsafe { slow5_get(read_id.as_ptr(), &mut slow5_rec, self.slow5_file) };
        if ret >= 0 {
            Ok(Record::new(true, slow5_rec))
        } else {
            // TODO Handle error code properly
            Err(Slow5Error::Unknown)
        }
    }
}

impl Drop for FileReader {
    fn drop(&mut self) {
        unsafe {
            if !(*self.slow5_file).index.is_null() {
                slow5lib_sys::slow5_idx_unload(self.slow5_file);
            }
            slow5lib_sys::slow5_close(self.slow5_file);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_iter() {
        let filename = "examples/example.slow5";
        let mut reader = FileReader::open(filename).unwrap();
        let mut acc = Vec::new();
        for rec in reader.records() {
            acc.push(rec);
        }
        assert!(!acc.is_empty());
    }
}
