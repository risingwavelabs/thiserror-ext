# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- No-std support by setting `default-features = false`.
- Stable backtrace support for `#[thiserror_ext(newtype(.., backtrace))]` without requiring nightly APIs.
- `backtrace()` access on generated newtypes and pointer wrappers (`ErrorBox`/`ErrorArc`), returning `Option<&std::backtrace::Backtrace>`.

### Changed

- Feature flag `provide` has been renamed to `nightly` in both `thiserror-ext` and `thiserror-ext-derive`.
- `extra_provide` is gated by the `nightly` feature.

### Removed

- `backtrace` feature flag has been removed; stable backtrace support is always available.
- Old `provide` feature alias has been removed in favor of `nightly`.
