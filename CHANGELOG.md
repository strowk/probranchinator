# Change Log

All notable changes to this project will be documented in this file. 
The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

<!-- next-header -->

## [Unreleased] - ReleaseDate

## [v0.2.0] - 2023-05-01

### Added

- List of branches to analyse can now be passed as a CLI arguments.
- Exit on Ctrl+C or SIGINT.
- Configurable limit on number of recent branches to analyse if no branches are passed as CLI arguments.

### Fixed

- Remove "origin/" prefix from branch names in output.

## [v0.1.0] - 2023-05-01

### Added

- Initial release.
- Compares all branches in repository and displays how they can be merged to each other.
- Uses `git` CLI to pull remote into temporary local repository.
- Builds available for Linux (x86_64, gnu), macOS(x86_64, aarch64) and Windows (x86_64, gnu).
- Detecting one of the following merge outcomes:
  - No conflicts, fast-forward is possible.
  - No merge possible due to absence of common ancestor.
  - No merge possible due to normal merge check failure.
  - No changes: branches are up-to-date.
  - Found conflicts, cannot resolve automatically.
  - Found conflicts, can resolve automatically.
- Output is given in terminal application as a table.

<!-- next-url -->
[Unreleased]: https://github.com/strowk/probranchinator/compare/v0.2.0...HEAD
[v0.2.0]: https://github.com/strowk/probranchinator/compare/v0.1.0...v0.2.0
[v0.1.0]: https://github.com/strowk/probranchinator/releases/tag/v0.1.0
