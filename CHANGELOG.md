# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2025-06-17

### Fixed
- Docker health check now uses `127.0.0.1` instead of `localhost` to avoid IPv6 resolution issues in Alpine containers
- This fixes the container showing as unhealthy when the service is actually running correctly

## [0.1.0] - 2025-06-17

### Added
- New `--host` CLI parameter to specify the bind address (defaults to `0.0.0.0`)
- Support for binding to all network interfaces for Docker compatibility

### Changed
- **BREAKING**: Default bind address changed from `127.0.0.1` to `0.0.0.0`
- **BREAKING**: `run_server()` and `start_server()` functions now require a host parameter
- Updated documentation to include Docker usage examples

### Security
- Server now binds to all interfaces by default. Use `--host 127.0.0.1` for localhost-only binding.

## [0.0.1] - Initial Release

### Added
- Initial implementation of BN254 key management service
- RESTful API for BLS signature operations
- Support for EigenLayer AVS operators