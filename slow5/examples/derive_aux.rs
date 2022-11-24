use std::error::Error;

use slow5::typed::{FieldExt, reader::FileReader};

#[derive(FieldExt)]
struct MyFields {
    // Primitive types only supported for now
    // Haven't implemented *char, arrays, enums, yet.
    // channel_number: String,
    median_before: f64,
    read_number: u32,
    start_mux: u8,
    start_time: u64,
}

fn main() -> Result<(), Box<dyn Error>> {
    let slow5: FileReader<MyFields> = FileReader::open("slow5/examples/example2.slow5")?;
    Ok(())
}
