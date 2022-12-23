use std::{
    collections::{HashMap, HashSet},
    ffi::{CStr, CString},
    fmt,
    os::unix::prelude::OsStrExt,
    path::Path,
};

use cstr::cstr;
use slow5lib_sys::{
    slow5_file, slow5_hdr_add_rg, slow5_hdr_write, slow5_open, slow5_set_press, slow5_write,
};

use crate::{
    header::{Header, HeaderExt},
    record::Record,
    to_cstring, FieldType, RecordCompression, SignalCompression, Slow5Error,
};

#[derive(Debug)]
enum FileType {
    Slow5,
    Blow5,
}

// Check the file extension, return Err if it isn't blow5 or slow5
fn check_file_ext<P>(file_path: P) -> Result<FileType, Slow5Error>
where
    P: AsRef<Path>,
{
    let file_path = file_path.as_ref();
    let Some(ext) = file_path.extension() else { return Err(Slow5Error::InvalidFilePath(String::from("No file extension found")) )};
    if ext == "blow5" {
        Ok(FileType::Blow5)
    } else if ext == "slow5" {
        Ok(FileType::Slow5)
    } else {
        Err(Slow5Error::InvalidFilePath(String::from("found ")))
    }
}

#[derive(Debug)]
pub(crate) enum Mode {
    Write,
    Append,
}

impl Mode {
    fn to_c_mode(&self) -> &CStr {
        match self {
            Mode::Write => cstr!("w"),
            Mode::Append => cstr!("a"),
        }
    }
}

/// Set attributes, auxiliary fields, and record and signal compression.
#[derive(Debug)]
pub struct WriteOptions {
    pub(crate) rec_comp: RecordCompression,
    pub(crate) sig_comp: SignalCompression,
    pub(crate) num_read_groups: u32,
    attributes: HashMap<(Vec<u8>, u32), Vec<u8>>,
    auxiliary_fields: HashMap<Vec<u8>, FieldType>,
    aux_enums: HashMap<Vec<u8>, Vec<Vec<u8>>>,
}

impl WriteOptions {
    fn new(
        rec_comp: RecordCompression,
        sig_comp: SignalCompression,
        num_read_groups: u32,
        attributes: HashMap<(Vec<u8>, u32), Vec<u8>>,
        auxiliary_fields: HashMap<Vec<u8>, FieldType>,
        aux_enums: HashMap<Vec<u8>, Vec<Vec<u8>>>,
    ) -> Self {
        Self {
            rec_comp,
            sig_comp,
            num_read_groups,
            attributes,
            auxiliary_fields,
            aux_enums,
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
    /// # fn main() -> anyhow::Result<()> {
    /// # let tmp_dir = TempDir::new()?;
    /// let mut opts = WriteOptions::default();
    /// let file_path = "test.slow5";
    /// # let file_path = tmp_dir.child(file_path);
    /// opts.attr("asic_id", "123456", 0);
    /// opts.attr("asic_id", "7891011", 1);
    ///
    /// opts.attr("device_type", "cool", 0);
    /// // Above line is ignored because it has the same read group and attributre
    /// opts.attr("device_type", "wow", 0);
    /// let slow5 = opts.create(file_path)?;
    /// let header = slow5.header();
    /// assert_eq!(header.get_attribute("device_type", 0)?, b"wow");
    /// # Ok(())
    /// # }
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

    /// Add auxiliary enum to writer with designated labels.
    pub fn aux_enum<B, C>(&mut self, name: B, labels: Vec<C>) -> &mut Self
    where
        B: Into<Vec<u8>>,
        C: Into<Vec<u8>>,
    {
        let name = name.into();
        let labels = labels.into_iter().map(|l| l.into()).collect();
        self.aux_enums.insert(name, labels);
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

    /// Create new file with the given options. File type will be SLOW5 or BLOW5
    /// based on the file extension. # Example
    /// ```
    /// # use slow5::WriteOptions;
    /// # use assert_fs::TempDir;
    /// # use assert_fs::fixture::PathChild;
    /// # fn main() -> anyhow::Result<()> {
    /// # let tmp_dir = TempDir::new()?;
    /// let file_path = "new.slow5";
    /// # let file_path = tmp_dir.child(file_path);
    /// let slow5 = WriteOptions::default().create(file_path)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    /// If you attempt to create a SLOW5 file with compression options, this
    /// function will return an Err. Since SLOW5 is ascii, no compression is
    /// allowed. If you do want compression create a BLOW5 file.
    ///
    /// File path must end in ".blow5" or ".slow5" otherwise, function will
    /// return an Err.
    pub fn create<P: AsRef<Path>>(&self, file_path: P) -> Result<FileWriter, Slow5Error> {
        FileWriter::with_options(file_path, self, Mode::Write)
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
            Default::default(),
        )
    }
}

/// Write a SLOW5 file
pub struct FileWriter {
    slow5_file: *mut slow5_file,

    // This stores CStrings used in slow5_aux_set and extends the lifetime of the CString until it
    // gets dropped. slow5_aux_get doesn't allocate so we must manually extend the lifetime.
    // TODO Replace using this with getting a pointer to the auxiliary field already allocated in
    // the header
    // TODO Replace with HashSet?
    pub(crate) auxiliary_fields: Vec<CString>,
}

impl fmt::Debug for FileWriter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let version = unsafe { &(*(*self.slow5_file).header).version };
        let version = (version.major, version.minor, version.patch);
        f.debug_struct("FileWriter")
            .field("version", &version)
            .field("record compression", &self.record_compression())
            .field("signal compression", &self.signal_compression())
            .finish()
    }
}

impl FileWriter {
    fn new(slow5_file: *mut slow5_file) -> Self {
        Self {
            slow5_file,
            auxiliary_fields: Vec::new(),
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
    /// use slow5::FileWriter;
    /// # use slow5::Slow5Error;
    /// # use assert_fs::TempDir;
    /// # use assert_fs::fixture::PathChild;
    /// # fn main() -> anyhow::Result<()> {
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
        Self::with_options(file_path, &Default::default(), Mode::Write)
    }

    /// Append to a previously created file.
    ///
    /// # Example
    /// ```
    /// # use slow5::FileWriter;
    /// # use assert_fs::TempDir;
    /// # use assert_fs::fixture::PathChild;
    /// # fn main() -> anyhow::Result<()> {
    /// # let tmp_dir = TempDir::new()?;
    /// let file_path = "examples/example3.blow5";
    /// # let tmp_file = tmp_dir.child("example3.blow5");
    /// # let _ = std::fs::copy(&file_path, &tmp_file)?;
    /// # let file_path = tmp_file;
    /// let writer = FileWriter::append(&file_path)?;
    /// # writer.close();
    /// # Ok(())
    /// # }
    /// ```
    pub fn append<P>(file_path: P) -> Result<Self, Slow5Error>
    where
        P: AsRef<Path>,
    {
        Self::with_options(file_path, &Default::default(), Mode::Append)
    }

    /// Create a file with given options
    ///
    /// # Details
    /// If the extension of `file_path` is not blow5 (ie "test.blow5"), the
    /// compression options are ignored.
    // TODO avoid having to check extension, either by adding it manually
    // or use a lower level API.
    pub(crate) fn with_options<P>(
        file_path: P,
        opts: &WriteOptions,
        mode: Mode,
    ) -> Result<Self, Slow5Error>
    where
        P: AsRef<Path>,
    {
        // If we aren't testing or running in debug mode, silence slow5lib logs
        #[cfg(any(not(test), not(debug_assertions)))]
        unsafe {
            slow5lib_sys::slow5_set_log_level(slow5lib_sys::slow5_log_level_opt_SLOW5_LOG_OFF);
        }

        let file_ext = check_file_ext(&file_path)?;

        // Check if compression is being used on a SLOW5, if so error out
        let has_rec_comp = !matches!(opts.rec_comp, RecordCompression::None);
        let has_sig_comp = !matches!(opts.sig_comp, SignalCompression::None);
        if matches!(file_ext, FileType::Slow5) && (has_rec_comp || has_sig_comp) {
            return Err(Slow5Error::Slow5CompressionError);
        }

        let file_path = file_path.as_ref().as_os_str().as_bytes();
        let file_path = to_cstring(file_path)?;
        let mode_str = mode.to_c_mode();

        let slow5_file = unsafe { slow5_open(file_path.as_ptr(), mode_str.as_ptr()) };
        if matches!(mode, Mode::Append) {
            return Ok(Self::new(slow5_file));
        }

        if slow5_file.is_null() {
            return Err(Slow5Error::Allocation);
        }

        unsafe {
            if matches!(file_ext, FileType::Blow5) {
                // Compression
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

            // Auxiliary fields
            for (name, fty) in opts.auxiliary_fields.iter() {
                header.add_aux_field(name.clone(), *fty)?;
            }

            // Auxiliary enum fields
            for (name, labels) in opts.aux_enums.iter() {
                header.add_aux_enum_field(name.clone(), labels.clone())?;
            }

            // Header
            let hdr_ret = slow5_hdr_write(slow5_file);
            if hdr_ret == -1 {
                return Err(Slow5Error::HeaderWriteFailed);
            }
        }

        Ok(Self::new(slow5_file))
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

    /// Add [`Record`] to SLOW5 file, not thread safe.
    ///
    /// # Example
    /// ```
    /// # use slow5::FileWriter;
    /// # use slow5::FileReader;
    /// # use slow5::Slow5Error;
    /// # use assert_fs::TempDir;
    /// # use assert_fs::fixture::PathChild;
    /// # use slow5::Record;
    /// # fn main() -> anyhow::Result<()> {
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
    /// # fn main() -> anyhow::Result<()> {
    /// # let tmp_dir = TempDir::new()?;
    /// let file_path = "test.slow5";
    /// # let file_path = tmp_dir.child(file_path);
    /// let mut opts = WriteOptions::default();
    /// opts.attr("asic_id", "test", 0);
    /// let writer = opts.create(&file_path)?;
    /// let header = writer.header();
    /// assert_eq!(header.get_attribute("asic_id", 0)?, b"test");
    /// # Ok(())
    /// # }
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

impl HeaderExt for FileWriter {
    fn header(&self) -> Header<'_> {
        self.header()
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

    #[test]
    fn test_append() {
        let tmp_dir = TempDir::new().unwrap();
        let file_path = tmp_dir.child("test.blow5");
        let writer = FileWriter::options()
            .signal_compression(SignalCompression::StreamVByte)
            .create(&file_path)
            .unwrap();
        writer.close();
        let appender = FileWriter::append(&file_path).unwrap();
        appender.close();
    }

    #[test]
    fn test_extension() {
        let tmp_dir = TempDir::new().unwrap();
        let file_path = tmp_dir.child("test.blow");
        let writer = FileWriter::create(file_path);
        assert!(writer.is_err());
    }

    #[test]
    fn test_compression_getter() -> anyhow::Result<()> {
        let tmp_dir = TempDir::new().unwrap();
        let file_path = tmp_dir.child("test.blow5");
        let record_press = RecordCompression::ZStd;
        let signal_press = SignalCompression::StreamVByte;
        let writer = FileWriter::options()
            .record_compression(record_press)
            .signal_compression(signal_press)
            .create(file_path)?;
        assert_eq!(record_press, writer.record_compression());
        Ok(())
    }
}
