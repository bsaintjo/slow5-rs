use anyhow::Result;
use assert_fs::TempDir;
use slow5::{FieldType, FileWriter, Header, Record};

fn main() -> Result<()> {
    let tmp_dir = TempDir::new()?;
    let file_path = tmp_dir.join("test.blow5");
    let mut slow5 = FileWriter::options()
        .attr("run_id", "run_0", 0)
        .attr("asic_id", "asic_id_0", 0)
        .aux("median", FieldType::Float)
        .create(file_path)?;

    let hdr = slow5.header();
    let rec = set_record_fields(&hdr)?;
    slow5.add_record(&rec)?;
    slow5.close();

    println!("Success!");
    Ok(())
}

fn set_record_fields(hdr: &Header) -> Result<Record> {
    let raw_signal = (0..10).collect::<Vec<_>>();
    let mut rec = Record::builder()
        .read_id("read_0")
        .read_group(0)
        .range(12.0)
        .digitisation(4096.)
        .offset(3.0)
        .sampling_rate(4000.)
        .raw_signal(&raw_signal)
        .build()?;
    rec.set_aux_field(hdr, "median", 1.2f32)?;
    Ok(rec)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        main().unwrap()
    }
}
