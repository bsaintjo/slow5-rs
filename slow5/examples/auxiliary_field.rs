use anyhow::Result;
use slow5::FileReader;

fn main() -> Result<()> {
    let mut slow5 = FileReader::open("slow5/examples/example2.slow5")?;
    let rec = slow5.records().next().unwrap()?;
    let median_before: f64 = rec.get_aux_field("median_before")?;
    println!("median_before = {median_before}");

    let start_time: u64 = rec.get_aux_field("start_time")?;
    println!("start_time = {start_time}");
    Ok(())
}
