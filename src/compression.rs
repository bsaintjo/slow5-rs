use slow5lib_sys::{
    slow5_press_method_SLOW5_COMPRESS_NONE, slow5_press_method_SLOW5_COMPRESS_SVB_ZD,
    slow5_press_method_SLOW5_COMPRESS_ZLIB, slow5_press_method_SLOW5_COMPRESS_ZSTD,
};

/// How to compress the SLOW5 records
pub enum RecordCompression {
    /// No compression
    None,
    /// Compress using zlib
    Zlib,
    /// Compress using zstd
    ZStd,
}

impl RecordCompression {
    pub(crate) fn to_slow5_rep(&self) -> u32 {
        match self {
            Self::None => slow5_press_method_SLOW5_COMPRESS_NONE,
            Self::ZStd => slow5_press_method_SLOW5_COMPRESS_ZSTD,
            Self::Zlib => slow5_press_method_SLOW5_COMPRESS_ZLIB,
        }
    }
}

/// How to compress the signal data
pub enum SignalCompression {
    /// No signal compression
    None,
    /// Compress using SVB-ZD algorithm
    SvbZd,
}

impl SignalCompression {
    pub(crate) fn to_slow5_rep(&self) -> u32 {
        match self {
            SignalCompression::None => slow5_press_method_SLOW5_COMPRESS_NONE,
            SignalCompression::SvbZd => slow5_press_method_SLOW5_COMPRESS_SVB_ZD,
        }
    }
}

/// Set record and signal compression
pub struct Options {
    pub(crate) rec_comp: RecordCompression,
    pub(crate) sig_comp: SignalCompression,
}

impl Options {
    pub fn new(rec_comp: RecordCompression, sig_comp: SignalCompression) -> Self {
        Options { rec_comp, sig_comp }
    }
}

impl Default for Options {
    fn default() -> Self {
        Options::new(RecordCompression::None, SignalCompression::None)
    }
}
