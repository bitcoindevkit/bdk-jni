# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Project
#### Added
- Add LICENSE
- Add README.md
- Add CHANGELOG.md
- Make the CI run tests on an Android emulator
- Rename everything to `bdk`
- Add gradle config and README instructions for publishing artifacts to local maven repo
- Upgrade gradle to 6.6.1
- Enable github actions ci pipeline
- Add CONTRIBUTING.md

### Rust lib and Java aar
#### Added
- Add `bdk` calls
- Add destructor
- Add list_transactions
- Include interface class & build library with gradle
- Add pipefail to build script and mkdir jniLibs folders
- Add Parcelable to WalletConstructor
- Return Transaction details data list from list_transactions
- Add android unit tests
- Add android tests
- Update `bdk` to rev `33a5ba6`
- Add multi-thread coroutine test, fix Lib.call() request string handling
- Pin `ring` version to 0.16.15

[unreleased]: https://github.com/bitcoindevkit/bdk-jni/compare/d08725cc...HEAD
