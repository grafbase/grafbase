# Changelog

## Unreleased

- Properly validate that object, interface and input object types define at least one field (https://github.com/graphql/graphql-spec/blame/October2021/spec/Section%203%20--%20Type%20System.md#L868). Previously, we were validating against `type Test {}` but not `type Test`.

## [0.1.3] - 2024-02-06

- fix typo in error (#1308)
