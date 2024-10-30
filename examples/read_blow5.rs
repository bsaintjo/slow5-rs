use slow5::{FileReader, RecordExt};

fn main() -> anyhow::Result<()> {
    let file_path = "examples/example3.blow5";
    let mut slow5 = FileReader::open(file_path)?;
    let mut records = slow5.records();
    while let Some(Ok(rec)) = records.next() {
        let id = std::str::from_utf8(rec.read_id())?;
        println!("{id}");
    }
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