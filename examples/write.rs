use anyhow::Result;
use assert_fs::TempDir;
use slow5::{Field, FileWriter, Header, FieldType};

fn main() -> Result<()> {
    let tmp_dir = TempDir::new()?;
    let file_path = tmp_dir.join("test.blow5");
    let slow5 = FileWriter::create(file_path)?;

    Ok(())
}

fn set_header_attributes(hdr: &mut Header) -> Result<()> {
    hdr.add_attribute("run_id")?;
    hdr.set_attribute("run_id", "run_0", 0)?;
    hdr.add_attribute("asic_id")?;
    hdr.set_attribute("asic_id", "asic_id_0", 0)?;
    Ok(())
}

fn set_header_aux_fields(hdr: &mut Header) -> Result<()> {
    let median_before = hdr.add_aux_field("median_before", FieldType::Double)?;
    let read_number = hdr.add_aux_field("read_number", FieldType::Int32)?;
    let start_mux = hdr.add_aux_field("start_mux", FieldType::Uint8)?;
    let start_time = hdr.add_aux_field("start_time", FieldType::Uint64)?;
    Ok(())
}