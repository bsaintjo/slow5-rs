use slow5::{EnumField, FileReader, RecordExt};

fn main() -> anyhow::Result<()> {
    let mut reader = FileReader::open("examples/example3.blow5")?;
    let labels = reader
        .iter_aux_enum_labels("end_reason")?
        .map(|x| String::from_utf8(x.to_vec()))
        .collect::<Result<Vec<_>, _>>()?;
    for rec in reader.records() {
        let rec = rec?;
        let EnumField(end_reason_idx) = rec.get_aux_enum_field("end_reason")?;
        let read_id = std::str::from_utf8(rec.read_id())?;
        println!("read_id: {read_id}");
        println!("{:?}", labels[end_reason_idx]);
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_aux_field_enum() {
        main().unwrap()
    }
}
