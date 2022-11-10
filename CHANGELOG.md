# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - yyyy-mm-dd

### Added

- aux: Field represents an auxiliary field from a header and add/set a Records auxiliary field with type-checking
- aux: AuxField trait map Rust types and C types that can represent auxiliary fields, currently only supports primitive data types
- writer: FileWriter can now be opened with compression options using Options and FileWriter::with_options
- writer: Added FileWriter::header to get access to SLOW5 header
- record: Added Record::get_aux_field and Record::set_aux_field for accessing Record auxiliary fields
- record: Added PicoAmpsSignalIter and RawSignalIter for signal data iteration
- reader: Added ReadIdIter and FileReader::read_id_iter to efficiently iterate over all read ids in a SLOW5 file
- headers: Added AuxNamesIter and Header::aux_names_iter for going over auxiliary name keys
- docs: Fill out documentation for more items
- api: RecordPointer wraps the raw pointer and try to avoid leaking API
- api: experimental module added to try different APIs
- examples: New example slow5 and blow5 files for testing and examples
- examples: auxiliary_field.rs to model slow5lib example

### Changed

- RecordIter returns a Record

### Deprecated

### Removed

- picoamps argument to some of the Record/RecordView
- remove SignalIterExt and combine with RecordExt

### Fixed

- bug: use-after-free bug in RecordIter removed
