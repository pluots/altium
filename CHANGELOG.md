# Changelog

<!-- next-header -->

## [Unreleased] - ReleaseDate

### Added

### Changed

### Removed



## [0.2.1] - 2023-10-23

Implement std `Error` for `altium::Error`, as well as a better `Debug`
implementation.



## [0.2.0] - 2023-10-23

### Added

- Expose a public APIs for schematic records, `Draw`, `Canvas`, `Storage` and
  many others. This is all unstable but will allow for some testing.

### Changed

- Improve drawing traits
- Update unit conversions and location parsing
- Clarify documentation
- Allow `save_svg` to overwrite existing files

## [0.1.0] - 2023-07-25

Initial release. This includes:

- Listing and retrieving components in schematic libraries
- Very basic drawing of these components to SVG works
- Very pasic `.PrjPcb` support

<!-- next-url -->
[Unreleased]: https://github.com/pluots/altium/compare/altium-v0.2.1...HEAD
[0.2.1]: https://github.com/pluots/altium/compare/altium-v0.2.0...altium-v0.2.1
[0.2.0]: https://github.com/pluots/altium/compare/altium-v0.1.0...altium-v0.2.0
[0.1.0]: https://github.com/pluots/altium/compare/490216bd119f...altium-v0.1.0
