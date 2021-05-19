# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.1]

### Changed

- Implemented workaround for allocating sets with empty layouts.
- Erupt integration now supports acceleration structures.

## [0.2.0]

### Added
- This CHANGELOG file.

### Changed

- Leaked sets now not get reported on panic.

## [0.1.1] - 2021-02-08

### Added
- CI check for no_std

### Fixed
- Prevented second panic when cleanup check fails on unwinding.

## [0.1.1] - 2021-01-30

### Added
- Initial Implementation for general purpose gpu descriptor allocator.
- Integration with gfx-hal crate.
