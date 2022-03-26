#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![doc = include_str!("../README.md")]

use libc::*;
use libz_sys::z_stream;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    #[test]
    fn test_slow5_open() {
        let file = cstr::cstr!("slow5lib/examples/example.slow5");
        let mode = cstr::cstr!("r");
        unsafe {
            crate::slow5_open(file.as_ptr(), mode.as_ptr());
        }
    }
}
