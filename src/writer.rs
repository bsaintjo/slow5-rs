use std::{os::unix::prelude::OsStrExt, path::Path};

use cstr::cstr;
use slow5lib_sys::{slow5_file, slow5_hdr_write, slow5_open, slow5_set_press, slow5_write};

use crate::{
    compression::Options,
    header::Header,
    record::{Record, RecordBuilder},
    to_cstring, Slow5Error,
};

/// Write a SLOW5 file
pub struct FileWriter {
    slow5_file: *mut slow5_file,
}

impl FileWriter {
    fn new(slow5_file: *mut slow5_file) -> Self {
        Self { slow5_file }
    }

    /// Create a new SLOW5 file, if one already exists, file will be written
    /// over.
    ///
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
        Self::with_options(file_path, Default::default())
    }

    /// Create a file with record and signal compression.
    ///
    /// # Details
    /// If the extension of `file_path` is not blow5 (ie "test.blow5"), the
    /// compression options are ignored.
    ///
    /// # Example
    /// ```
    /// # use assert_fs::TempDir;
    /// # use assert_fs::fixture::PathChild;
    /// # use slow5::FileWriter;
    /// # use slow5::SignalCompression;
    /// # use slow5::RecordCompression;
    /// # use slow5::Options;
    ///
    /// # let tmpdir = TempDir::new().unwrap();
    /// let file_path = "test.blow5";
    /// # let file_path = tmpdir.child(file_path);
    /// let opts = Options::new(RecordCompression::ZStd, SignalCompression::SvbZd);
    /// let writer = FileWriter::with_options(file_path, opts).unwrap();
    /// # writer.close();
    /// ```
    // TODO avoid having to check extension, either by adding it manually
    // or use a lower level API.
    pub fn with_options<P>(file_path: P, opts: Options) -> Result<Self, Slow5Error>
    where
        P: AsRef<Path>,
    {
        let file_path = file_path.as_ref();
        let is_blow5 = {
            if let Some(ext) = file_path.extension() {
                ext == "blow5"
            } else {
                false
            }
        };
        let file_path = file_path.as_os_str().as_bytes();
        let file_path = to_cstring(file_path)?;
        let mode = cstr!("w");

        let slow5_file = unsafe {
            let slow5_file = slow5_open(file_path.as_ptr(), mode.as_ptr());

            if slow5_file.is_null() {
                return Err(Slow5Error::Allocation);
            }

            // Compression
            if is_blow5 {
                let comp_ret = slow5_set_press(
                    slow5_file,
                    opts.rec_comp.to_slow5_rep(),
                    opts.sig_comp.to_slow5_rep(),
                );
                if comp_ret < 0 {
                    return Err(Slow5Error::CompressionError);
                }
            } else {
                log::info!("Not a BLOW5 file, skipping compression");
            }

            // Header
            // TODO Do this at the end?
            let hdr_ret = slow5_hdr_write(slow5_file);
            if hdr_ret == -1 {
                return Err(Slow5Error::HeaderWriteFailed);
            }
            slow5_file
        };

        Ok(Self::new(slow5_file))
    }

    /// Add [`Record`] to SLOW5 file, not thread safe.
    ///
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
    /// let rec = RecordBuilder::builder().read_id(b"test").build()?;
    /// writer.add_record(&rec)?;
    /// # writer.close();
    /// # assert!(file_path.exists());
    /// # let reader = FileReader::open(&file_path)?;
    /// # let rec = reader.get_record(b"test")?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Attempting to add a record with a read ID already in the SLOW5 file will
    /// result in an error.
    pub fn add_record(&mut self, record: &Record) -> Result<(), Slow5Error> {
        let ret = unsafe { slow5_write(record.slow5_rec, self.slow5_file) };
        if ret > 0 {
            Ok(())
        } else {
            Err(Slow5Error::Unknown)
        }
    }

    /// Access header of FileWriter
    pub fn header(&mut self) -> Header {
        let h = unsafe { (*self.slow5_file).header };
        Header::new(h)
    }

    /// Write record to SLOW5 file, with a closure that makes a one-shot
    /// [`RecordBuilder`], an alternative to add_record. Not thread safe.
    ///
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
    /// # let tmp_path = file_path.to_path_buf();
    /// # let mut writer = FileWriter::create(&file_path)?;
    /// # let rec = RecordBuilder::builder().read_id(b"test").build()?;
    /// let read_id = b"test";
    /// writer.write_record(|mut builder| builder.read_id(read_id).build())?;
    /// # writer.close();
    /// # assert!(tmp_path.exists());
    /// # let reader = FileReader::open(&file_path)?;
    /// # let rec = reader.get_record(b"test")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_record<F>(&mut self, build_fn: F) -> Result<(), Slow5Error>
    where
        F: FnOnce(RecordBuilder) -> Result<Record, Slow5Error>,
    {
        let builder = RecordBuilder::default();
        let record = build_fn(builder)?;
        self.add_record(&record)
    }

    /// Close the SLOW5 file.
    pub fn close(self) {
        drop(self)
    }
}

impl Drop for FileWriter {
    fn drop(&mut self) {
        unsafe {
            slow5lib_sys::slow5_close(self.slow5_file);
        }
    }
}

#[cfg(test)]
mod test {

    use anyhow::Result;
    use assert_fs::{fixture::PathChild, TempDir};

    use super::*;
    use crate::{FileReader, RecordExt};

    #[test]
    fn test_writer() -> Result<()> {
        let tmp_dir = TempDir::new()?;
        let file_path = "test.slow5";
        let read_id = b"test";
        let file_path = tmp_dir.child(file_path);
        let mut writer = FileWriter::create(&file_path)?;
        let rec = RecordBuilder::builder()
            .read_id(read_id)
            .raw_signal(&[1, 2, 3])
            .build()?;
        writer.add_record(&rec)?;
        writer.close();
        assert!(file_path.exists());

        let reader = FileReader::open(&file_path)?;
        let rec = reader.get_record(read_id)?;
        assert_eq!(rec.read_id(), read_id);
        Ok(())
    }

    #[test]
    fn test_writer_two() -> Result<()> {
        let tmp_dir = TempDir::new()?;
        let file_path = "test.slow5";
        let read_id = b"test";
        let file_path = tmp_dir.child(file_path);
        let mut writer = FileWriter::create(&file_path)?;
        writer.write_record(|mut builder| builder.read_id(b"r1").raw_signal(&[1, 2, 3]).build())?;
        let rec = RecordBuilder::builder()
            .read_id(read_id)
            .raw_signal(&[1, 2, 3])
            .build()?;
        writer.add_record(&rec)?;
        writer.close();
        assert!(file_path.exists());

        let reader = FileReader::open(&file_path)?;
        let rec = reader.get_record(b"r1")?;
        assert_eq!(rec.read_id(), b"r1");

        let rec = reader.get_record(read_id)?;
        assert_eq!(rec.read_id(), read_id);
        Ok(())
    }

    // #[test]
    // fn test_same_rec() -> Result<()> {
    //     let tmp_dir = TempDir::new()?;
    //     let file_path = "test.slow5";
    //     let read_id = b"test";
    //     let file_path = tmp_dir.child(file_path);
    //     let mut writer = FileWriter::create(&file_path)?;
    //     let rec = RecordBuilder::builder()
    //         .read_id(read_id)
    //         .raw_signal(&[1, 2, 3])
    //         .build()?;
    //     writer.add_record(&rec)?;
    //     let same = writer.add_record(&rec);
    //     assert!(same.is_err());
    //     Ok(())
    // }
}
