use anyhow::Result;
use assert_fs::TempDir;
use slow5::{FieldType, FileWriter, Header, Record, RecordBuilder};

fn main() -> Result<()> {
    let tmp_dir = TempDir::new()?;
    let file_path = tmp_dir.join("test.blow5");
    let mut slow5 = FileWriter::create(file_path)?;

    set_header_attributes(&mut slow5.header())?;

    let mut hdr = slow5.header();
    set_header_aux_fields(&mut hdr)?;
    let rec = set_record_fields(&hdr)?;
    slow5.add_record(&rec)?;
    slow5.close();

    Ok(())
}

fn set_record_fields(hdr: &Header) -> Result<Record> {
    let raw_signal = (0..10).collect::<Vec<_>>();
    let mut rec = RecordBuilder::builder()
        .read_id(b"read_0")
        .digitisation(4096.)
        .offset(3.0)
        .sampling_rate(4000.)
        .raw_signal(&raw_signal)
        .build()?;
    rec.set_aux_field(hdr, "median", 1.2f32)?;
    Ok(rec)
}

fn set_header_attributes(hdr: &mut Header) -> Result<()> {
    hdr.add_attribute("run_id")?;
    hdr.set_attribute("run_id", "run_0", 0)?;
    hdr.add_attribute("asic_id")?;
    hdr.set_attribute("asic_id", "asic_id_0", 0)?;
    Ok(())
}

fn set_header_aux_fields(hdr: &mut Header) -> Result<()> {
    hdr.add_aux_field("median", FieldType::Float)?;
    Ok(())
}
