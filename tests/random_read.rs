use anyhow::Result;
use slow5::FileReader;
use slow5::RecordExt;

#[test]
fn main() -> Result<()> {
    let file_path = "examples/example.slow5";
    let slow5 = FileReader::open(file_path)?;
    let rec = slow5.get_record("r3")?;
    assert_eq!("r3", rec.read_id());
    Ok(())
}
