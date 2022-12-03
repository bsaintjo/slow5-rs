use std::{
    borrow::Borrow,
    ffi::{CStr, CString},
    marker::PhantomData,
    mem::size_of,
    ptr::null_mut,
};

use libc::{c_char, c_void};
use slow5lib_sys::{slow5_aux_set, slow5_file, slow5_rec_free, slow5_rec_t};

use crate::{aux::AuxField, error::Slow5Error, to_cstring, Header};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum BuilderError {
    #[error("Field not set {0}")]
    RequiredValueUnset(&'static str),
    #[error("Conversion error")]
    ConversionError,
    #[error("Failed to allocate memory")]
    AllocationError,
    #[error("Read ID Error, interior nul?")]
    ReadIDError,
}

/// Builder to create a Record, call methods to set parameters and build to
/// convert into a [`Record`].
///
/// # Example
/// ```
/// # use anyhow::Result;
/// # use slow5::RecordBuilder;
/// # fn main() -> Result<()> {
/// let record = RecordBuilder::default()
///     .read_id("test_id")
///     .read_group(0)
///     .digitisation(4096.0)
///     .offset(4.0)
///     .range(12.0)
///     .sampling_rate(4000.0)
///     .raw_signal(&[0, 1, 2, 3])
///     .build()?;
/// # Ok(())
/// # }
/// ```
#[derive(Default)]
pub struct RecordBuilder {
    read_id: Option<Vec<u8>>,
    read_group: Option<u32>,
    digitisation: Option<f64>,
    offset: Option<f64>,
    range: Option<f64>,
    sampling_rate: Option<f64>,
    raw_signal: Option<Vec<i16>>,
}

impl RecordBuilder {
    /// Set the read id of the Record
    pub fn read_id<B: Into<Vec<u8>>>(&mut self, read_id: B) -> &mut Self {
        let read_id = read_id.into();
        self.read_id = Some(read_id);
        self
    }

    /// Set the read group of the Record
    pub fn read_group(&mut self, read_group: u32) -> &mut Self {
        self.read_group = Some(read_group);
        self
    }

    /// Set the digitisation of the Record
    pub fn digitisation(&mut self, digitisation: f64) -> &mut Self {
        self.digitisation = Some(digitisation);
        self
    }

    /// Set the offset of the Record
    pub fn offset(&mut self, offset: f64) -> &mut Self {
        self.offset = Some(offset);
        self
    }

    /// Set the range of the Record
    pub fn range(&mut self, range: f64) -> &mut Self {
        self.range = Some(range);
        self
    }

    /// Set the sampling rate of the Record
    pub fn sampling_rate(&mut self, sampling_rate: f64) -> &mut Self {
        self.sampling_rate = Some(sampling_rate);
        self
    }

    /// Set the signal of the Record using raw values
    pub fn raw_signal(&mut self, raw_signal: &[i16]) -> &mut Self {
        let raw_signal = raw_signal.to_vec();
        self.raw_signal = Some(raw_signal);
        self
    }

    /// Attempt to convert to Record
    ///
    /// # Errors
    /// `RecordBuilder::build` will fail if
    /// A) Unable to allocate memory for Record
    // TODO Should be able to prevent this?
    /// B) Read ID contains an interior NUL character
    /// C) Length of Read ID is greater than u16
    /// D) Length of signal is greater than u64
    pub fn build(&self) -> Result<Record, BuilderError> {
        let Some(ref read_id) = self.read_id else { return Err(BuilderError::RequiredValueUnset("read_id"))};
        let Some(read_group) = self.read_group else { return Err(BuilderError::RequiredValueUnset("read_group"))};
        let Some(digitisation) = self.digitisation else { return Err(BuilderError::RequiredValueUnset("digitisation"))};
        let Some(offset) = self.offset else { return Err(BuilderError::RequiredValueUnset("offset"))};
        let Some(range) = self.range else { return Err(BuilderError::RequiredValueUnset("range"))};
        let Some(sampling_rate) = self.sampling_rate else { return Err(BuilderError::RequiredValueUnset("sampling_rate"))};
        let Some(ref raw_signal) = self.raw_signal else { return Err(BuilderError::RequiredValueUnset("raw_signal"))};


        unsafe {
            let record = libc::calloc(1, size_of::<slow5_rec_t>()) as *mut slow5_rec_t;
            if record.is_null() {
                return Err(BuilderError::AllocationError);
            }

            let read_id_cs = to_cstring(read_id.clone()).map_err(|_| BuilderError::ReadIDError)?;
            let read_id_ptr = read_id_cs.into_raw();
            let read_id_len = read_id.len();
            (*record).read_id = libc::strdup(read_id_ptr as *const c_char);
            (*record).read_id_len = read_id_len.try_into().map_err(|_| BuilderError::ConversionError)?;
            let _ = CString::from_raw(read_id_ptr);

            (*record).read_group = read_group;
            (*record).digitisation = digitisation;
            (*record).offset = offset;
            (*record).range = range;
            (*record).sampling_rate = sampling_rate;

            let len_raw_signal = raw_signal
                .len()
                .try_into()
                .map_err(|_| BuilderError::ConversionError)?;
            (*record).len_raw_signal = len_raw_signal;
            let raw_signal_ptr = allocate(size_of::<i16>() * raw_signal.len())? as *mut i16;

            for (idx, &signal) in raw_signal.iter().enumerate() {
                *raw_signal_ptr.add(idx) = signal;
            }
            (*record).raw_signal = raw_signal_ptr;

            Ok(Record::new(record))
        }
    }
}

// malloc with moving the error checking into a Result enum
unsafe fn allocate(size: usize) -> Result<*mut c_void, BuilderError> {
    let ptr = libc::malloc(size);
    if ptr.is_null() {
        Err(BuilderError::AllocationError)
    } else {
        Ok(ptr)
    }
}

/// Represents a SLOW5 record.
pub struct Record {
    pub(crate) slow5_rec: *mut slow5_rec_t,
}

impl Record {
    pub(crate) fn new(slow5_rec: *mut slow5_rec_t) -> Self {
        Self { slow5_rec }
    }

    /// Initialize builder to make new Record
    pub fn builder() -> RecordBuilder {
        Default::default()
    }

    /// ## Example
    /// ```
    /// # use anyhow::Result;
    /// # use slow5::FileWriter;
    /// # use slow5::FieldType;
    /// # use slow5::RecordBuilder;
    /// # use slow5::WriteOptions;
    /// # use assert_fs::TempDir;
    /// # use assert_fs::fixture::PathChild;
    /// # fn main() -> Result<()> {
    /// # let tmp_dir = TempDir::new().unwrap();
    /// let path = "new.slow5";
    /// # let path = tmp_dir.child(path);
    /// let mut opts = WriteOptions::default();
    /// opts.aux("median", FieldType::Float);
    /// let mut slow5 = opts.create(path)?;
    /// let header = slow5.header();
    /// let mut rec = RecordBuilder::default()
    ///     .read_id("test_id")
    ///     .read_group(0)
    ///     .digitisation(4096.0)
    ///     .offset(4.0)
    ///     .range(12.0)
    ///     .sampling_rate(4000.0)
    ///     .raw_signal(&[0, 1, 2, 3])
    ///     .build()?;
    /// rec.set_aux_field(&header, "median", 10.0f32)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_aux_field<B, T>(
        &mut self,
        hdr: &Header,
        field_name: B,
        value: impl Borrow<T>,
    ) -> Result<(), Slow5Error>
    where
        B: Into<Vec<u8>>,
        T: AuxField,
    {
        let name = to_cstring(field_name)?;
        let value = value.borrow() as *const T as *const c_void;
        let ret = unsafe { slow5_aux_set(self.slow5_rec, name.as_ptr(), value, hdr.header) };
        if ret < 0 {
            Err(Slow5Error::SetAuxFieldError)
        } else {
            Ok(())
        }
    }

    // Expected API
    /// ```
    /// # use anyhow::Result;
    /// # use slow5::FileReader;
    /// # use assert_fs::TempDir;
    /// # use assert_fs::fixture::PathChild;
    /// # fn main() -> Result<()> {
    /// let path = "examples/example2.slow5";
    /// let slow5 = FileReader::open(path)?;
    /// let rec = slow5.get_record("r0")?;
    /// let channel_number: i32 = rec.get_aux_field("read_number")?;
    /// assert_eq!(channel_number, 4019);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_aux_field<B, T>(&self, name: B) -> Result<T, Slow5Error>
    where
        B: Into<Vec<u8>>,
        T: AuxField,
    {
        T::aux_get(self, name)
    }
}

impl Drop for Record {
    fn drop(&mut self) {
        unsafe {
            slow5_rec_free(self.slow5_rec);
        }
    }
}

/// Trait for accessing Record values.
/// 
/// For more info: <https://hasindu2008.github.io/slow5specs/fast5_demystified.pdf>
/// 
/// Method documentation derived from above link.
pub trait RecordExt: RecPtr {
    /// A unique identifier for the read. This is a Universally unique identifier (UUID) version
    /// 4 and should be unique for any read from any device.
    fn read_id(&self) -> &[u8] {
        let str_ptr: *mut c_char = unsafe { (*self.ptr().ptr).read_id };
        let read_id = unsafe { CStr::from_ptr(str_ptr) };

        read_id.to_bytes()
    }

    /// The numnber of quantisation levels in the Analog to Digital Converter. If ADC is 12 bits,
    /// digitisation is ( 2^12 ) = 4096.0.
    fn digitisation(&self) -> f64 {
        unsafe { (*self.ptr().ptr).digitisation }
    }

    /// The ADC offset error. This value is added when converting the signal to pico ampere
    fn offset(&self) -> f64 {
        unsafe { (*self.ptr().ptr).offset }
    }

    /// The full scale measurement range in pico amperes
    fn range(&self) -> f64 {
        unsafe { (*self.ptr().ptr).range }
    }

    /// Group of the read.
    fn read_group(&self) -> u32 {
        unsafe { (*self.ptr().ptr).read_group }
    }

    /// Number of signal measurements
    fn len_signal(&self) -> u64 {
        unsafe { (*self.ptr().ptr).len_raw_signal }
    }

    /// Sampling frequency of the ADC, i.e., the number of data points collected per second
    /// (in Hertz).
    fn sampling_rate(&self) -> f64 {
        unsafe { (*self.ptr().ptr).sampling_rate }
    }

    /// Return iterator over signal in terms of picoamps
    fn picoamps_signal_iter(&self) -> PicoAmpsSignalIter<'_> {
        PicoAmpsSignalIter::new(self.ptr().ptr)
    }

    /// Return iterator over raw signal measurements
    fn raw_signal_iter(&self) -> RawSignalIter<'_> {
        RawSignalIter::new(self.ptr().ptr)
    }
}

impl RecordExt for Record {}

/// Iterator over Records from a SLOW5 file.
///
/// If error occurs, iterator will produce Some(Err(_)) and then subsequent
/// iterations will be None This struct is generated by calling [`records`] on a
/// [`FileReader`].
///
/// [`records`]: crate::FileReader::records
/// [`FileReader`]: crate::FileReader
pub struct RecordIter<'a> {
    slow5_file: *mut slow5_file,
    errored: bool,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> RecordIter<'a> {
    pub(crate) fn new(slow5_file: *mut slow5_file) -> Self {
        Self {
            slow5_file,
            errored: false,
            _lifetime: PhantomData,
        }
    }
}

impl<'a> Iterator for RecordIter<'a> {
    type Item = Result<Record, Slow5Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut rec = null_mut() as *mut slow5_rec_t;
        let ret = unsafe { slow5lib_sys::slow5_get_next(&mut rec, self.slow5_file) };
        if self.errored {
            None
        } else if ret >= 0 {
            Some(Ok(Record::new(rec)))
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

/// Convert raw signal into a picoamps measurement
pub fn to_picoamps(raw_signal: f64, digitisation: f64, offset: f64, range: f64) -> f64 {
    ((raw_signal) + offset) * (range / digitisation)
}

/// Convert picoamps signal into the raw signal
pub fn to_raw_signal(picoamps: f64, digitisation: f64, offset: f64, range: f64) -> f64 {
    (picoamps / (range / digitisation)) - offset
}

/// Iterator over signal in picoamps from Record.
///
/// This struct is generally created by calling [`picoamps_signal_iter`] on a
/// record type.
///
/// [`picoamps_signal_iter`]: RecordExt::picoamps_signal_iter
pub struct PicoAmpsSignalIter<'a> {
    i: u64,
    read: *mut slow5_rec_t,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> PicoAmpsSignalIter<'a> {
    fn new(read: *mut slow5_rec_t) -> Self {
        Self {
            i: 0,
            read,
            _lifetime: PhantomData,
        }
    }
}

impl<'a> Iterator for PicoAmpsSignalIter<'a> {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.i < (*self.read).len_raw_signal {
                let signal = *(*self.read).raw_signal.offset(self.i as isize) as f64;
                let signal = to_picoamps(
                    signal,
                    (*self.read).digitisation,
                    (*self.read).offset,
                    (*self.read).range,
                );
                self.i += 1;
                Some(signal as f64)
            } else {
                None
            }
        }
    }
}

/// Iterator over signal in picoamps from Record.
///
/// This struct is generally created by calling [`raw_signal_iter`] on a
/// record type.
///
/// [`raw_signal_iter`]: RecordExt::raw_signal_iter
pub struct RawSignalIter<'a> {
    i: u64,
    read: *mut slow5_rec_t,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> RawSignalIter<'a> {
    fn new(read: *mut slow5_rec_t) -> Self {
        Self {
            i: 0,
            read,
            _lifetime: PhantomData,
        }
    }
}

impl<'a> Iterator for RawSignalIter<'a> {
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.i < (*self.read).len_raw_signal {
                let signal = *(*self.read).raw_signal.offset(self.i as isize);
                self.i += 1;
                Some(signal)
            } else {
                None
            }
        }
    }
}

pub struct RecordPointer {
    pub(crate) ptr: *mut slow5_rec_t,
}

impl RecordPointer {
    pub(crate) fn new(ptr: *mut slow5_rec_t) -> Self {
        RecordPointer { ptr }
    }
}

// TODO Hide docs, since it isnt useful to have in the documented API
pub trait RecPtr {
    fn ptr(&self) -> RecordPointer;
}

impl RecPtr for Record {
    fn ptr(&self) -> RecordPointer {
        RecordPointer::new(self.slow5_rec)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{aux::FieldType, FileWriter};

    #[test]
    fn test_aux() -> anyhow::Result<()> {
        use assert_fs::{fixture::PathChild, TempDir};
        let tmp_dir = TempDir::new()?;
        let path = "new.slow5";
        let path = tmp_dir.child(path);
        let slow5 = FileWriter::options()
            .aux("median", FieldType::Float)
            .create(path)?;
        let header = slow5.header();
        let mut rec = RecordBuilder::default()
            .read_id("test_id")
            .read_group(0)
            .digitisation(4096.0)
            .offset(4.0)
            .range(12.0)
            .sampling_rate(4000.0)
            .raw_signal(&[0, 1, 2, 3])
            .build()?;
        rec.set_aux_field(&header, "median", 10.0f32)?;
        Ok(())
    }
}
