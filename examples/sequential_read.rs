use anyhow::Result;
use slow5::Builder;

fn main() -> Result<()> {
    let mut slow5_file = Builder::default()
        .picoamps(true)
        .open("examples/example.slow5")?;
    for slow5_rec in slow5_file.seq_reads().flatten() {
        for signal in slow5_rec.signal_iter() {
            println!("{}", signal);
        }
    }
    Ok(())
}
