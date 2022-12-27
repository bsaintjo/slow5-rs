use std::{
    ffi::{CStr, CString},
    marker::PhantomData,
    mem::size_of,
    os::unix::prelude::OsStrExt,
    path::Path,
};

use cstr::cstr;
use libc::{c_char, c_void};
use slow5lib_sys::{
    slow5_file_t, slow5_get, slow5_get_aux_enum_labels, slow5_get_hdr_keys, slow5_get_rids,
    slow5_hdr_t, slow5_rec_t,
};

use crate::{
    error::Slow5Error,
    header::HeaderExt,
    record::{Record, RecordIter},
    to_cstring, Header, RecordCompression, SignalCompression,
};

/// Read from a SLOW5 file
pub struct FileReader {
    pub(crate) slow5_file: *mut slow5_file_t,
}

unsafe impl Send for FileReader {}

impl std::fmt::Debug for FileReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileReader").finish()
    }
}

impl FileReader {
    fn new(slow5_file: *mut slow5_file_t) -> Self {
        Self { slow5_file }
    }

    /// Open a SLOW5 file, creates an index if one doesn't exist.
    ///
    /// # Example
    /// ```
    /// use slow5::FileReader;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let reader = FileReader::open("examples/example.slow5")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn open<P: AsRef<Path>>(file_path: P) -> Result<Self, Slow5Error> {
        // If we aren't testing or running in debug mode, silence slow5lib logs
        #[cfg(any(not(test), not(debug_assertions)))]
        unsafe {
            slow5lib_sys::slow5_set_log_level(slow5lib_sys::slow5_log_level_opt_SLOW5_LOG_OFF);
        }
        let file_path = file_path.as_ref();
        if !file_path.exists() {
            log::error!("File path doesn't exist: {file_path:?}");
            return Err(Slow5Error::IncorrectPath(file_path.to_owned()));
        }

        let file_path = file_path.as_os_str().as_bytes();
        let file_path = to_cstring(file_path)?;
        let mode = cstr!("r");
        let slow5_file: *mut slow5_file_t =
            unsafe { slow5lib_sys::slow5_open(file_path.as_ptr(), mode.as_ptr()) };
        let ret = unsafe { slow5lib_sys::slow5_idx_load(slow5_file) };
        if ret == -1 {
            log::error!("No index was loaded");
            Err(Slow5Error::NoIndex)
        } else {
            Ok(FileReader::new(slow5_file))
        }
    }

    /// Get file's record compression
    pub fn record_compression(&self) -> RecordCompression {
        let compress = unsafe { (*self.slow5_file).compress };
        if compress.is_null() {
            return RecordCompression::None;
        }
        let record_press = unsafe { (*(*compress).record_press).method };
        RecordCompression::from_u32(record_press)
    }

    /// Get file's signal compression
    pub fn signal_compression(&self) -> SignalCompression {
        let compress = unsafe { (*self.slow5_file).compress };
        if compress.is_null() {
            return SignalCompression::None;
        }
        let signal_press = unsafe { (*(*compress).signal_press).method };
        SignalCompression::from_u32(signal_press)
    }

    /// Access header of a SLOW5 file
    pub fn header(&self) -> Header<'_> {
        let header: *mut slow5_hdr_t = unsafe { (*self.slow5_file).header };
        Header::new(header)
    }

    /// Return iterator over each read in a SLOW5 file as a [`RecordIter`].
    ///
    /// # Example
    /// ```
    /// # use slow5::FileReader;
    /// use slow5::RecordExt;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// # let mut reader = FileReader::open("examples/example.slow5")?;
    /// for record in reader.records() {
    ///     println!("{:?}", record?.read_id());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    // TODO I could avoid the reader being consumed by using rewinding the file ptr
    // and making it &mut self. RecordIter would need to do the rewinding once its
    // finished.
    pub fn records(&mut self) -> RecordIter {
        RecordIter::new(self)
    }

    /// Random-access a single [`Record`] by read_id.
    ///
    /// # Example
    /// ```
    /// # use slow5::FileReader;
    /// use slow5::RecordExt;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// # let reader = FileReader::open("examples/example.slow5")?;
    /// let read_id: &[u8] = b"r3";
    /// let record = reader.get_record(read_id)?;
    /// assert_eq!(record.read_id(), read_id);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Mutating the Record will not cause changes in the SLOW5 file.
    pub fn get_record<B>(&self, read_id: B) -> Result<Record, Slow5Error>
    where
        B: Into<Vec<u8>>,
    {
        let mut slow5_rec =
            unsafe { libc::calloc(1, size_of::<slow5_rec_t>()) as *mut slow5_rec_t };
        let read_id = to_cstring(read_id)?;
        let rid_ptr = read_id.into_raw();
        let ret = unsafe { slow5_get(rid_ptr, &mut slow5_rec, self.slow5_file) };
        let _ = unsafe { CString::from_raw(rid_ptr) };
        if ret >= 0 {
            Ok(Record::new(slow5_rec))
        } else {
            // TODO Handle error code properly
            unsafe { libc::free(slow5_rec as *mut c_void) };
            Err(Slow5Error::GetRecordFailed)
        }
    }

    /// Returns iterator over all the read ids in a SLOW5 file
    /// ```
    /// # use slow5::FileReader;
    /// use std::str;
    ///
    /// let slow5 = FileReader::open("examples/example.slow5").unwrap();
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
    pub fn iter_read_ids(&self) -> Result<ReadIdIter<'_>, Slow5Error> {
        ReadIdIter::new(self)
    }

    /// Returns iterator over the labels for an enum auxiliary field
    ///
    /// # Errors
    /// Return Err if field's type is not an auxiliary enum or field is not
    /// within in the header
    // TODO Example
    pub fn iter_aux_enum_labels<B>(&self, field: B) -> Result<AuxEnumLabelIter, Slow5Error>
    where
        B: Into<Vec<u8>>,
    {
        let mut n = 0;
        let field = to_cstring(field)?;
        let label_ptr =
            unsafe { slow5_get_aux_enum_labels(self.header().header, field.as_ptr(), &mut n) };
        if label_ptr.is_null() {
            Err(Slow5Error::Unknown)
        } else {
            Ok(AuxEnumLabelIter::new(self, label_ptr, n))
        }
    }

    /// Returns iterator over attribute keys for all read groups
    // TODO Example
    pub fn iter_attr_keys(&self) -> Result<AttrKeysIter, Slow5Error> {
        let mut n = 0;
        let keys = unsafe { slow5_get_hdr_keys(self.header().header, &mut n) };
        if keys.is_null() {
            Err(Slow5Error::Unknown)
        } else {
            Ok(AttrKeysIter {
                _reader: self,
                keys,
                n_keys: n,
                idx: 0,
            })
        }
    }
}

impl HeaderExt for FileReader {
    fn header(&self) -> Header<'_> {
        self.header()
    }
}

impl Drop for FileReader {
    fn drop(&mut self) {
        unsafe {
            slow5lib_sys::slow5_close(self.slow5_file);
        }
    }
}

/// Iterator over all the read IDs in a SLOW5 file
// TODO Is liftime fine, or should I just hold the &'a FileReader?
pub struct ReadIdIter<'a> {
    idx: u64,
    num_reads: u64,
    read_id_ptr: *mut *mut c_char,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> std::fmt::Debug for ReadIdIter<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReadIdIter")
            .field("idx", &self.idx)
            .field("num_reads", &self.num_reads)
            .finish()
    }
}

impl<'a> ReadIdIter<'a> {
    fn new(reader: &FileReader) -> Result<Self, Slow5Error> {
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

/// Iterator over labels for an auxiliary field enum
pub struct AuxEnumLabelIter<'a> {
    _reader: &'a FileReader,
    label_ptr: *mut *mut c_char,
    n: u8,
    idx: u8,
}

impl<'a> AuxEnumLabelIter<'a> {
    fn new(_reader: &'a FileReader, label_ptr: *mut *mut c_char, n: u8) -> Self {
        AuxEnumLabelIter {
            _reader,
            label_ptr,
            n,
            idx: 0,
        }
    }
}
impl<'a> std::fmt::Debug for AuxEnumLabelIter<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuxEnumLabelIter")
            .field("idx", &self.idx)
            .field("n_labels", &self.n)
            .finish()
    }
}

impl<'a> Iterator for AuxEnumLabelIter<'a> {
    type Item = &'a [u8];
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.n {
            let label = unsafe { self.label_ptr.offset(self.idx as isize) };
            let label = unsafe { CStr::from_ptr(*label) };
            self.idx += 1;
            Some(label.to_bytes())
        } else {
            None
        }
    }
}

pub struct AttrKeysIter<'a> {
    _reader: &'a FileReader,
    keys: *mut *const c_char,
    n_keys: u64,
    idx: u64,
}

impl<'a> std::fmt::Debug for AttrKeysIter<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuxEnumLabelIter")
            .field("idx", &self.idx)
            .field("n_keys", &self.n_keys)
            .finish()
    }
}

impl<'a> Iterator for AttrKeysIter<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.n_keys {
            let attr_key = unsafe { self.keys.offset(self.idx as isize) };
            let attr_key = unsafe { CStr::from_ptr(*attr_key) };
            self.idx += 1;
            Some(attr_key.to_bytes())
        } else {
            None
        }
    }
}

impl<'a> Drop for AttrKeysIter<'a> {
    fn drop(&mut self) {
        unsafe { libc::free(self.keys as *mut c_void) }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::RecordExt;

    #[test]
    fn test_reader() {
        let filename = "examples/example.slow5";
        let mut reader = FileReader::open(filename).unwrap();

        let read_id: &[u8] = b"r3";
        let rec = reader.get_record(read_id).unwrap();
        assert_eq!(rec.read_id(), read_id);

        let mut acc = Vec::new();
        for rec in reader.records() {
            acc.push(rec);
        }
        assert!(!acc.is_empty());
    }

    #[test]
    fn test_bad_path() {
        let filename = "random_fileoufnseif";
        let reader = FileReader::open(filename);
        assert!(matches!(reader, Err(Slow5Error::IncorrectPath(_))));
    }

    #[test]
    fn test_no_compression() {
        let filename = "examples/example.slow5";
        let reader = FileReader::open(filename).unwrap();
        assert_eq!(reader.record_compression(), RecordCompression::None);
        assert_eq!(reader.signal_compression(), SignalCompression::None);
    }
}
