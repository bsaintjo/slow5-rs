use anyhow::Result;
use slow5::Builder;

fn main() -> Result<()> {
    let mut acc = Vec::new();
    let mut slow5_file = Builder::default()
        .picoamps(true)
        .open("examples/example.slow5")?;
    let mut iter = slow5_file.seq_reads();
    while let Some(Ok(slow5_rec)) = iter.next() {
        println!("{}", std::str::from_utf8(slow5_rec.read_id())?);
        for signal in slow5_rec.signal_iter() {
            acc.push(signal)
        }
    }

    println!("length {}", acc.len());
    Ok(())
}
