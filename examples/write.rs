use anyhow::Result;
use assert_fs::TempDir;
use slow5::{Field, FieldType, FileWriter, Header, Record, RecordBuilder};

fn main() -> Result<()> {
    let tmp_dir = TempDir::new()?;
    let file_path = tmp_dir.join("test.blow5");
    let mut slow5 = FileWriter::create(file_path)?;

    set_header_attributes(&mut slow5.header())?;

    {
        let hdr = slow5.header();
        let auxs = MyAuxFields::set_header_aux_fields(&hdr)?;
        let rec = set_record_fields(&auxs)?;
        slow5.add_record(&rec)?;
    }

    Ok(())
}

fn set_record_fields(auxs: &MyAuxFields) -> Result<Record> {
    let raw_signal = (0..10).collect::<Vec<_>>();
    let mut rec = RecordBuilder::builder()
        .read_id(b"read_0")
        .digitisation(4096.)
        .offset(3.0)
        .sampling_rate(4000.)
        .raw_signal(&raw_signal)
        .build()?;
    auxs.set_record_aux_fields(&mut rec)?;
    Ok(rec)
}

fn set_header_attributes(hdr: &mut Header) -> Result<()> {
    hdr.add_attribute("run_id")?;
    hdr.set_attribute("run_id", "run_0", 0)?;
    hdr.add_attribute("asic_id")?;
    hdr.set_attribute("asic_id", "asic_id_0", 0)?;
    Ok(())
}

struct MyAuxFields<'a> {
    median_before: Field<'a>,
    read_number: Field<'a>,
    start_mux: Field<'a>,
    start_time: Field<'a>,
}

impl<'a> MyAuxFields<'a> {
    fn set_header_aux_fields(hdr: &'a Header) -> Result<Self> {
        let median_before = hdr.add_aux_field("median_before", FieldType::Double)?;
        let read_number = hdr.add_aux_field("read_number", FieldType::Int32)?;
        let start_mux = hdr.add_aux_field("start_mux", FieldType::Uint8)?;
        let start_time = hdr.add_aux_field("start_time", FieldType::Uint64)?;

        Ok(MyAuxFields {
            median_before,
            read_number,
            start_mux,
            start_time,
        })
    }

    fn set_record_aux_fields(&self, rec: &mut Record) -> Result<()> {
        let median_before = 0.1;
        let read_number = 10;
        let start_mux = 1;
        let start_time = 100;

        rec.set_aux_field(&self.median_before, &median_before)?;
        rec.set_aux_field(&self.read_number, &read_number)?;
        rec.set_aux_field(&self.start_mux, &start_mux)?;
        rec.set_aux_field(&self.start_time, &start_time)?;

        Ok(())
    }
}
