use assert_fs::{prelude::PathChild, TempDir};
use slow5::{
    FieldType, FileWriter, HeaderExt, RecordBuilder, RecordCompression, SignalCompression,
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
        .signal_compression(SignalCompression::StreamVByte)
        .record_compression(RecordCompression::ZStd)
        .create(&file_path)?;
    assert_eq!(writer.get_attribute("attr", 0)?, b"val");
    assert_eq!(writer.get_attribute("attr", 1)?, b"other");
    assert_eq!(writer.aux_names_iter()?.count(), 2);

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
        rec.set_aux_field(&writer, "median", 10.0f32)?;
        rec.set_aux_field(&writer, "read_number", 7)?;
        writer.add_record(&rec)?;
    }
    writer.close();
    Ok(())
}
