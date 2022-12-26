#[cfg(feature = "zstd")]
use slow5lib_sys::slow5_press_method_SLOW5_COMPRESS_ZSTD;
use slow5lib_sys::{
    slow5_press_method_SLOW5_COMPRESS_NONE, slow5_press_method_SLOW5_COMPRESS_SVB_ZD,
    slow5_press_method_SLOW5_COMPRESS_ZLIB,
};

/// SLOW5 record compression
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum RecordCompression {
    /// No compression
    None,
    /// Compress using zlib
    Zlib,
    #[cfg(feature = "zstd")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "zstd")))]
    /// Compress using zstd
    ZStd,
}

impl RecordCompression {
    pub(crate) fn to_slow5_rep(self) -> u32 {
        match self {
            Self::None => slow5_press_method_SLOW5_COMPRESS_NONE,
            #[cfg(feature = "zstd")]
            Self::ZStd => slow5_press_method_SLOW5_COMPRESS_ZSTD,
            Self::Zlib => slow5_press_method_SLOW5_COMPRESS_ZLIB,
        }
    }

    #[allow(non_upper_case_globals)]
    pub(crate) fn from_u32(n: u32) -> Self {
        match n {
            slow5_press_method_SLOW5_COMPRESS_NONE => Self::None,
            slow5_press_method_SLOW5_COMPRESS_ZLIB => Self::Zlib,
            #[cfg(feature = "zstd")]
            slow5_press_method_SLOW5_COMPRESS_ZSTD => Self::ZStd,
            _ => unreachable!("Invalid record compression"),
        }
    }
}

/// SLOW5 signal compression
#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy)]
pub enum SignalCompression {
    /// No signal compression
    None,
    /// Compress using streamvbyte library
    StreamVByte,
}

impl SignalCompression {
    pub(crate) fn to_slow5_rep(self) -> u32 {
        match self {
            SignalCompression::None => slow5_press_method_SLOW5_COMPRESS_NONE,
            SignalCompression::StreamVByte => slow5_press_method_SLOW5_COMPRESS_SVB_ZD,
        }
    }

    #[allow(non_upper_case_globals)]
    pub(crate) fn from_u32(x: u32) -> Self {
        match x {
            slow5_press_method_SLOW5_COMPRESS_NONE => SignalCompression::None,
            slow5_press_method_SLOW5_COMPRESS_SVB_ZD => SignalCompression::StreamVByte,
            _ => unreachable!("Invalid compression"),
        }
    }
}
