use std::{
    ffi::{CStr, CString},
    marker::PhantomData,
    mem::size_of,
};

use libc::{c_char, c_void};
use slow5lib_sys::{slow5_rec_free, slow5_rec_t};

use crate::{
    error::Slow5Error,
    header::{Aux, Header},
    FileReader,
};

/// Builder to create a Record, call methods to set parameters and build to
/// convert into a [`Record`].
///
/// # Example
/// ```
/// # use anyhow::Result;
/// # use slow5::RecordBuilder;
/// # fn main() -> Result<()> {
/// let record = RecordBuilder::builder()
///     .read_id(b"test_id")
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
    read_id: Vec<u8>,
    read_group: u32,
    digitisation: f64,
    offset: f64,
    range: f64,
    sampling_rate: f64,
    raw_signal: Vec<i16>,
}

impl RecordBuilder {
    pub fn builder() -> Self {
        Default::default()
    }
    pub fn read_id(&mut self, read_id: &[u8]) -> &mut Self {
        let read_id = read_id.to_vec();
        self.read_id = read_id;
        self
    }

    pub fn read_group(&mut self, read_group: u32) -> &mut Self {
        self.read_group = read_group;
        self
    }

    pub fn digitisation(&mut self, digitisation: f64) -> &mut Self {
        self.digitisation = digitisation;
        self
    }

    pub fn offset(&mut self, offset: f64) -> &mut Self {
        self.offset = offset;
        self
    }

    pub fn range(&mut self, range: f64) -> &mut Self {
        self.range = range;
        self
    }

    pub fn sampling_rate(&mut self, sampling_rate: f64) -> &mut Self {
        self.sampling_rate = sampling_rate;
        self
    }

    pub fn raw_signal(&mut self, raw_signal: &[i16]) -> &mut Self {
        let raw_signal = raw_signal.to_vec();
        self.raw_signal = raw_signal;
        self
    }

    fn picoamps(&mut self, picoamps: &[f64]) -> &mut Self {
        unimplemented!()
    }

    pub fn build(&mut self) -> Result<Record, Slow5Error> {
        unsafe {
            let record = libc::calloc(1, size_of::<slow5_rec_t>()) as *mut slow5_rec_t;
            if record.is_null() {
                return Err(Slow5Error::Allocation);
            }

            let read_id = CString::new(self.read_id.clone()).map_err(Slow5Error::InteriorNul)?;
            let read_id_ptr = read_id.into_raw();
            let read_id_len = self.read_id.len();
            (*record).read_id = libc::strdup(read_id_ptr as *const c_char);
            (*record).read_id_len = read_id_len.try_into().map_err(|_| Slow5Error::Conversion)?;
            let _ = CString::from_raw(read_id_ptr);

            (*record).read_group = self.read_group;
            (*record).digitisation = self.digitisation;
            (*record).offset = self.offset;
            (*record).range = self.range;
            (*record).sampling_rate = self.sampling_rate;

            let len_raw_signal = self
                .raw_signal
                .len()
                .try_into()
                .map_err(|_| Slow5Error::Conversion)?;
            (*record).len_raw_signal = len_raw_signal;
            let raw_signal_ptr = allocate(size_of::<i16>() * self.raw_signal.len())? as *mut i16;

            for idx in 0..self.raw_signal.len() {
                *raw_signal_ptr.add(idx) = self.raw_signal[idx];
            }
            (*record).raw_signal = raw_signal_ptr;

            Ok(Record::new(true, record))
        }
    }
}

// malloc with moving the error checking into a Result enum
unsafe fn allocate(size: usize) -> Result<*mut c_void, Slow5Error> {
    let ptr = libc::malloc(size);
    if ptr.is_null() {
        Err(Slow5Error::Allocation)
    } else {
        Ok(ptr)
    }
}

/// Owned-type representing a SLOW5 record.
pub struct Record {
    // TODO Figure out whether to keep picoamps
    picoamps: bool,
    pub(crate) slow5_rec: *mut slow5_rec_t,
}

impl Record {
    pub(crate) fn new(picoamps: bool, slow5_rec: *mut slow5_rec_t) -> Self {
        Self {
            picoamps,
            slow5_rec,
        }
    }
    // Expected API
    /// ```ignore
    /// # use anyhow::Result;
    /// # use slow5::FileWriter;
    /// # use assert_fs::TempDir;
    /// # use assert_fs::fixture::PathChild;
    /// # fn main() -> Result<()> {
    /// # let tmp_dir = TempDir::new().unwrap();
    /// let path = "new.slow5";
    /// # let path = tmp_dir.child(path);
    /// let mut slow5 = FileWriter::create(path)?;
    /// let header = slow5.header();
    /// let mut aux: Aux<f64> = header.add_aux_field("median")?;
    /// let rec = RecordBuilder::default().build()?;
    /// rec.add_aux_field(&mut aux, 10.0)?;
    /// # Ok(())
    /// # }
    /// ```
    fn add_aux_field<T>(&mut self, aux: &mut Aux<T>, value: T) -> Result<(), Slow5Error> {
        todo!()
    }

    // Expected API
    /// ```ignore
    /// # use anyhow::Result;
    /// # use slow5::FileWriter;
    /// # use assert_fs::TempDir;
    /// # use assert_fs::fixture::PathChild;
    /// # fn main() -> Result<()> {
    /// let path = "examples/example.slow5";
    /// let slow5 = FileReader::open(path)?;
    /// let rec = slow5.get_record_id("r1");
    /// let header = slow5.header();
    /// let aux: Aux<f64> = header.get_aux_field("median")?;
    /// let value = rec.get_aux_field(aux)?;
    /// # Ok(())
    /// # }
    /// ```
    fn get_aux_field<T>(&mut self, aux: Aux<T>) -> Result<T, Slow5Error> {
        todo!()
    }
}

impl Drop for Record {
    fn drop(&mut self) {
        unsafe {
            slow5_rec_free(self.slow5_rec);
        }
    }
}

/// Immutable view into a single SLOW5 record.
#[derive(Clone)]
pub struct RecordView {
    slow5_rec: *mut slow5_rec_t,
}

impl RecordView {
    fn new(slow5_rec: *mut slow5_rec_t) -> Self {
        Self { slow5_rec }
    }
}

/// Trait for common record methods.
pub trait RecordExt {
    /// Get record's read id.
    fn read_id(&self) -> &[u8];
}

impl RecordExt for RecordView {
    fn read_id(&self) -> &[u8] {
        let str_ptr: *mut c_char = unsafe { (*self.slow5_rec).read_id };
        let read_id = unsafe { CStr::from_ptr(str_ptr) };

        read_id.to_bytes()
    }
}

impl RecordExt for Record {
    fn read_id(&self) -> &[u8] {
        let str_ptr: *mut c_char = unsafe { (*self.slow5_rec).read_id };
        let read_id = unsafe { CStr::from_ptr(str_ptr) };

        read_id.to_bytes()
    }
}

/// Iterator over Records from a SLOW5 file.
///
/// If error occurs, iterator will produce Some(Err(_)) and then subsequent
/// iterations will be None This struct is generated by calling [`records`] on a
/// [`FileReader`].
///
/// [`records`]: FileReader::records
pub struct RecordIter {
    slow5_rec_ptr: *mut slow5_rec_t,
    slow5_file: FileReader,
    errored: bool,
}

impl RecordIter {
    pub(crate) fn new(slow5_rec_ptr: *mut slow5_rec_t, slow5_file: FileReader) -> Self {
        Self {
            slow5_rec_ptr,
            slow5_file,
            errored: false,
        }
    }
}

impl Iterator for RecordIter {
    type Item = Result<RecordView, Slow5Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = unsafe {
            slow5lib_sys::slow5_get_next(&mut self.slow5_rec_ptr, self.slow5_file.slow5_file)
        };
        if self.errored {
            None
        } else if ret >= 0 {
            Some(Ok(RecordView::new(self.slow5_rec_ptr)))
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

impl Drop for RecordIter {
    fn drop(&mut self) {
        unsafe { slow5_rec_free(self.slow5_rec_ptr) }
    }
}

fn to_picoamps(raw_val: f64, digitisation: f64, offset: f64, range: f64) -> f64 {
    ((raw_val) + offset) * (range / digitisation)
}

/// Iterator over signal in picoamps from Record.
///
/// This struct is generally created by calling [`signal_iter`] on a record
/// type.
///
/// [`signal_iter`]: SignalIterExt::signal_iter
pub struct SignalIter<'a> {
    picoamps: bool,
    i: u64,
    read: *mut slow5_rec_t,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> SignalIter<'a> {
    fn new(picoamps: bool, read: *mut slow5_rec_t) -> Self {
        Self {
            picoamps,
            i: 0,
            read,
            _lifetime: PhantomData,
        }
    }
}

impl<'a> Iterator for SignalIter<'a> {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.i < (*self.read).len_raw_signal {
                let mut signal = *(*self.read).raw_signal.offset(self.i as isize) as f64;
                if self.picoamps {
                    signal = to_picoamps(
                        signal,
                        (*self.read).digitisation,
                        (*self.read).offset,
                        (*self.read).range,
                    );
                }
                self.i += 1;
                Some(signal as f64)
            } else {
                None
            }
        }
    }
}

/// Extension trait to get a SignalIter from record-like types
// TODO maybe combine this with RecordExt
pub trait SignalIterExt {
    fn signal_iter(&self) -> SignalIter<'_>;
}

impl SignalIterExt for Record {
    fn signal_iter(&self) -> SignalIter<'_> {
        SignalIter::new(self.picoamps, self.slow5_rec)
    }
}

impl SignalIterExt for RecordView {
    fn signal_iter(&self) -> SignalIter<'_> {
        SignalIter::new(true, self.slow5_rec)
    }
}

trait RecPtr {
    fn ptr(&self) -> &*mut slow5_rec_t;
}

impl RecPtr for Record {
    fn ptr(&self) -> &*mut slow5_rec_t {
        &self.slow5_rec
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::FileWriter;

    #[ignore = "Brainstorming api"]
    #[test]
    fn test_aux() -> anyhow::Result<()> {
        // use anyhow::Result;
        use assert_fs::{fixture::PathChild, TempDir};
        // use slow5::FileWriter;
        // fn main() -> Result<()> {
        let tmp_dir = TempDir::new()?;
        let path = "new.slow5";
        let path = tmp_dir.child(path);
        let mut slow5 = FileWriter::create(path)?;
        let mut header = slow5.header();
        let mut aux: Aux<f64> = header.add_aux_field("median")?;
        let mut rec = RecordBuilder::default().build()?;
        rec.add_aux_field(&mut aux, 10.0)?;
        Ok(())
        // }
    }
}
