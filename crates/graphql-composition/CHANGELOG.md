# Changelog

## Unreleased

### Improvements

- Implemented support for the `@internal` directive from the composite schemas spec (https://github.com/grafbase/grafbase/pull/3185)

## 0.8.0 - 2025-05-16

### Breaking changes

- The import url for the composite schemas spec changed from "https://specs.grafbase.com/composite-schema/v1" to "https://specs.grafbase.com/composite-schemas/v1"

## 0.7.3 - 2025-04-30

### Improvements

- Composition now allows federated graphs without a query root. That happens if none of the subgraphs define a query root type. The new, relaxed requirement is that at least one root field (query, mutation or subscription) is defined, so the federated schema is not empty. (https://github.com/grafbase/grafbase/pull/3111)
- Fixed a few inaccurate directive definition locations in the federated SDL emitted by render_federated_sdl().

### Fixes

- The selection sets of `@key` directives are now validated, instead of leading to a runtime panic when they reference fields that do not exist.

## 0.7.2 - 2025-04-28

- Corrected a bug that made directive imports with `@link(imports:)` sometimes exceed the scope of their subgraph.

## 0.7.1 - 2025-04-28

### Changes

- Add `@oneOf` support
- Limited native composite schemas spec support

### Fixes

- Corrected the definition of `@join__type` in federated SDL to include the `isInterfaceObject` and `extension` arguments, as per the spec.

## 0.7.0 - 2025-04-25

### Changes

- GraphQL SDL is now rendered with two spaces indentation (previously four spaces).

### Fixes

- Do not warn on `@specifiedBy` directive when it is not imported. It is a GraphQL built-in. (https://github.com/grafbase/grafbase/pull/2673)
- Fixed directives from extensions on enum definitions sometimes not being rendered in the federated graph. (https://github.com/grafbase/grafbase/pull/3073)

## 0.6.1 - 2025-02-13

### Features

- Unknown directives are now reported as warnings (https://github.com/grafbase/grafbase/pull/2618).

### Fixes

- Make the `url` argument optional in the definition of the `@join__graph` directive, to reflect the optionality of url introduced for virtual subgraphs in https://github.com/grafbase/grafbase/pull/2589.
- (BREAKING) Take the `@` prefix into account in aliased imports from `@link` directives. Previously, the `as:` argument for individual directives would only work if you did not prefix the alias with an `@`. But the link spec is clear: if you import a directive, you have to import it with an `@` prefix, and other types should be imported without. This behaviour is fixed, and the presence or absence of `@` is now enforced.

## 0.6.0 - 2025-02-10

### Features

- Implemented the rule that any directive composed with `@composeDirective` must have the same definition in all subgraphs. (https://github.com/grafbase/grafbase/pull/2532)
- The definitions for directives composed with `@composeDirective` are now included in the composed federated graph. (https://github.com/grafbase/grafbase/pull/2539)
- The `url` argument of `Subgraph::ingest()` is now optional, to support virtual subgraphs that have no associated url. (https://github.com/grafbase/grafbase/pull/2589)

### Fixes

- Descriptions of field arguments was not implemented. They are now properly included in the federated graph. (https://github.com/grafbase/grafbase/pull/2544)

## 0.5.0 - 2025-01-06

### Features

- Added composition for default values of output field arguments and input fields. They are now reflected in the federated graph.
- Selection sets inside `@requires` and `@provides` directives can now include inline fragments.
- The selection sets in the "fields:" argument on `@requires` are now validated against the schema, with proper errors with context when invalid.
- A new `Subgraphs::ingest_str()` method has been added to ingest a federated graph from a string, instead of from an async_graphql_parser AST. This is both for convenience and because async_graphql parser will soon not be part of the public API anymore.
- There is no longer a `VersionedFederatedGraph`. The serializable version of the federated graph is dropped — that role will be fulfilled by the federated SDL instead (https://github.com/grafbase/grafbase/pull/2310).
- `graphql_composition` now reexports the companion `graphql_federated_graph` crate (https://github.com/grafbase/grafbase/pull/2310).
- Added support for the `@cost` directive (https://github.com/grafbase/grafbase/pull/2305).
- We now validate that subgraphs do not define the `join__Graph` enum. (https://github.com/grafbase/grafbase/pull/2325)
- Support the experimental `@authorized` directive

### Fixes

- Fixed the ingestion of numeric literals when creating a federated graph from a string.
- Fixed the ingestion of `null` literals.
- In federated_graph, when parsing a schema with `@join__type` and no key argument, then rendering it with `render_federated_sdl()` would produce a `@join__type` directive
- Fix rendering of object literals in render_federated_sdl(). We were erroneously quoting the keys, like in a JSON object, that is {"a": true} instead of the correct GraphQL literal {a: true}. (https://github.com/grafbase/grafbase/pull/2247)
- Fixed double rendering of `@authorized` on fields in federated SDL when `@composeDirective(name: "@authorized")` is used. (https://github.com/grafbase/grafbase/pull/2251)

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
