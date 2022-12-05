# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - yyyy-mm-dd

### Added

### Changed

### Deprecated

### Removed

### Fixed

## [0.9.0] - yyyy-mm-dd

### Added

Added support for string (char *) and array auxiliary fields.
Error for invalid file path based on the file extension.
FileWriter::record_compression + FileWriter::signal_compression to check what compression was used.
Debug implementations

### Changed

FileWriter returns Err(Slow5Error::Slow5CompressionError) if compression options are set for SLOW5 file output

## [0.8.1] - 2022-12-03

### Fixed

Fixed link to documentation on crates.io

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
