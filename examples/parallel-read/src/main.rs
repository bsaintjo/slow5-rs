use rayon::prelude::{ParallelBridge, ParallelIterator};
use slow5::{FileReader, RecordExt};

// TODO Need a better example to illustrate parallelization
fn main() -> anyhow::Result<()> {
    let mut reader = FileReader::open("examples/example3.blow5")?;
    let record_iter = reader.records();
    let res: f64 = record_iter
        .par_bridge()
        .fold(
            || 0.0,
            |a, b| {
                let b = b.unwrap();
                a + b.digitisation()
            },
        )
        .sum();
    println!("{res}");
    Ok(())
}
