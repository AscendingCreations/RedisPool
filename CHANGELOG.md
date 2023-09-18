# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)

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
