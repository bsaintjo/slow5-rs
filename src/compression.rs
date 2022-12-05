use slow5lib_sys::{
    slow5_press_method_SLOW5_COMPRESS_NONE, slow5_press_method_SLOW5_COMPRESS_SVB_ZD,
    slow5_press_method_SLOW5_COMPRESS_ZLIB, slow5_press_method_SLOW5_COMPRESS_ZSTD,
};

/// SLOW5 record compression
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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

impl From<u32> for RecordCompression {
    #[allow(non_upper_case_globals)]
    fn from(n: u32) -> Self {
        match n {
            slow5_press_method_SLOW5_COMPRESS_NONE => Self::None,
            slow5_press_method_SLOW5_COMPRESS_ZLIB => Self::Zlib,
            slow5_press_method_SLOW5_COMPRESS_ZSTD => Self::ZStd,
            _ => unreachable!("Invalid record compression"),
        }
    }
}

/// SLOW5 signal compression
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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

impl From<u32> for SignalCompression {
    #[allow(non_upper_case_globals)]
    fn from(n: u32) -> Self {
        match n {
            slow5_press_method_SLOW5_COMPRESS_NONE => SignalCompression::None,
            slow5_press_method_SLOW5_COMPRESS_SVB_ZD => SignalCompression::StreamVByte,
            _ => unreachable!("Invalid compression"),
        }
    }
}
