extern crate slow5lib_sys;

use std::error::Error;
use std::ptr::null_mut;

fn to_picoamps(raw_val: i16, digitisation: f64, offset: f64, range: f64) -> f64 {
    ((raw_val as f64) + offset) * (range / digitisation)
}

#[test]
fn main() -> Result<(), Box<dyn Error>> {
    unsafe {
        let file_path = cstr::cstr!("slow5lib/examples/example.slow5");
        let mode = cstr::cstr!("r");
        let sp = slow5lib_sys::slow5_open(file_path.as_ptr(), mode.as_ptr());

        let mut rec: *mut slow5lib_sys::slow5_rec_t = null_mut();

        let ret = slow5lib_sys::slow5_idx_load(sp);
        if ret < 0 {
            panic!("Error in loading index");
        }

        let read_id = cstr::cstr!("r3");
        let rec_mut_ptr: *mut *mut slow5lib_sys::slow5_rec_t = &mut rec;
        let ret = slow5lib_sys::slow5_get(read_id.as_ptr(), rec_mut_ptr, sp);
        if ret < 0 {
            panic!("Error in fetching the read");
        } else {
            println!("{:?}", (*rec).read_id);
            let len_raw_signal = (*rec).len_raw_signal;
            for i in 0..len_raw_signal {
                let picoamp = to_picoamps(
                    *(*rec).raw_signal.offset(i as isize),
                    (*rec).digitisation,
                    (*rec).offset,
                    (*rec).range,
                );
                println!("{}", picoamp);
            }
        }

        slow5lib_sys::slow5_rec_free(rec);
        slow5lib_sys::slow5_idx_unload(sp);
        slow5lib_sys::slow5_close(sp);
        Ok(())
    }
}
