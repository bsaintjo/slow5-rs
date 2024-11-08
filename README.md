# slow5-rs

[![License][license-badge]][license-url]
[![Crates.io][crates-badge]][crates-url]
[![docs.rs][docs-badge]][docs-url]
[![Rust][ci-badge]][ci-url]
[![stable][stability-badge]][stability-url]
[![codecov][codecov-badge]][codecov-url]

[license-badge]: https://img.shields.io/crates/l/slow5?style=flat-square
[license-url]: https://github.com/bsaintjo/slow5-rs#license
[crates-badge]: https://img.shields.io/crates/v/slow5?style=flat-square
[crates-url]: https://crates.io/crates/slow5
[docs-badge]: https://img.shields.io/docsrs/slow5?style=flat-square
[docs-url]: https://docs.rs/slow5
[ci-badge]: https://github.com/bsaintjo/slow5-rs/actions/workflows/rust.yml/badge.svg
[ci-url]: https://github.com/bsaintjo/slow5-rs/actions/workflows/rust.yml
[codecov-badge]: https://codecov.io/gh/bsaintjo/slow5-rs/branch/main/graph/badge.svg?token=MODXRVRNQ0
[codecov-url]: https://codecov.io/gh/bsaintjo/slow5-rs
[stability-badge]: https://img.shields.io/badge/stability-stable-green.svg
[stability-url]: http://github.com/badges/stability-badges
A library for interacting with SLOW5/BLOW5 files in the Rust programming language.

For more information check out the main [slow5lib repository](https://github.com/hasindu2008/slow5lib) or the [paper](https://doi.org/10.1038/s41587-021-01147-4).

This repository provides two crates:

- `slow5lib-sys`: Bindings to the C library using `bindgen`
- `slow5`: Rust API built on-top of `slow5lib-sys`

The interface for `slow5` is largely stable and now updated primarily to follow new versions `slow5lib`. If there is additional functionality you'd like to see, please file a Github Issue.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
slow5 = "0.12"
```

### Git

If you'd like to download the git version, use the following command to download the repo

```bash
git clone --recursive https://github.com/bsaintjo/slow5-rs.git
```

## Getting started

### Reading signal from SLOW5 file

```rust
use slow5::{FileReader, RecordExt};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut slow5 = FileReader::open("examples/example.slow5").unwrap();
    for record in slow5.records() {
        for signal in record?.picoamps_signal_iter() {
            // Do stuff
        }
    }
    Ok(())
}
```

### Writing a compressed BLOW5 file with attributes

```rust
use std::error::Error;
use slow5::{FileWriter, SignalCompression, Record};

fn main() -> Result<(), Box<dyn Error>> {
    let tmp_dir = std::env::temp_dir();
    let output = tmp_dir.join("test.blow5");
    let mut writer = FileWriter::options()
        .signal_compression(SignalCompression::StreamVByte)
        .attr("attribute", "value", 0)
        .create(output)?;
    let rec = Record::builder()
        .read_id("test_id")
        .read_group(0)
        .digitisation(4096.0)
        .offset(4.0)
        .range(12.0)
        .sampling_rate(4000.0)
        .raw_signal(&[0, 1, 2, 3])
        .build()?;
    writer.add_record(&rec)?;
    writer.close();
    Ok(())
}
```

### More example code

For more example code, see the [test code](tests/full.rs). Several examples from the original library have been ported to rust and can be found in the [examples](./examples/) directory.

## Feature flags

- `zstd`:       Enable zstd-based compression
  - enabled by default
- `zlib-ng`:    Enable usage of high performance zlib-ng
  - adds `cmake` dependency
- `serde`:      Enable `serde` dependency
  - Mainly for serializing `Record` into `serde`-compatible formats

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
