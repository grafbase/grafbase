# Changelog

## Unreleased

### Changed

- The library no longer produces changes for fields of newly added types, variants of newly added unions, values of enums, etc. This turned out to be too verbose. This could be made configurable if there is interest. (https://github.com/grafbase/grafbase/pull/1421)

### Fixed

- Handle empty schema strings gracefully

## 0.1.1 - 2024-02-06

- Writte a better README

## 0.1.0 - 2024-01-25

- This is the initial release. The crate is feature complete, and we are
  starting to use it in production.
