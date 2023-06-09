# Change Log

All notable changes to this project will be documented in this file.
The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

<!-- next-header -->

## [Unreleased] - ReleaseDate

### Changed

- Modified log output using with levels and colors when enabled (using env_logger).

## [v0.3.1] - 2023-05-14

### Fixed

- Fixed incorrect order of recently updated branches retrieval.

## [v0.3.0] - 2023-05-09

### Added

- Progress indicator for clone and analysis.
- More types of output: JSON, table, simple text, markdown.

## [v0.2.3] - 2023-05-06

### Changed

- Name of header to Merging Branches.
- Branches in table now in bold style.

## [v0.2.2] - 2023-05-06

### Fixed

- Corrected status message when normal merge is possible.
- If there was an error while reading branch name, that branch would be ignored instead of crashing.
- If there was an error while getting branch last commit, app would exit with error instead of panicing.
- Other issues during analysis would cause correct exit and error message instead of panicing.

### Changed

- Clone remote silently instead of forwarding output to stdout.
- Log output is now more compact and easier to read.
- Uses unstable sort for recent branches as it is recommended to be faster and we don't care about order of branches with the same date.

## [v0.2.1] - 2023-05-05

### Fixed

- Call prune after fetching remote to not show deleted branches.

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
[Unreleased]: https://github.com/strowk/probranchinator/compare/v0.3.1...HEAD
[v0.3.1]: https://github.com/strowk/probranchinator/compare/v0.3.0...v0.3.1
[v0.3.0]: https://github.com/strowk/probranchinator/compare/v0.2.3...v0.3.0
[v0.2.3]: https://github.com/strowk/probranchinator/compare/v0.2.2...v0.2.3
[v0.2.2]: https://github.com/strowk/probranchinator/compare/v0.2.2...v0.2.2
[v0.2.2]: https://github.com/strowk/probranchinator/compare/v0.2.1...v0.2.2
[v0.2.1]: https://github.com/strowk/probranchinator/compare/v0.2.0...v0.2.1
[v0.2.0]: https://github.com/strowk/probranchinator/compare/v0.1.0...v0.2.0
[v0.1.0]: https://github.com/strowk/probranchinator/releases/tag/v0.1.0
