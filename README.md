# slow5-rs

[![License][license-badge]][license-url]
[![Crates.io][crates-badge]][crates-url]
[![docs.rs][docs-badge]][docs-url]
[![Rust](https://github.com/bsaintjo/slow5-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/bsaintjo/slow5-rs/actions/workflows/rust.yml)

[license-badge]: https://img.shields.io/crates/l/slow5?style=flat-square
[license-url]: https://github.com/bsaintjo/slow5-rs#license
[crates-badge]: https://img.shields.io/crates/v/slow5?style=flat-square
[crates-url]: https://crates.io/crates/slow5
[docs-badge]: https://img.shields.io/docsrs/slow5?style=flat-square
[docs-url]: https://docs.rs/slow5

A library for interacting with SLOW5 files in rust.

*Note*: Library design is in flux and care should be taken in upgrading this crate.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
slow5 = "0.3.1"
```

Note: version does not directly translate to version of slow5lib.

## Example

### Getting record by id

```rust
fn get_by_read_id() {
    let file_path = "examples/example.slow5";
    let mut slow5_file = slow5::Builder::default().open(file_path).unwrap();
    let record = slow5_file.get_read(b"r3").unwrap();
    assert_eq!(b"r3", record.read_id());
}
```

### Iterating over records sequentially

```rust
use std::error::Error;

fn iterating_example() -> Result<(), Box<dyn Error>> {
    let file_path = "examples/example.slow5";
    let mut slow5_file = slow5::Builder::default()
        .picoamps(true)
        .open(file_path)?;
    let mut rec_iter = slow5_file.seq_reads();
    while let Some(Ok(slow5_rec)) = rec_iter.next() {
        // Iterate over every read
        for signal in slow5_rec.signal_iter() {
            // Iterate over signal measurements in pA
        }
    }
    Ok(())
}
```

## TODO

- [x] Read slow5 file
- [ ] Write slow5 file
- [x] Iterating over records
- [x] Iterating over raw or picoamp measurements
- [ ] Parity with pyslow5
- [ ] Read blow5 file (haven't tested)
- [ ] Reading headers
- [ ] Reading aux info

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
