use anyhow::Result;
use slow5::Builder;

#[test]
fn main() -> Result<()> {
    let file_path = "examples/example.slow5";
    let mut slow5 = Builder::default().open(file_path)?;
    let rec = slow5.get_read(b"r3")?;
    assert_eq!(b"r3", rec.read_id());
    Ok(())
}
