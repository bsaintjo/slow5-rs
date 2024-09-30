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

## [0.10.0] - 2024-11-04

### Added

- Support new compression algorithm with slow5_press_method_SLOW5_COMPRESS_EX_ZD
- fn slow5lib_sys::slow5_arr_qts_round
- fn slow5lib_sys::slow5_rec_qts_round
- fn slow5lib_sys::slow5_idx_init_empty
- fn slow5lib_sys::slow5_idx_read
- fn slow5lib_sys::slow5_set_skip_rid

### Changed

- Bump version of `slow5lib` to v1.3.0

## [0.9.1] - 2024-03-11

### Added

- Add `__gnuc_va_list` type to fix compilation errors
