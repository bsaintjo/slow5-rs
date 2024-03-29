use anyhow::Result;
use slow5::FileReader;

fn main() -> Result<()> {
    let mut slow5 = FileReader::open("examples/example2.slow5")?;
    let rec = slow5.records().next().unwrap()?;
    let median_before: f64 = rec.get_aux_field("median_before")?;
    println!("median_before = {median_before:.2}");

    let start_time: u64 = rec.get_aux_field("start_time")?;
    println!("start_time = {start_time}");
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
