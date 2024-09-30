use std::path::Path;

use anyhow::Ok;
use assert_fs::{prelude::PathChild, TempDir};
use slow5::{
    FieldType, FileReader, FileWriter, RecordBuilder, RecordCompression, RecordExt,
    SignalCompression,
};

fn write_test_file(
    file_path: &Path,
    signal_comp: SignalCompression,
    rec_comp: RecordCompression,
) -> anyhow::Result<()> {
    let mut writer = FileWriter::options()
        .attr("attr", "val", 0)
        .attr("attr", "other", 1)
        .num_read_groups(3)
        .unwrap()
        .aux("median", FieldType::Float)
        .aux("read_number", FieldType::Uint32)
        .aux("string", FieldType::Str)
        .aux("not set", FieldType::Uint16)
        .signal_compression(signal_comp)
        .record_compression(rec_comp)
        .create(file_path)?;

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
        writer.add_record(&rec).unwrap();
    }
    writer.close();
    Ok(())
}

#[test]
fn test_overwrite() {
    let tmp_dir = TempDir::new().unwrap();
    if let Err(e) = (|| {
        let example = tmp_dir.child("example.blow5");
        write_test_file(&example, SignalCompression::None, RecordCompression::None)?;
        write_test_file(
            &example,
            SignalCompression::StreamVByte,
            RecordCompression::ZStd,
        )?;

        let reader = FileReader::open(&example)?;
        anyhow::ensure!(reader.signal_compression() == SignalCompression::StreamVByte);
        anyhow::ensure!(reader.record_compression() == RecordCompression::ZStd);
        let rec = reader.get_record("read_2")?;
        anyhow::ensure!(rec.read_group() == 2);

        let signals = [[0, 1, 2], [3, 4, 5], [6, 7, 8]];
        assert_eq!(
            rec.raw_signal_iter().collect::<Vec<_>>(),
            signals[2].to_vec()
        );
        anyhow::ensure!(rec.get_aux_field::<f32>("median").unwrap() == 10.0f32);
        anyhow::ensure!(rec.get_aux_field::<&str>("string").unwrap() == "here");
        anyhow::ensure!(rec.get_aux_field::<i16>("not set").is_err());
        anyhow::Result::Ok(())
    })() {
        eprintln!("Error: {e:?}");
    } else {
        println!("Success!");
    }
}
