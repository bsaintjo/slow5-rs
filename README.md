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

A library for interacting with SLOW5 files in rust. Not official.

*Note*: Library design is in flux and care should be taken in upgrading this crate.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
slow5 = "0.5.0"
```

Note: version does not directly translate to version of slow5lib.

## TODO

- [x] Read SLOW5 files
- [x] Iterating over SLOW5 records
- [x] Iterating over picoamp measurements in records
- [x] Write slow5 file
- [ ] Handle BLOW5 files
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
