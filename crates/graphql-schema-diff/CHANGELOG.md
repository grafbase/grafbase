# Changelog

## Unreleased

- Enrich `Change` with spans pointing to the added / changed / removed parts of schemas (https://github.com/grafbase/grafbase/pull/2014)
- New top-level export: `patch()`. This lets you take a diff and spans resolved from it, and apply it to a schema. (https://github.com/grafbase/grafbase/pull/2072)
- The `ChangeKind` enum now has a `ChangeKind::as_str()` function and a `FromStr` implementation, implementing respectively its conversion to and from strings.
- Implemented `ChangeKind::AddSchemaExtension` and `ChangeKind::RemoveSchemaExtension`. Schema extensions are assumed to be in the same order between the schemas.
- BREAKING: Overhaul the path string format, and add a typed version (`Path`) with parsing and display implementations.
- BREAKING: `diff()` no longer emits `AddFieldArgument` where there is already an `AddField` for the parent field, it will only be emitted if the field existed in the source schema. This is for consistency with similar nesting cases.
- BREAKING: `diff()` no longer emits `AddInterfaceImplementation` where there is already an `AddObject` or `AddInterface` for the parent type. It will only be added if the parent type existed in the source schema. This is for consistency with similar nesting cases. The converse also applies for `RemoveInterfaceImplementation`.
- Add `diff_asts()` entrypoint to diff without parsing, and infallibly, if you already have `cynic-parser` ASTs.
- Implemented patching for added and removed field arguments (https://github.com/grafbase/grafbase/pull/3302)

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
