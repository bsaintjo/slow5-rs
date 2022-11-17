// use anyhow::Result;
// use assert_fs::TempDir;
// use slow5::{Field, FieldType, FileWriter, Header, Record, RecordBuilder};

use std::error::Error;

use crate::{FileWriter, Record, RecordBuilder, Header};

use super::field_t::Field;

fn example() -> Result<(), Box<dyn Error>> {
    // let tmp_dir = TempDir::new()?;
    // let file_path = tmp_dir.join("test.blow5");
    let file_path = "test.blow5";
    let mut slow5 = FileWriter::create(file_path)?;

    set_header_attributes(&mut slow5.header())?;

    {
        let mut hdr = slow5.header();
        let auxs = MyAuxFields::set_header_aux_fields(&mut hdr)?;
        let rec = set_record_fields(&auxs)?;
        slow5.add_record(&rec)?;
    }

    Ok(())
}

fn set_record_fields(auxs: &MyAuxFields) -> Result<Record, Box<dyn Error>> {
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

fn set_header_attributes(hdr: &mut Header) -> Result<(), Box<dyn Error>> {
    hdr.add_attribute("run_id")?;
    hdr.set_attribute("run_id", "run_0", 0)?;
    hdr.add_attribute("asic_id")?;
    hdr.set_attribute("asic_id", "asic_id_0", 0)?;
    Ok(())
}

struct MyAuxFields<'a> {
    median_before: Field<'a, f64>,
    read_number: Field<'a, i32>,
    start_mux: Field<'a, u8>,
    start_time: Field<'a, u64>,
}

impl<'a> MyAuxFields<'a> {
    fn init(hdr: &Header, median_before: f64, read_number: i32, start_mux: u8, start_time: u64) -> Self {
        todo!()
    }

    fn set_header_aux_fields(hdr: &'a Header) -> Result<Self, Box<dyn Error>> {
        let median_before = hdr.add_aux_field_t("median_before")?;
        let read_number = hdr.add_aux_field_t("read_number")?;
        let start_mux = hdr.add_aux_field_t("start_mux")?;
        let start_time = hdr.add_aux_field_t("start_time")?;

        Ok(MyAuxFields {
            median_before,
            read_number,
            start_mux,
            start_time,
        })
    }

    fn set_record_aux_fields(&self, rec: &mut Record) -> Result<(), Box<dyn Error>> {
        let median_before = 0.1;
        let read_number = 10;
        let start_mux = 1;
        let start_time = 100;

        // rec.set_aux_field_t(&self.median_before, &median_before)?;
        // rec.set_aux_field_t(&self.read_number, &read_number)?;
        // rec.set_aux_field_t(&self.start_mux, &start_mux)?;
        // rec.set_aux_field_t(&self.start_time, &start_time)?;

        self.median_before.aux_set(rec, median_before)?;
        self.read_number.aux_set(rec, read_number)?;
        self.start_mux.aux_set(rec, start_mux)?;
        self.start_time.aux_set(rec, start_time)?;

        Ok(())
    }
}
