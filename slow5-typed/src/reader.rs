//! Provides a FileReader generic over the auxiliary fields.
use std::{
    ffi::{CStr, CString},
    marker::PhantomData,
    mem::size_of,
    os::unix::prelude::OsStrExt,
    path::Path,
};

use cstr::cstr;
use libc::c_char;
use slow5lib_sys::{slow5_file_t, slow5_get, slow5_get_rids, slow5_hdr_t, slow5_rec_t};

use crate::to_cstring;

use super::{header::Header, FieldExt};
use slow5::{Record, RecordIter, Slow5Error};

/// Read from a SLOW5 file
pub struct FileReader<A = ()> {
    pub(crate) slow5_file: *mut slow5_file_t,
    aux: PhantomData<A>,
}

impl<A: FieldExt> FileReader<A> {
    pub(crate) fn new(slow5_file: *mut slow5_file_t) -> Self {
        Self {
            slow5_file,
            aux: PhantomData,
        }
    }

    /// Open a SLOW5 file, creates an index if one doesn't exist.
    ///
    /// # Example
    /// ```
    /// # use std::error::Error;
    /// use slow5::typed::FileReader;
    ///
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let reader: FileReader<()> = FileReader::open("examples/example.slow5")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn open<P: AsRef<Path>>(file_path: P) -> Result<Self, Slow5Error> {
        // If we aren't testing or running in debug mode, silence slow5lib logs
        #[cfg(any(not(test), not(debug_assertions)))]
        unsafe {
            slow5lib_sys::slow5_set_log_level(slow5lib_sys::slow5_log_level_opt_SLOW5_LOG_OFF);
        }

        let file_path = file_path.as_ref().as_os_str().as_bytes();
        let file_path = to_cstring(file_path)?;
        let mode = cstr!("r");
        let slow5_file: *mut slow5_file_t =
            unsafe { slow5lib_sys::slow5_open(file_path.as_ptr(), mode.as_ptr()) };
        let ret = unsafe { slow5lib_sys::slow5_idx_load(slow5_file) };
        if ret == -1 {
            Err(Slow5Error::NoIndex)
        } else {
            let reader = FileReader::new(slow5_file);
            Ok(reader)
        }
    }

    /// Access header of a SLOW5 file
    pub fn header(&self) -> Header<'_, A> {
        let header: *mut slow5_hdr_t = unsafe { (*self.slow5_file).header };
        Header::new(header)
    }

    /// Return iterator over each read in a SLOW5 file as a [`RecordIter`].
    ///
    /// # Example
    /// ```ignore
    /// # use std::error::Error;
    /// # use slow5::typed::FileReader;
    /// use slow5::RecordExt;
    ///
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// # let mut reader: FileReader<()> = FileReader::open("examples/example.slow5")?;
    /// for record in reader.records() {
    ///     println!("{:?}", record?.read_id());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    // TODO I could avoid the reader being consumed by using rewinding the file ptr
    // and making it &mut self. RecordIter would need to do the rewinding once its
    // finished.
    // pub fn records(&mut self) -> RecordIter {
    //     RecordIter::new(self.slow5_file)
    // }

    /// Random-access a single [`Record`] by read_id.
    ///
    /// # Example
    /// ```ignore
    /// # use slow5::typed::FileReader;
    /// # use std::error::Error;
    /// use slow5::RecordExt;
    ///
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// # let reader: FileReader<()> = FileReader::open("examples/example.slow5")?;
    /// let read_id = "r3";
    /// let record = reader.get_record(read_id)?;
    /// assert_eq!(record.read_id(), read_id.as_bytes());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Mutating the Record will not cause changes in the SLOW5 file.
    // pub fn get_record<B: Into<Vec<u8>>>(&self, read_id: B) -> Result<Record, Slow5Error> {
    //     let mut slow5_rec =
    //         unsafe { libc::calloc(1, size_of::<slow5_rec_t>()) as *mut slow5_rec_t };
    //     let read_id = to_cstring(read_id)?;
    //     let rid_ptr = read_id.into_raw();
    //     let ret = unsafe { slow5_get(rid_ptr, &mut slow5_rec, self.slow5_file) };
    //     let _ = unsafe { CString::from_raw(rid_ptr) };
    //     if ret >= 0 {
    //         Ok(Record::new(slow5_rec))
    //     } else {
    //         // TODO Handle error code properly
    //         Err(Slow5Error::Unknown)
    //     }
    // }

    /// Returns iterator over all the read ids in a SLOW5 file
    /// ```ignore
    /// # use slow5::typed::FileReader;
    /// use std::str;
    ///
    /// let slow5: FileReader<()> = FileReader::open("examples/example.slow5").unwrap();
    /// # let mut read_ids = Vec::new();
    /// let read_id_iter = slow5.iter_read_ids().unwrap();
    /// for rid in read_id_iter {
    ///     println!("{}", str::from_utf8(rid).unwrap());
    /// #   read_ids.push(rid);
    /// }
    /// # assert_eq!(read_ids.len(), 5);
    /// # assert_eq!(read_ids[0], b"r1");
    /// # assert_eq!(read_ids[1], b"r2");
    /// ```
    // TODO figure out how to seek back after
    // Records has to take ownership because the file pointer is changed during
    // iteration Maybe ideal to fseek + other with the fp after dropping the
    // RecordIter
    // pub fn iter_read_ids(&self) -> Result<ReadIdIter<'_>, Slow5Error> {
    //     ReadIdIter::new(self)
    // }
    fn _placeholder() {}
}

impl<A> Drop for FileReader<A> {
    fn drop(&mut self) {
        unsafe {
            slow5lib_sys::slow5_close(self.slow5_file);
        }
    }
}

/// Iterator over Read IDs for a typed [`FileReader`]
pub struct ReadIdIter<'a> {
    idx: u64,
    num_reads: u64,
    read_id_ptr: *mut *mut c_char,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> ReadIdIter<'a> {
    fn new<A>(reader: &FileReader<A>) -> Result<Self, Slow5Error> {
        let mut num_reads = 0;
        let rids = unsafe { slow5_get_rids(reader.slow5_file, &mut num_reads) };
        if rids.is_null() || num_reads == 0 {
            Err(Slow5Error::ReadIdIterError)
        } else {
            Ok(ReadIdIter {
                idx: 0,
                num_reads,
                read_id_ptr: rids,
                _lifetime: PhantomData,
            })
        }
    }
}

impl<'a> Iterator for ReadIdIter<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.num_reads {
            let rid = unsafe { self.read_id_ptr.offset(self.idx as isize) };
            let rid = unsafe { CStr::from_ptr(*rid) };
            self.idx += 1;
            Some(rid.to_bytes())
        } else {
            None
        }
    }
}

// #[cfg(test)]
// mod test {
//     use super::*;
//     use crate::RecordExt;

//     #[test]
//     fn test_reader() {
//         let filename = "examples/example.slow5";
//         let mut reader: FileReader<()> = FileReader::open(filename).unwrap();

//         let read_id = "r3";
//         let rec = reader.get_record(read_id).unwrap();
//         assert_eq!(rec.read_id(), read_id.as_bytes());

//         let mut acc = Vec::new();
//         for rec in reader.records() {
//             acc.push(rec);
//         }
//         assert!(!acc.is_empty());
//     }
// }
