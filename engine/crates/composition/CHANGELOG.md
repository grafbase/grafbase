# Changelog

## Unreleased

### Features

- Added composition for default values of output field arguments and input fields. They are now reflected in the federated graph.
- Support the experimental @authorized directive
- Selection sets inside `@requires` and `@provides` directives can now include inline fragments.
- The selection sets in the "fields:" argument on `@requires` are now validated against the schema, with proper errors with context when invalid.

### Fixes

- Fixed the ingestion of numeric literals when creating a federated graph from a string.
- Fixed the ingestion of `null` literals.
- In federated_graph, when parsing a schema with `@join__type` and no key argument, then rendering it with `render_federated_sdl()` would produce a `@join__type` directive

## 0.4.0 - 2024-06-11

- Ignore federation mandated fields (_entities, _service) and types (https://github.com/grafbase/grafbase/pull/1743)
- Validate that required arguments are provided in @requires selections (https://github.com/grafbase/grafbase/pull/1683)
- More context for error messages regarding `@requires` fields validations (https://github.com/grafbase/grafbase/pull/1683)
- Subgraph names must now start with an alphabetic character and be entirely alphanumeric characters and hyphens (https://github.com/grafbase/grafbase/pull/1685)
- Two subgraphs that only differ by name case are not allowed to be composed anymore (https://github.com/grafbase/grafbase/pull/1685)

## 0.3.0 - 2024-02-06

- Implement non-default root type support (#1154)
- Emit multiple `@field`s for shareable fields of entities (#1255)
- Restore broken assumption about interned strings (#1158)

## 0.2.0 - 2023-12-14

This is the first version that comes close to complete support for the spec.
The test suite has been expanded. It is used in production by [Grafbase
Federated Graphs](https://grafbase.com/changelog/federated-graphs) and [Schema
Checks](https://grafbase.com/changelog/schema-checks).
