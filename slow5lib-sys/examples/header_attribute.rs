use std::{error::Error, ffi::CStr};

extern crate slow5lib_sys;

fn main() -> Result<(), Box<dyn Error>> {
    unsafe {
        let file_path = cstr::cstr!("slow5lib-sys/slow5lib/examples/example.slow5");
        let mode = cstr::cstr!("r");
        let sp = slow5lib_sys::slow5_open(file_path.as_ptr(), mode.as_ptr());
        if sp.is_null() {
            panic!("Error in opening file");
        }
        let header = (*sp).header;
        let read_group = 0;
        let run_id = cstr::cstr!("run_id");
        let read_group_0_run_id_value =
            slow5lib_sys::slow5_hdr_get(run_id.as_ptr(), read_group, header);

        if !read_group_0_run_id_value.is_null() {
            let cstr = CStr::from_ptr(read_group_0_run_id_value);
            println!("{}", cstr.to_str().unwrap());
        }
        slow5lib_sys::slow5_close(sp);
        Ok(())
    }
}
