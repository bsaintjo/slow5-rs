use std::{
    collections::{HashMap, HashSet},
    fmt,
    os::unix::prelude::OsStrExt,
    path::Path,
};

use cstr::cstr;
use slow5lib_sys::{
    slow5_file, slow5_hdr_add_rg, slow5_hdr_write, slow5_open, slow5_set_press, slow5_write,
};

use crate::{
    header::Header, record::Record, to_cstring, FieldType, RecordCompression, SignalCompression,
    Slow5Error,
};

/// Set attributes, auxiliary fields, and record and signal compression.
#[derive(Debug)]
pub struct WriteOptions {
    pub(crate) rec_comp: RecordCompression,
    pub(crate) sig_comp: SignalCompression,
    pub(crate) num_read_groups: u32,
    attributes: HashMap<(Vec<u8>, u32), Vec<u8>>,
    auxiliary_fields: HashMap<Vec<u8>, FieldType>,
}

impl WriteOptions {
    fn new(
        rec_comp: RecordCompression,
        sig_comp: SignalCompression,
        num_read_groups: u32,
        attributes: HashMap<(Vec<u8>, u32), Vec<u8>>,
        auxiliary_fields: HashMap<Vec<u8>, FieldType>,
    ) -> Self {
        Self {
            rec_comp,
            sig_comp,
            num_read_groups,
            attributes,
            auxiliary_fields,
        }
    }

    /// Set attribute for header.
    ///
    /// # Note
    /// If the key and read_group are the same, the value for it will be
    /// overwritten.
    ///
    /// # Example
    /// ```
    /// # use slow5::WriteOptions;
    /// # use assert_fs::TempDir;
    /// # use assert_fs::fixture::PathChild;
    /// # let tmp_dir = TempDir::new().unwrap();
    /// let mut opts = WriteOptions::default();
    /// let file_path = "test.slow5";
    /// # let file_path = tmp_dir.child(file_path);
    /// opts.attr("asic_id", "123456", 0);
    /// opts.attr("asic_id", "7891011", 1);
    ///
    /// opts.attr("device_type", "cool", 0);
    /// // Above line is ignored because it has the same read group and attributre
    /// opts.attr("device_type", "wow", 0);
    /// let slow5 = opts.create(file_path).unwrap();
    /// let header = slow5.header();
    /// assert_eq!(header.get_attribute("device_type", 0).unwrap(), b"wow");
    /// ```
    pub fn attr<K, V>(&mut self, key: K, value: V, read_group: u32) -> &mut Self
    where
        K: Into<Vec<u8>>,
        V: Into<Vec<u8>>,
    {
        let key = (key.into(), read_group);
        let value = value.into();
        self.attributes.insert(key, value);
        if read_group > self.num_read_groups {
            self.num_read_groups = read_group;
        }
        self
    }

    /// Set auxiliary field for header. See [`FieldType`] for info on
    /// types allowed as fields.
    ///
    /// # Note
    /// If the same name is used multiple times, the last FieldType will be used
    /// in the header.
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

    /// Set compression of the SLOW5 signal data. By default no compression is
    /// used.
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

    /// Explicitly set the number of read groups. See [`attr`] for more
    /// information.
    ///
    /// # Notes
    /// Returns Err if n is lower than inferred number of read groups
    /// ```
    /// # use slow5::WriteOptions;
    /// let mut opts = WriteOptions::default();
    /// opts.attr("test", "val", 0).attr("test", "bigger", 10);
    /// assert!(opts.num_read_groups(2).is_err());
    /// ```
    ///
    /// [`attr`]: crate::WriteOptions::attr
    pub fn num_read_groups(&mut self, n: u32) -> Result<&mut Self, Slow5Error> {
        if n < self.num_read_groups {
            Err(Slow5Error::NumReadGroups(n, self.num_read_groups))
        } else {
            self.num_read_groups = n;
            Ok(self)
        }
    }

    /// Create new SLOW5 at file path with Options
    /// # Example
    /// ```
    /// # use slow5::WriteOptions;
    /// # use assert_fs::TempDir;
    /// # use assert_fs::fixture::PathChild;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let tmp_dir = TempDir::new()?;
    /// let file_path = "new.slow5";
    /// # let file_path = tmp_dir.child(file_path);
    /// let slow5 = WriteOptions::default().create(file_path)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn create<P: AsRef<Path>>(&self, file_path: P) -> Result<FileWriter, Slow5Error> {
        FileWriter::with_options(file_path, self)
    }
}

impl Default for WriteOptions {
    fn default() -> Self {
        WriteOptions::new(
            RecordCompression::None,
            SignalCompression::None,
            0,
            Default::default(),
            Default::default(),
        )
    }
}

#[derive(Debug)]
struct Version {
    major: u8,
    minor: u8,
    patch: u8,
}

/// Write a SLOW5 file
pub struct FileWriter {
    slow5_file: *mut slow5_file,
}

impl fmt::Debug for FileWriter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FileWriter")
            .field("version", &self.version())
            .finish()
    }
}

impl FileWriter {
    fn new(slow5_file: *mut slow5_file) -> Self {
        Self { slow5_file }
    }

    fn version(&self) -> Version {
        let version = unsafe { &(*(*self.slow5_file).header).version };
        let major = version.major;
        let minor = version.minor;
        let patch = version.patch;
        Version {
            major,
            minor,
            patch,
        }
    }

    /// Create a file with set of options
    pub fn options() -> WriteOptions {
        WriteOptions::default()
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
        Self::with_options(file_path, &Default::default())
    }

    /// Create a file with given options
    ///
    /// # Details
    /// If the extension of `file_path` is not blow5 (ie "test.blow5"), the
    /// compression options are ignored.
    // TODO avoid having to check extension, either by adding it manually
    // or use a lower level API.
    pub(crate) fn with_options<P>(file_path: P, opts: &WriteOptions) -> Result<Self, Slow5Error>
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

        // Check if compression is being used on a SLOW5, if so error out
        let has_rec_comp = !matches!(opts.rec_comp, RecordCompression::None);
        let has_sig_comp = !matches!(opts.sig_comp, SignalCompression::None);
        if !is_blow5 && (has_rec_comp || has_sig_comp) {
            return Err(Slow5Error::Slow5CompressionError);
        }

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

            // Add read groups
            let header_ptr = (*slow5_file).header;
            for rg in 0..opts.num_read_groups {
                let ret = slow5_hdr_add_rg(header_ptr);
                if ret < 0 {
                    return Err(Slow5Error::FailedAddReadGroup(rg));
                }
            }
            // (*header_ptr).num_read_groups = opts.num_read_groups + 1;

            // Initialize all attributes and auxiliary fields
            let mut header = Header::new(header_ptr);
            let mut added_attr: HashSet<Vec<u8>> = HashSet::new();
            for ((name, rg), value) in opts.attributes.iter() {
                if !added_attr.contains(name) {
                    added_attr.insert(name.clone());
                    header.add_attribute(name.clone())?;
                }
                header.set_attribute(name.clone(), value.clone(), *rg)?;
            }

            for (name, fty) in opts.auxiliary_fields.iter() {
                header.add_aux_field(name.clone(), *fty)?;
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
    /// # use slow5::Record;
    /// # fn main() -> Result<()> {
    /// # let tmp_dir = TempDir::new()?;
    /// # let file_path = "test.slow5";
    /// # let file_path = tmp_dir.child(file_path);
    /// # let mut writer = FileWriter::create(&file_path)?;
    /// let rec = Record::builder()
    ///     .read_id("test")
    ///     .read_group(0)
    ///     .digitisation(4096.0)
    ///     .offset(4.0)
    ///     .range(12.0)
    ///     .sampling_rate(4000.0)
    ///     .raw_signal(&[0, 1, 2, 3])
    ///     .build()?;
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
    /// let writer = opts.create(&file_path).unwrap();
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
        let rec = Record::builder()
            .read_id(read_id)
            .read_group(0)
            .digitisation(4096.0)
            .offset(4.0)
            .range(12.0)
            .sampling_rate(4000.0)
            .raw_signal(&[0, 1, 2, 3])
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
    fn test_err_slow5_compression() -> anyhow::Result<()> {
        let tmp_dir = TempDir::new()?;
        let file_path = "test.slow5";
        let file_path = tmp_dir.child(file_path);
        let writer = FileWriter::options()
            .signal_compression(SignalCompression::StreamVByte)
            .create(&file_path);
        assert!(writer.is_err());

        let writer = FileWriter::options()
            .record_compression(RecordCompression::ZStd)
            .create(&file_path);
        assert!(writer.is_err());

        let writer = FileWriter::options()
            .signal_compression(SignalCompression::StreamVByte)
            .record_compression(RecordCompression::Zlib)
            .create(&file_path);
        assert!(writer.is_err());
        Ok(())
    }
}
