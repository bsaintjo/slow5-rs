use std::{error::Error, mem::size_of};

use libc::{c_void, malloc, strdup, strlen};
use slow5lib_sys::{
    slow5_aux_set, slow5_close, slow5_file, slow5_hdr_write, slow5_rec_free, slow5_rec_t,
    slow5_write,
};

fn main() -> Result<(), Box<dyn Error>> {
    unsafe {
        let file_path = cstr::cstr!("test.slow5");
        let mode = cstr::cstr!("w");
        let sp = slow5lib_sys::slow5_open(file_path.as_ptr(), mode.as_ptr());

        set_header_aux_fields(sp);
        slow5_hdr_write(sp);

        let slow5_record = libc::calloc(1, size_of::<slow5_rec_t>()) as *mut slow5_rec_t;

        set_record_primary_fields(slow5_record);
        set_record_aux_fields(slow5_record, sp);

        slow5_write(slow5_record, sp);
        slow5_rec_free(slow5_record);
        slow5_close(sp);

        Ok(())
    }
}

unsafe fn set_header_aux_fields(sp: *mut slow5lib_sys::slow5_file) {
    let channel_number = cstr::cstr!("median_before");
    slow5lib_sys::slow5_aux_add(
        channel_number.as_ptr(),
        slow5lib_sys::slow5_aux_type_SLOW5_DOUBLE,
        (*sp).header,
    );
}

unsafe fn set_record_primary_fields(slow5_record: *mut slow5_rec_t) {
    let id = cstr::cstr!("read_0");
    (*slow5_record).read_id = strdup(id.as_ptr());
    (*slow5_record).read_id_len = strlen((*slow5_record).read_id) as u16;
    (*slow5_record).read_group = 0;
    (*slow5_record).digitisation = 4096.0;
    (*slow5_record).offset = 3.0;
    (*slow5_record).range = 10.0;
    (*slow5_record).sampling_rate = 4000.0;
    (*slow5_record).len_raw_signal = 10;
    (*slow5_record).raw_signal = malloc(size_of::<i16>() * 10) as *mut i16;
    for i in 0i16..10 {
        *(*slow5_record).raw_signal.add(i as usize) = i;
    }
}

unsafe fn set_record_aux_fields(slow5_record: *mut slow5_rec_t, sp: *mut slow5_file) {
    let median_before: f64 = 0.1;
    let median_before_ptr = &median_before as *const _ as *const c_void;
    let field_name = cstr::cstr!("median_before");
    slow5_aux_set(
        slow5_record,
        field_name.as_ptr(),
        median_before_ptr,
        (*sp).header,
    );
}
