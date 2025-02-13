# Changelog

## Unreleased

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
