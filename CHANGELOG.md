# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.1] - 2024-11-24

### Added

- Debian Packages to releases.

## [0.5.0] - 2024-11-23

### Added

- Subcommand to add allowed signers.

### Changed

- Unknown fields in the configuration will now cause an error instead of being ignored.
- Loading a configuration file that does not exist will cause an error.
- All of a signers sources are now queried concurrently.
- If a user does not exist on a source a warning will be logged instead of returning an error.
- If a user does not have any keys configured on a source, a warning will be logged.

## [0.4.1] - 2024-10-29

### Security

- Update yanked [futures-rs](https://github.com/rust-lang/futures-rs) version containing use after free.

## [0.4.0] - 2024-10-28

### Added

- This Changelog.
- Build provenance file to releases.

### Removed

- Allowed signers file option from configuration file.

## [0.3.0] - 2024-10-25

### Added

- Attested container image built in CI.
- Improved logging.
- Using GitHub as default source when left unspecified.
- Tracking of test coverage.
- Logging GitHub ratelimit details when using the trace log level.

### Changed

- Make gix-config an optional dependency enabled per default by the `detect-allowed-signers` feature.

### Internal

- Improve crate structure and naming.
- Use reference counting for sources to enable usage from different threads.

## [0.2.1] - 2024-07-18

### Security

- Update gix-path to fix CVE-2024-40644.

## [0.2.0] - 2024-07-18

### Added

- Logging of API responses in trace mode.
- MIT license.

### Fixed

- aarch64 builds.

[unreleased]: https://github.com/SRv6d/hanko/compare/v0.5.1...HEAD
[0.5.1]: https://github.com/SRv6d/hanko/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/SRv6d/hanko/compare/v0.4.1...v0.5.0
[0.4.1]: https://github.com/SRv6d/hanko/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/SRv6d/hanko/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/SRv6d/hanko/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/SRv6d/hanko/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/SRv6d/hanko/compare/v0.1.0...v0.2.0
