use slow5lib_sys::slow5_file;

struct FileWriter {
    slow5_file: *mut slow5_file,
}
