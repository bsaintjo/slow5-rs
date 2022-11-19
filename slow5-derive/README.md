# slow5-derive

Crate for easily deriving auxiliary fields

## Purpose

The purpose of this crate is to provide support for creating structs that initialize the auxiliary fields in a SLOW5 file.

## Planned API

```rust
#[derive(FieldExt)]
struct MyAuxFields {
    #[field]
    median_before: f64,

    #[field]
    read_number: u32,

    some_other_fields: Vec<u32>
}

// Reading

let slow5: FileReader<MyAuxFields> = FileReader::open("example.slow5")?;
let rec: Record<MyAuxFields> = slow.get_read(b"read_1")?;
let median_before = rec.aux().median_before();
```
