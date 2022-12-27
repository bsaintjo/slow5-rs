use slow5::FileReader;

fn main() -> anyhow::Result<()> {
    let mut reader = FileReader::open("examples/example.slow5")?;
    for read in reader.records() {
        let read = read?;
        let json = serde_json::to_string_pretty(&read)?;
        println!("{json}");
    }
    Ok(())
}
