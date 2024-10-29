# Changelog

## Unreleased

- Enrich `Change` with spans pointing to the added / changed / removed parts of schemas (https://github.com/grafbase/grafbase/pull/2014)
- New top-level export: `patch()`. This lets you take a diff and spans resolved from it, and apply it to a schema. (https://github.com/grafbase/grafbase/pull/2072)
- The `ChangeKind` enum now has a `ChangeKind::as_str()` function and a `FromStr` implementation, implementing respectively its conversion to and from strings.
- Implemented `ChangeKind::AddSchemaExtension` and `ChangeKind::RemoveSchemaExtension`. Schema extensions are assumed to be in the same order between the schemas.

## 0.2.0 - 2024-07-16

### Changed

- The library no longer produces changes for fields of newly added types, variants of newly added unions, values of enums, etc. This turned out to be too verbose. This could be made configurable if there is interest. (https://github.com/grafbase/grafbase/pull/1421)

### Fixed

- Handle empty schema strings gracefully

## 0.1.1 - 2024-02-06

- Writte a better README

## 0.1.0 - 2024-01-25

- This is the initial release. The crate is feature complete, and we are
  starting to use it in production.
