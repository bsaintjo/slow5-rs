use slow5lib_sys::slow5_add_rec;
use slow5lib_sys::slow5_fmt_SLOW5_FORMAT_ASCII;
use slow5lib_sys::slow5_init_empty;
use std::{os::unix::prelude::OsStrExt, path::Path};

use slow5lib_sys::slow5_file;

use crate::RecordExt;
use crate::{record::RecordBuilder, to_cstring, record::Record, Slow5Error};

use cstr::cstr;

pub struct FileWriter {
    slow5_file: *mut slow5_file,
}

impl FileWriter {
    fn new(slow5_file: *mut slow5_file) -> Self {
        Self { slow5_file }
    }

    /// Create a new SLOW5 file, if one already exists, file will be written over
    /// # Example
    /// ```
    /// # use anyhow::Result;
    /// use slow5::FileWriter;
    /// # use slow5::Slow5Error;
    /// # use assert_fs::TempDir;
    /// # use assert_fs::fixture::PathChild;
    /// # fn main() -> Result<()> {
    /// # let tmp_dir = TempDir::new()?;
    /// let file_path = "test.slow5";
    /// # let file_path = tmp_dir.child(file_path);
    /// # let tmp_path = file_path.to_path_buf();
    /// let mut writer = FileWriter::create(file_path)?;
    /// # assert!(tmp_path.exists());
    /// # Ok(())
    /// # }
    /// ```
    pub fn create<P>(file_path: P) -> Result<Self, Slow5Error>
    where
        P: AsRef<Path>,
    {
        let file_path = file_path.as_ref().as_os_str().as_bytes();
        let file_path = to_cstring(file_path)?;
        let mode = cstr!("w");

        let slow5_file = unsafe {
            let fp = libc::fopen(file_path.as_ptr(), mode.as_ptr());
            slow5_init_empty(fp, file_path.as_ptr(), slow5_fmt_SLOW5_FORMAT_ASCII)
        };

        if slow5_file.is_null() {
            return Err(Slow5Error::Allocation);
        }

        let ret = unsafe { slow5lib_sys::slow5_idx_load(slow5_file) };
        if ret == -1 {
            Err(Slow5Error::NoIndex)
        } else {
            Ok(Self::new(slow5_file))
        }
    }

    /// Add [`Record`] to SLOW5 file, not thread safe
    /// # Example
    /// ```
    /// # use anyhow::Result;
    /// # use slow5::FileWriter;
    /// # use slow5::FileReader;
    /// # use slow5::Slow5Error;
    /// # use assert_fs::TempDir;
    /// # use assert_fs::fixture::PathChild;
    /// # use slow5::RecordBuilder;
    /// # fn main() -> Result<()> {
    /// # let tmp_dir = TempDir::new()?;
    /// # let file_path = "test.slow5";
    /// # let file_path = tmp_dir.child(file_path);
    /// # let mut writer = FileWriter::create(&file_path)?;
    /// let rec = RecordBuilder::default().read_id(b"test").build()?;
    /// writer.add_record(&rec)?;
    /// # drop(writer);
    /// # assert!(file_path.exists());
    /// # let reader = FileReader::open(&file_path)?;
    /// # let rec = reader.get_record(b"test")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_record(&mut self, record: &Record) -> Result<(), Slow5Error> {
        let ret = unsafe { slow5_add_rec(record.slow5_rec, self.slow5_file) };
        if ret == 0 {
            Ok(())
        } else if ret == -1 {
            Err(Slow5Error::NullArgument)
        } else if ret == -2 {
            Err(Slow5Error::DuplicateReadId(
                String::from_utf8(record.read_id().to_vec()).unwrap(),
            ))
        } else {
            Err(Slow5Error::Unknown)
        }
    }

    /// Write record to SLOW5 file, with a closure that makes a one-shot [`RecordBuilder`],
    /// an alternative to add_record. Not thread safe
    /// # Example
    /// ```
    /// # use anyhow::Result;
    /// # use slow5::FileWriter;
    /// # use slow5::Slow5Error;
    /// # use assert_fs::TempDir;
    /// # use assert_fs::fixture::PathChild;
    /// # use slow5::RecordBuilder;
    /// # fn main() -> Result<()> {
    /// # let tmp_dir = TempDir::new()?;
    /// # let file_path = "test.slow5";
    /// # let file_path = tmp_dir.child(file_path);
    /// # let tmp_path = file_path.to_path_buf();
    /// # let mut writer = FileWriter::create(file_path)?;
    /// # let rec = RecordBuilder::default().read_id(b"test").build()?;
    /// let read_id = b"test";
    /// writer.write_record(|builder| builder.read_id(read_id).build())?;
    /// # drop(writer)
    /// # assert!(tmp_path.exists());
    /// # let reader = FileReader::open(&file_path)?;
    /// # let rec = reader.get_record(b"test")?;
    /// # Ok(())
    /// # }
    /// ```
    fn write_record<F>(&mut self, build_fn: F) -> Result<(), Slow5Error>
    where
        F: FnOnce(RecordBuilder) -> Result<Record, Slow5Error>,
    {
        let builder = RecordBuilder::default();
        let record = build_fn(builder)?;
        self.add_record(&record)
    }
}

impl Drop for FileWriter {
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

    use crate::FileReader;

    use super::*;
    use anyhow::Result;
    use assert_fs::{fixture::PathChild, TempDir};

    #[test]
    fn test_writer() -> Result<()> {
        let tmp_dir = TempDir::new()?;
        let file_path = "test.slow5";
        let file_path = tmp_dir.child(file_path);
        let mut writer = FileWriter::create(&file_path)?;
        let rec = RecordBuilder::default().read_id(b"test").build()?;
        writer.add_record(&rec)?;
        drop(writer);
        assert!(file_path.exists());

        let reader = FileReader::open(&file_path)?;
        let rec = reader.get_record(b"test")?;
        assert_eq!(rec.read_id(), b"test");
        Ok(())
    }
}
