#[cfg(feature = "zstd")]
use slow5lib_sys::slow5_press_method_SLOW5_COMPRESS_ZSTD;
use slow5lib_sys::{
    slow5_press_method_SLOW5_COMPRESS_NONE, slow5_press_method_SLOW5_COMPRESS_SVB_ZD,
    slow5_press_method_SLOW5_COMPRESS_ZLIB, slow5_press_method_SLOW5_COMPRESS_EX_ZD
};

/// SLOW5 record compression
#[non_exhaustive]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum RecordCompression {
    /// No compression
    None,
    /// Compress using zlib
    Zlib,
    #[cfg(feature = "zstd")]
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
    #[allow(non_snake_case)]
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
#[non_exhaustive]
#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy)]
pub enum SignalCompression {
    /// No signal compression
    None,
    /// Compress using streamvbyte library
    StreamVByte,
    /// Compress using ex-zd algorithm
    ExZd
}

impl SignalCompression {
    pub(crate) fn to_slow5_rep(self) -> u32 {
        match self {
            SignalCompression::None => slow5_press_method_SLOW5_COMPRESS_NONE,
            SignalCompression::StreamVByte => slow5_press_method_SLOW5_COMPRESS_SVB_ZD,
            SignalCompression::ExZd => slow5_press_method_SLOW5_COMPRESS_EX_ZD,
        }
    }

    #[allow(non_upper_case_globals)]
    #[allow(non_snake_case)]
    pub(crate) fn from_u32(x: u32) -> Self {
        match x {
            slow5_press_method_SLOW5_COMPRESS_NONE => SignalCompression::None,
            slow5_press_method_SLOW5_COMPRESS_SVB_ZD => SignalCompression::StreamVByte,
            slow5_press_method_SLOW5_COMPRESS_EX_ZD => SignalCompression::ExZd,
            _ => unreachable!("Invalid compression"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sig_comp_from_u32() {
        let method = slow5_press_method_SLOW5_COMPRESS_EX_ZD;
        assert_eq!(SignalCompression::from_u32(method), SignalCompression::ExZd);

        let method = slow5_press_method_SLOW5_COMPRESS_SVB_ZD;
        assert_eq!(SignalCompression::from_u32(method), SignalCompression::StreamVByte);
    }
}