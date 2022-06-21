use anyhow::Result;
use slow5::{FileReader, RecordExt, SignalIterExt};

fn main() -> Result<()> {
    let mut acc = Vec::new();
    let reader = FileReader::open("examples/example.slow5")?;
    for read in reader.records() {
        let read = read?;
        println!("{:?}", read.read_id());
        for signal in read.signal_iter() {
            acc.push(signal);
        }
    }

    println!("length {}", acc.len());
    Ok(())
}
