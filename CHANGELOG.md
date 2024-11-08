# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- ## [Unreleased] - yyyy-mm-dd

### Added

### Changed

### Deprecated

### Removed

### Fixed -->

## [0.12.1] - 2024-11-05

### Fixed

- Remove include to fix upload so source files build correctly

## [0.12.0] - 2024-11-04

### Added

- Derive a `Debug` implementation for `slow5::EnumField`
- MSRV verified at 1.70 on ARM MacOS
- Add support for new compression algorithm `slow5::SignalCompression::ExZd`
- Control log output from slow5lib with `slow5::log`
- Testing for all combinations of compression
- Additional `Debug` implementations
- Bump MSRV to 1.70

### Changed

- Mark `slow5::RecordCompression` and `slow5::SignalCompression` as non-exhaustive. This is to help future proof from new compression algorithms forcing a semantic versioning major release.
- Update `slow5lib-sys` to version `0.10.0`
- Update `slow5-rs` to version `0.12.0`

### Fixed

- Make `RawSignalIter` and `PicoAmpsSignalIter` iterators public

## [0.11.0]

### Changed

- bump `slow5lib` submodule to v1.1.0
- bump `slow5lib-sys` to v0.9.0

### Fixed

- Switch from deprecated `bindgen::Builder` rustfmt_bindings method to formatter

## [0.10.0] - 2022-12-27

### Added

- reader: `AuxEnumLabelIter` and `FileReader::iter_aux_enum_label` for iterating through the labels for an enum auxiliary field
- reader: `AttrKeyIter` and `FileReader::iter_attr_keys` for iterating through the keys of all the attributes
- auxiliary: `EnumField` type for getting/setting enum values for auxiliary fields
- parallel: Add `Send` impl for `FileReader`, `RecordIter`, and `Record`
- parallel: parallel read example for reading records in parallel with rayon `ParallelBridge`
- field: Add `impl Vec<B> for B: Into<Vec<u8>>` as a convience for the `FieldType::Enum` variant
- errors: Additional errors for dealing with auxiliary setting
- serde: Add `Serialize` implementation for `Record` and optional `serde` dependency

### Changed

- record: `RecordIter` now holds a `FileReader` instead of a `*mut slow5_file`
- deps: `zstd` is now an optional dependency, enabled by default but can be disabled by setting `default-features = false`
- deps: `bindgen` dependency upgraded to 0.63
- aux: AuxFieldSetExt is pub
- write: `WriteOptions::aux` can now add enums
- record: `Record::get_aux_field` now takes a impl `Into<Vec<u8>>` which should make the turbofish syntax less awkward

### Removed

- aux: Removed the `to_slow5_t` required method on `AuxField` since it wasn't necessary and making dealing with enums harder

### Fixed

- mem: Fixed potential memory leaks during error paths

## [0.9.0] - 2022-12-10

### Added

- auxiliary: Added support for string (char *) and array auxiliary fields.
- reader + writer: Error for invalid file path based on the file extension.
- compression: File*::record_compression + File*::signal_compression to check what compression was used.
- all: Debug implementations for public types
- header: HeaderExt for using File* as a Header
- record: AuxFieldSetExt trait for specifying which types are allowed to be used to set auxiliary fields.

### Changed

- writer: FileWriter returns Err(Slow5Error::Slow5CompressionError) if compression options are set for SLOW5 file output
- record: Record::set_aux_field value must implement AuxFieldSetExt instead of AuxField
- header: aux_names_iter will always return and for SLOW5 files with no auxiliary fields, it just returns an empty iterator

### Fixed

- record: Fixed bug in Record::set_aux_field where no error would occur but value would not get written to file. Fixed by extending lifetime of String used to set the aux_field and storing it in FileWriter.

## [0.8.1] - 2022-12-03

### Fixed

- Fixed link to documentation on crates.io

## [0.8.0] - 2022-12-03

### Added

- aux: AuxField trait map Rust types and C types that can represent auxiliary fields, currently only supports primitive data types
- writer: WriteOptions is a builder for FileWriter with auxiliary fields, attributes, and compression
- writer: Added FileWriter::header to get access to SLOW5 header
- record: Added Record::get_aux_field and Record::set_aux_field for accessing Record auxiliary fields
- record: Added PicoAmpsSignalIter and RawSignalIter for signal data iteration
- reader: Added ReadIdIter and FileReader::read_id_iter to iterate over all read ids in a SLOW5 file
- headers: Added AuxNamesIter and Header::aux_names_iter for going over auxiliary name keys
- docs: Fill out documentation for more items
- api: RecordPointer wraps the raw pointer and try to avoid leaking API
- examples: New example slow5 and blow5 files for testing and examples
- examples: auxiliary_field.rs to model slow5lib example

### Changed

- record: RecordIter returns a Record
- sys: slowlib-sys now compiles with SIMD based on architecture, or disables it entirely. So it should now compile on more archtectures.

### Deprecated

### Removed

- writer: FileWriter::write_record remove, didn't add much and overlaps with add_record
- record: picoamps argument to some of the Record/RecordView
- record: remove SignalIterExt and combine with RecordExt
- record: remove SignalIter and replace with Raw and PicoAmp version
- header: remove HeaderView since all header attributes will be initialized at the beginning and no reason to mutate it afterwards.
- record: RecordBuilder::builder(), now     moved to Record::builder to follow builder pattern
- record: RecordView removed because it was only used for iteration but ended creating a use-after-free bug so allocation is required anyways

### Fixed

- bug: use-after-free bug in RecordIter removed
