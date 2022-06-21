use slow5lib_sys::slow5_get;
use slow5lib_sys::slow5_rec_t;
use std::marker::PhantomData;
use std::mem::size_of;
use std::os::unix::prelude::OsStrExt;
use std::path::Path;

use slow5lib_sys::slow5_file_t;
use slow5lib_sys::slow5_hdr_t;

use cstr::cstr;

use crate::error::Slow5Error;
use crate::header::HeaderView;
use crate::record::Record;
use crate::record::RecordIter;
use crate::to_cstring;

/// Read from a SLOW5 file
pub struct FileReader {
    pub(crate) slow5_file: *mut slow5_file_t,
}

impl FileReader {
    fn new(slow5_file: *mut slow5_file_t) -> Self {
        Self { slow5_file }
    }

    /// Open a SLOW5 file, creates an index if one doesn't exist.
    /// # Example
    /// ```
    /// # use std::error::Error;
    /// use slow5::FileReader;
    ///
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let reader = FileReader::open("examples/example.slow5")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn open<P: AsRef<Path>>(file_path: P) -> Result<Self, Slow5Error> {
        let file_path = file_path.as_ref().as_os_str().as_bytes();
        let file_path = to_cstring(file_path)?;
        let mode = cstr!("r");
        let slow5_file: *mut slow5_file_t =
            unsafe { slow5lib_sys::slow5_open(file_path.as_ptr(), mode.as_ptr()) };
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

    /// Return iterator over each read in a SLOW5 file as a [`RecordIter`].
    /// # Example
    /// ```
    /// # use std::error::Error;
    /// # use slow5::FileReader;
    /// use slow5::RecordExt;
    ///
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// # let mut reader = FileReader::open("examples/example.slow5")?;
    /// for record in reader.records() {
    ///     println!("{:?}", record?.read_id());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn records(self) -> RecordIter {
        let slow5_rec_ptr =
            unsafe { libc::calloc(1, size_of::<slow5_rec_t>()) as *mut slow5_rec_t };
        RecordIter::new(slow5_rec_ptr, self)
    }

    /// Random-access a single [`Record`] by read_id
    /// # Example
    /// ```
    /// # use slow5::FileReader;
    /// # use std::error::Error;
    /// use slow5::RecordExt;
    ///
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// # let reader = FileReader::open("examples/example.slow5")?;
    /// let read_id = b"r3";
    /// let record = reader.get_record(read_id)?;
    /// assert_eq!(record.read_id(), read_id);
    /// # Ok(())
    /// # }
    /// ```
    /// 
    /// Mutating the Record will not cause changes in the SLOW5 file.
    pub fn get_record(&self, read_id: &[u8]) -> Result<Record, Slow5Error> {
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
    use crate::RecordExt;

    use super::*;

    #[test]
    fn test_reader() {
        let filename = "examples/example.slow5";
        let reader = FileReader::open(filename).unwrap();

        let read_id = b"r3";
        let rec = reader.get_record(read_id).unwrap();
        assert_eq!(rec.read_id(), read_id);

        let mut acc = Vec::new();
        for rec in reader.records() {
            acc.push(rec);
        }
        assert!(!acc.is_empty());
    }
}
