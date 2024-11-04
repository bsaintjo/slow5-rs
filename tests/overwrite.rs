use std::path::Path;

use assert_fs::TempDir;
use slow5::{FieldType, FileWriter, RecordBuilder, RecordCompression, SignalCompression};

fn write_test_file(file_path: &Path, signal_comp: SignalCompression, rec_comp: RecordCompression) {
    // let file_path = tmp_dir.child(format!("new_{rec_idx}_{sig_idx}.blow5"));
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
        .create(file_path)
        .unwrap();

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
            .build()
            .unwrap();
        rec.set_aux_field(&mut writer, "median", 10.0f32).unwrap();
        rec.set_aux_field(&mut writer, "read_number", 7u32).unwrap();
        rec.set_aux_field(&mut writer, "string", "here").unwrap();
        writer.add_record(&rec).unwrap();
    }
    writer.close();
}

#[test]
fn test_overwrite() {
    let tmp_dir = TempDir::new().unwrap();
}
