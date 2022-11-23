# slow5-derive

Crate for easily deriving auxiliary fields

## Purpose

The purpose of this crate is to provide support for creating structs that initialize the auxiliary fields in a SLOW5 file.

## Planned API

```rust
#[derive(FieldExt)]
struct MyAuxFields {
    // This is the name of a auxiliary field in the SLOW5
    #[field]
    median_before: f64,

    // This one too
    #[field]
    read_number: u32,

    // This one isn't in the SLOW5 (still TODO atm)
    some_other_fields: Vec<u32>
}

// Reading

// Readers and records and the respective auxiliary fields associated
// with each other.
let slow5: FileReader<MyAuxFields> = FileReader::open("example.slow5")?;
let rec: Record<MyAuxFields> = slow.get_read(b"read_1")?;
// deriving FieldExt automatically adds typed getters for your records 
let median_before = MyAuxFields::median_before(&rec);

// Writing

// The new slow5 file automatically has the auxiliary fields intialized (TODO)
let slow5 = FileWriter<MyAuxFields> = FileWriter::create("new.slow5")?;

// Ideally, you have to build a record that contains all the auxiliary fields,
// otherwise you get a compile(? or runtime) error.
let rec = ...
```

## TODO

- [ ] Implement #[field] attribute, for now, struct must only contain fields that are in the SLOW5 file
- [ ] Implement methods for writing
