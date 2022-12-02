use std::str::from_utf8;

use anyhow::Result;
use slow5::{FileReader, RecordExt};

fn main() -> Result<()> {
    let mut acc = Vec::new();
    let mut reader = FileReader::open("examples/example.slow5")?;
    for read in reader.records() {
        let read = read?;
        let read_id = from_utf8(read.read_id())?;
        println!("{read_id}");
        for signal in read.picoamps_signal_iter() {
            acc.push(signal);
        }
    }

    println!("length {}", acc.len());
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
