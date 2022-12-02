use std::error::Error;

use slow5::typed::{reader::FileReader, FieldExt};

#[allow(dead_code)]
#[derive(FieldExt)]
struct MyAuxFields {
    // Primitive types only supported for now
    // Haven't implemented *char, arrays, enums, yet.
    // channel_number: String,
    median_before: f64,
    read_number: u32,
    start_mux: u8,
    start_time: u64,
}

fn main() -> Result<(), Box<dyn Error>> {
    let _slow5: FileReader<MyAuxFields> = FileReader::open("examples/example2.slow5")?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        main().unwrap()
    }
}
