# Changelog

## Unreleased

- When validating that the root types used in a schema definition exists,
  we were skipping the validation whenever the type is the default one for
  the root (Query, Mutation or Subscription). This commit makes the validation
  stricter. If Query does not exist, it is an error. (#1502)

- Properly validate that object, interface and input object types define at least one field (https://github.com/graphql/graphql-spec/blame/October2021/spec/Section%203%20--%20Type%20System.md#L868). Previously, we were validating against `type Test {}` but not `type Test`.
- Whenever we had a graph of input objects with multiple cycles in graphql-schema-validation, we would go into an infinite loop and stack overflow. This is fixed in this release.

## [0.1.3] - 2024-02-06

- fix typo in error (#1308)
