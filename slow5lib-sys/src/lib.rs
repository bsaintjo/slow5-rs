#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    #[test]
    fn test_slow5_open() {
        let file = CString::new("slow5lib/examples/example.slow5").expect("CString::new failed");
        let mode = CString::new("r").expect("CString::new failed");
        unsafe {
            crate::slow5_open(file.as_ptr(), mode.as_ptr());
        }
    }
}
