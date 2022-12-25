use assert_fs::{prelude::PathChild, TempDir};
use slow5::{
    FieldType, FileReader, FileWriter, HeaderExt, Record, RecordBuilder, RecordCompression,
    RecordExt, SignalCompression,
};

#[test]
fn main() -> anyhow::Result<()> {
    let tmp_dir = TempDir::new()?;
    let file_path = tmp_dir.child("new.blow5");

    let mut writer = FileWriter::options()
        .attr("attr", "val", 0)
        .attr("attr", "other", 1)
        .num_read_groups(3)?
        .aux("median", FieldType::Float)
        .aux("read_number", FieldType::Uint32)
        .aux("string", FieldType::Str)
        .aux("not set", FieldType::Uint16)
        .signal_compression(SignalCompression::StreamVByte)
        .record_compression(RecordCompression::Zlib)
        .create(&file_path)?;
    assert_eq!(writer.get_attribute("attr", 0)?, b"val");
    assert_eq!(writer.get_attribute("attr", 1)?, b"other");
    assert_eq!(writer.aux_names_iter().count(), 4);
    let aux_names: [&[u8]; 4] = [b"read_number", b"median", b"string", b"not set"];
    assert!(aux_names.contains(&writer.aux_names_iter().next().unwrap()));

    let mut builder = RecordBuilder::default();
    builder
        .digitisation(4096.0)
        .offset(4.0)
        .range(12.0)
        .sampling_rate(4000.0);
    let signals = [[0, 1, 2], [3, 4, 5], [6, 7, 8]];
    for i in 0..3 {
        let id = format!("read_{i}");
        let mut rec = builder
            .read_id(id)
            .read_group(i)
            .raw_signal(&signals[i as usize])
            .build()?;
        rec.set_aux_field(&mut writer, "median", 10.0f32)?;
        rec.set_aux_field(&mut writer, "read_number", 7u32)?;
        rec.set_aux_field(&mut writer, "string", "here")?;
        writer.add_record(&rec)?;
    }
    writer.close();

    let mut writer = FileWriter::append(&file_path)?;
    let mut rec = Record::builder()
        .read_id("read_3")
        .read_group(0)
        .digitisation(4096.0)
        .offset(4.0)
        .range(12.0)
        .sampling_rate(4000.0)
        .raw_signal(&[7, 7, 7])
        .build()?;
    rec.set_aux_field(&mut writer, "median", 10.0f32)?;
    rec.set_aux_field(&mut writer, "read_number", 7u32)?;
    rec.set_aux_field(&mut writer, "string", String::from("i am"))?;
    rec.set_aux_field(&mut writer, "not set", 123i16)?;
    writer.add_record(&rec)?;
    writer.close();

    let reader = FileReader::open(&file_path)?;
    assert_eq!(reader.record_compression(), RecordCompression::Zlib);
    assert_eq!(reader.signal_compression(), SignalCompression::StreamVByte);
    let rec = reader.get_record("read_2")?;
    assert_eq!(rec.read_group(), 2);
    assert_eq!(
        rec.raw_signal_iter().collect::<Vec<_>>(),
        signals[2].to_vec()
    );
    assert_eq!(rec.get_aux_field::<f32>("median")?, 10.0f32);
    assert_eq!(rec.get_aux_field::<&str>("string")?, "here");
    assert!(rec.get_aux_field::<i16>("not set").is_err());
    Ok(())
}
