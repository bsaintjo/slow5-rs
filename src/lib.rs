#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

mod aux;
mod compression;
mod error;
mod header;
mod reader;
mod record;
mod writer;


use std::ffi::CString;

pub use aux::FieldType;
pub use aux::AuxField;
pub use compression::{RecordCompression, SignalCompression};
pub use error::Slow5Error;
pub use header::Header;
pub use reader::FileReader;
pub use record::{to_picoamps, to_raw_signal, Record, RecordBuilder, RecordExt, RecordIter};
pub use writer::{FileWriter, WriteOptions};

pub(crate) fn to_cstring<T: Into<Vec<u8>>>(x: T) -> Result<CString, Slow5Error> {
    CString::new(x).map_err(Slow5Error::InteriorNul)
}

#[cfg(doctest)]
doc_comment::doctest!("../README.md", readme);

#[cfg(test)]
mod test {
    use assert_fs::{prelude::PathChild, TempDir};

    use super::*;

    #[test]
    fn test_rw() -> anyhow::Result<()> {
        let tmp_dir = TempDir::new()?;
        let filepath = tmp_dir.child("new.slow5");

        let mut writer = FileWriter::options()
            .attr("attr", "val", 0)
            .attr("attr", "other", 1)
            .num_read_groups(3)?
            .aux("median", FieldType::Float)
            .aux("read_number", FieldType::Uint32)
            .create(&filepath)?;
        {
            let header = writer.header();
            assert_eq!(header.get_attribute("attr", 0)?, b"val");
            assert_eq!(header.get_attribute("attr", 1)?, b"other");
            assert_eq!(header.aux_names_iter()?.count(), 2);
        }

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
            rec.set_aux_field(&writer.header(), "median", 10.0f32)?;
            rec.set_aux_field(&writer.header(), "read_number", 7)?;
            writer.add_record(&rec)?;
        }
        writer.close();
        Ok(())
    }
}
