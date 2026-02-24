# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- The signers file is no longer written if an update does not produce any entries. 

### Fixed

- `hanko signer add` no longer appends a duplicate to the configuration if an identical signer already exists.
- Redefining a built-in source in configuration now returns an explicit error instead of silently being ignored.

## [1.0.0] - 2025-10-19

### Added

- Use of http2 without prior negotiation for included sources.
- Support for pagination for included sources.
- Expiration to GitLab keys that have it set.

### Internal

- Use trusted publishing in release workflow.
- Bump rust edition to 2024.
- Update all dependencies.

## [0.5.4] - 2025-05-07

### Added

- Features and code examples are now checked in CI.

### Internal

- Fixed some typos.
- Updated all dependencies.

## [0.5.3] - 2025-01-07

### Fixed

- Missing signer principals return an error.

## [0.5.2] - 2024-12-15

### Added

- Shell completion files for bash, zsh, fish, elvish and powershell.
- Manpages.

### Changed

- Version output now contains features and build time information.

### Internal

- Minimize enabled features for external dependencies and improve binary size.

## [0.5.1] - 2024-11-24

### Added

- Debian Packages to releases.

### Internal

- Updated all dependencies.

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

[unreleased]: https://github.com/SRv6d/hanko/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/SRv6d/hanko/compare/v0.5.4...v1.0.0
[0.5.4]: https://github.com/SRv6d/hanko/compare/v0.5.3...v0.5.4
[0.5.3]: https://github.com/SRv6d/hanko/compare/v0.5.2...v0.5.3
[0.5.2]: https://github.com/SRv6d/hanko/compare/v0.5.1...v0.5.2
[0.5.1]: https://github.com/SRv6d/hanko/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/SRv6d/hanko/compare/v0.4.1...v0.5.0
[0.4.1]: https://github.com/SRv6d/hanko/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/SRv6d/hanko/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/SRv6d/hanko/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/SRv6d/hanko/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/SRv6d/hanko/compare/v0.1.0...v0.2.0
