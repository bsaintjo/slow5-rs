use slow5::{FileReader, RecordExt};


#[test]
fn main() -> anyhow::Result<()> {
    let file_path = "examples/example3.blow5";
    let slow5 = FileReader::open(file_path)?;
    let mut records = slow5.records();
    while let Some(Ok(rec)) = records.next() {
        println!("{:?}", rec.read_id());
    }
    Ok(())
}
