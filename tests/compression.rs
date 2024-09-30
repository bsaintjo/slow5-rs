/// Test all combinations of signal and record compression and decompression
use assert_fs::{prelude::PathChild, TempDir};
use slow5::{
    FieldType, FileReader, FileWriter, RecordBuilder, RecordCompression, RecordExt,
    SignalCompression,
};

#[test]
fn test_compression() {
    let tmp_dir = TempDir::new().unwrap();

    let record_compressions = [
        RecordCompression::None,
        RecordCompression::ZStd,
        RecordCompression::Zlib,
    ];
    let signal_compressions = [
        SignalCompression::ExZd,
        SignalCompression::None,
        SignalCompression::StreamVByte,
    ];
    for (rec_idx, rec_comp) in record_compressions.iter().enumerate() {
        for (sig_idx, signal_comp) in signal_compressions.iter().enumerate() {
            let file_path = tmp_dir.child(format!("new_{rec_idx}_{sig_idx}.blow5"));
            let mut writer = FileWriter::options()
                .attr("attr", "val", 0)
                .attr("attr", "other", 1)
                .num_read_groups(3)
                .unwrap()
                .aux("median", FieldType::Float)
                .aux("read_number", FieldType::Uint32)
                .aux("string", FieldType::Str)
                .aux("not set", FieldType::Uint16)
                .signal_compression(*signal_comp)
                .record_compression(*rec_comp)
                .create(&file_path)
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

            let reader = FileReader::open(&file_path).unwrap();
            assert_eq!(reader.record_compression(), *rec_comp);
            assert_eq!(reader.signal_compression(), *signal_comp);
            let rec = reader.get_record("read_2").unwrap();
            assert_eq!(rec.read_group(), 2);
            assert_eq!(
                rec.raw_signal_iter().collect::<Vec<_>>(),
                signals[2].to_vec()
            );
            assert_eq!(rec.get_aux_field::<f32>("median").unwrap(), 10.0f32);
            assert_eq!(rec.get_aux_field::<&str>("string").unwrap(), "here");
            assert!(rec.get_aux_field::<i16>("not set").is_err());
        }
    }
}
