# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)

## 0.4.0 (12. April, 2024)
### Changed
- (Breaking) Updated Redis to 0.25.3, This removes the current support for PubSub abd Monitor. We will attempt to readd these back in.

## 0.3.0 (21. December, 2023)
### Changed
- (Breaking) Updated Redis to 0.24.0, Note Redis 7.2 with resp3 can cause breaking changes to older code. @cking
- Updated Ping test to support newer Redis with resp3 @KrisCarr

## 0.2.1 (18. September, 2023)
### Added
- Redis Pubsub and monitor support
- Factory Exposure
- Testing 

## 0.2.0 (9. September, 2023)

- make redis pool struct generic to work with both single instance and clustered connections
- remove lock from pool queue
- add optional upper bound on amount of connections
- add basic automated integration tests

## 0.1.1 (7. September, 2023)

### Fixed

- removed Tokio mutex as it caused issues on Drop.

## 0.1.0 (7. September, 2023)

### Added

- Initial release.
