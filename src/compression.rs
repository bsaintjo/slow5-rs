use slow5lib_sys::{
    slow5_press_method_SLOW5_COMPRESS_NONE, slow5_press_method_SLOW5_COMPRESS_SVB_ZD,
    slow5_press_method_SLOW5_COMPRESS_ZLIB, slow5_press_method_SLOW5_COMPRESS_ZSTD,
};

/// SLOW5 record compression
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

/// SLOW5 signal compression
pub enum SignalCompression {
    /// No signal compression
    None,
    /// Compress using streamvbyte library
    StreamVByte,
}

impl SignalCompression {
    pub(crate) fn to_slow5_rep(&self) -> u32 {
        match self {
            SignalCompression::None => slow5_press_method_SLOW5_COMPRESS_NONE,
            SignalCompression::StreamVByte => slow5_press_method_SLOW5_COMPRESS_SVB_ZD,
        }
    }
}
