# Change Log

All notable changes to this project will be documented in this file. 
This project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased] - ReleaseDate

## [v0.1.0] - 2023-05-01

### Added

- Initial release
- Compares all branches in repository and displays how they can be merged to each other
- Uses `git` CLI to pull remote into temporary local repository
- Builds available for Linux (x86_64, gnu), macOS(x86_64, aarch64) and Windows (x86_64, gnu)
- Detecting one of the following merge outcomes:
  - No conflicts, fast-forward is possible
  - No merge possible due to absence of common ancestor
  - No merge possible due to normal merge check failure
  - No changes: branches are up-to-date
  - Found conflicts, cannot resolve automatically
  - Found conflicts, can resolve automatically
- Output is given in terminal application as a table