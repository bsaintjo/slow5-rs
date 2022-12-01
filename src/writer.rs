use std::{collections::HashMap, os::unix::prelude::OsStrExt, path::Path};

use cstr::cstr;
use slow5lib_sys::{slow5_file, slow5_hdr_write, slow5_open, slow5_set_press, slow5_write};

use crate::{
    header::Header,
    record::{Record, RecordBuilder},
    to_cstring, FieldType, RecordCompression, SignalCompression, Slow5Error,
};

/// Set attributes, auxiliary fields, and record and signal compression.
pub struct WriteOptions {
    pub(crate) rec_comp: RecordCompression,
    pub(crate) sig_comp: SignalCompression,
    attributes: HashMap<Vec<u8>, (Vec<u8>, u32)>,
    auxiliary_fields: HashMap<Vec<u8>, FieldType>,
}

impl WriteOptions {
    fn new(
        rec_comp: RecordCompression,
        sig_comp: SignalCompression,
        attributes: HashMap<Vec<u8>, (Vec<u8>, u32)>,
        auxiliary_fields: HashMap<Vec<u8>, FieldType>,
    ) -> Self {
        Self {
            rec_comp,
            sig_comp,
            attributes,
            auxiliary_fields,
        }
    }

    /// Set attribute for header.
    ///
    /// # Note
    /// If the key and read_group are the same, the value for it will be overwritten.
    ///
    /// # Example
    /// ```
    /// # use slow5::WriteOptions;
    /// let mut opts = WriteOptions::default();
    /// opts.attr("asic_id", "123456", 0);
    /// opts.attr("asic_id", "7891011", 1);
    ///
    /// opts.attr("device_type", "cool", 0);
    /// // Above line is ignored but this is the same read group
    /// opts.attr("device_type", "wow", 0);
    /// ```
    pub fn attr<K, V>(&mut self, key: K, value: V, read_group: u32) -> &mut Self
    where
        K: Into<Vec<u8>>,
        V: Into<Vec<u8>>,
    {
        let key = key.into();
        let value = value.into();
        self.attributes.insert(key, (value, read_group));
        self
    }

    /// Set auxiliary field for header. See [`FieldType`] for info on
    /// types allowed as fields.
    ///
    /// # Note
    /// If the same name is used multiple times, the last FieldType will be used in the header.
    ///
    /// # Example
    /// ```
    /// # use slow5::WriteOptions;
    /// use slow5::FieldType;
    /// let mut opts = WriteOptions::default();
    /// opts.aux("median", FieldType::Double);
    /// opts.aux("read_number", FieldType::Uint8);
    /// ```
    pub fn aux<B>(&mut self, name: B, field_ty: FieldType) -> &mut Self
    where
        B: Into<Vec<u8>>,
    {
        let name = name.into();
        self.auxiliary_fields.insert(name, field_ty);
        self
    }

    /// Set compression of the SLOW5 records. By default no compression is used.
    ///
    /// # Example
    /// ```
    /// # use slow5::WriteOptions;
    /// use slow5::RecordCompression;
    /// let mut opts = WriteOptions::default();
    /// opts.record_compression(RecordCompression::Zlib);
    /// ```
    pub fn record_compression(&mut self, rcomp: RecordCompression) -> &mut Self {
        self.rec_comp = rcomp;
        self
    }

    /// Set compression of the SLOW5 signal data. By default no compression is used.
    ///
    /// # Example
    /// ```
    /// # use slow5::WriteOptions;
    /// use slow5::SignalCompression;
    /// let mut opts = WriteOptions::default();
    /// opts.signal_compression(SignalCompression::StreamVByte);
    pub fn signal_compression(&mut self, scomp: SignalCompression) -> &mut Self {
        self.sig_comp = scomp;
        self
    }
}

impl Default for WriteOptions {
    fn default() -> Self {
        WriteOptions::new(
            RecordCompression::None,
            SignalCompression::None,
            Default::default(),
            Default::default(),
        )
    }
}

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
    /// use slow5::{RecordCompression::ZStd, SignalCompression::SvbZd};
    /// # use slow5::WriteOptions;
    ///
    /// # let tmpdir = TempDir::new().unwrap();
    /// let file_path = "test.blow5";
    /// # let file_path = tmpdir.child(file_path);
    /// let mut opts = WriteOptions::default();
    /// opts.record_compression(ZStd).signal_compression(SvbZd);
    /// let writer = FileWriter::with_options(file_path, opts).unwrap();
    /// # writer.close();
    /// ```
    // TODO avoid having to check extension, either by adding it manually
    // or use a lower level API.
    pub fn with_options<P>(file_path: P, opts: WriteOptions) -> Result<Self, Slow5Error>
    where
        P: AsRef<Path>,
    {
        // If we aren't testing or running in debug mode, silence slow5lib logs
        #[cfg(any(not(test), not(debug_assertions)))]
        unsafe {
            slow5lib_sys::slow5_set_log_level(slow5lib_sys::slow5_log_level_opt_SLOW5_LOG_OFF);
        }

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

            // Initialize all attributes and auxiliary fields
            let mut header = Header::new((*slow5_file).header);
            for (name, (value, rg)) in opts.attributes.into_iter() {
                header.add_attribute(name.clone())?;
                header.set_attribute(name, value, rg)?;
            }

            for (name, fty) in opts.auxiliary_fields.into_iter() {
                header.add_aux_field(name, fty)?;
            }

            // Header
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
    /// let rec = RecordBuilder::builder().read_id("test").build()?;
    /// writer.add_record(&rec)?;
    /// # writer.close();
    /// # assert!(file_path.exists());
    /// # let reader = FileReader::open(&file_path)?;
    /// # let rec = reader.get_record("test")?;
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
    /// # Example
    /// ```
    /// # use slow5::FileWriter;
    /// # use assert_fs::TempDir;
    /// # use assert_fs::fixture::PathChild;
    /// # use slow5::WriteOptions;
    /// # let tmp_dir = TempDir::new().unwrap();
    /// let file_path = "test.slow5";
    /// # let file_path = tmp_dir.child(file_path);
    /// let mut opts = WriteOptions::default();
    /// opts.attr("asic_id", "test", 0);
    /// let writer = FileWriter::with_options(&file_path, opts).unwrap();
    /// let header = writer.header();
    /// assert_eq!(header.get_attribute("asic_id", 0).unwrap(), b"test");
    /// ```
    pub fn header(&self) -> Header {
        let h = unsafe { (*self.slow5_file).header };
        Header::new(h)
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
        let read_id: &[u8] = b"test";
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
}
