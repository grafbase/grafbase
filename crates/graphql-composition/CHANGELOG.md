# Changelog

## Unreleased

## 0.11.0 - 2025-08-27

### Improvements

- Significant performance improvements. We saw a 66% improvement on a very large federated graph, from various optimizations, including iterating over flat data structures instead of BTrees.
- `Diagnostic::composite_schemas_error_code` is now exposed. Note that as the spec and this implementation evolve, more error codes will be added to this enum and its variants.
- Implemented some composite schemas spec validations for the `@override` directive, resulting in more consistent logic and better diagnostics (https://github.com/grafbase/grafbase/pull/3374)
- Implemented the composite schemas spec validation rules for `@shareable`. It makes the validation more relaxed: if any subgraph defines the field as shareable, then others don't need to annotate with `@shareable` as well. It should not make any schema that composes today fail to compose. The diagnostics have also been improved to list all the relevant subgraphs.
- Better diagnostic message when you try to compose zero subgraphs.
- Unknown override labels are no longer an error.
- `@extends` no longer triggers unknown directive warnings.
- Instances of `@key(field: "...")` (instead of "fields") now triggers a warning.
- Selection set validation errors now include the subgraph name.
- We now emit the `@join__enumValue` directive on enum values in the federated SDL (https://github.com/grafbase/grafbase/pull/3456).

### Fixes

- Fixed a panic in validation for `@provides` on fields returning a built-in scalar type, like `Int`.
- Fixed `Diagnostics::iter_warnings()` iterating over warnings instead of errors.
- Interfaces both defined in federation v1 subgraphs and as entity interfaces in federation v2 subgraphs do not force the federation v1 subgraphs to define all implementers of the entity interface anymore.
- Federation v1 subgraphs are no longer taken into account when composing entity interfaces.
- Fixed handling of entities with type extensions, where in some cases `@key`s would be missed for scenarios where the same entity is extended, then defined in the same subgraph, with other definitions in between. (https://github.com/grafbase/grafbase/pull/3454)

## Breaking changes

- Many types, fields and methods that are part of the `FederatedGraph` data structure are now private. `FederatedGraph` as a whole will become private as well, since the scope of this crate is only to render the federated SDL.
- `compose()` now takes an `&mut Subgraphs` argument instead of `&Subgraphs`.

## 0.10.0 - 2025-07-30

### Improvements

- Implemented composite schemas spec validation to warn against non-nullable lookup fields. (https://github.com/grafbase/grafbase/pull/3295)
- The `diagnostics` module is now exposed, as well as the `Diagnostic` and `Severity` types, and the  `Diagnostics::iter()` method.

## 0.9.0 - 2025-06-19

### Improvements

- Implemented support for the `@internal` directive from the composite schemas spec (https://github.com/grafbase/grafbase/pull/3185)
- Implemented support for the `@require` directive from the composite schemas spec (https://github.com/grafbase/grafbase/pull/3189)
- Implemented validation that the root query type is not `@inaccessible`. (https://github.com/grafbase/grafbase/pull/3281)

## Breaking changes

- The graphql-federated-graph dependency, which was always an implementation detail, is now deprecated. Its functionality has been folded into graphql-composition, as a module. As a result, the crate does not re-export the `graphql_federated_graph` crate anymore, just a select few items. Most fields of `FederatedGraph` have also become private. Please get in touch if this is something you were relying on.
- The Grafbase-specific, non-standard `@authorized` directive is no longer getting special treatment (https://github.com/grafbase/grafbase/pull/3189).

## Fixes

- When an enum is used both in input and output positions, composition enforces that definitions of that enum in all subgraphs must have exactly the same values. In the case where one subgraph had more values than another, composition would do an out of bound access to test equality, and panic. It's order dependent, so it was not encountered in the existing tests. This commit fixes the out of bound errors, preserving the same logic, and improves the composition error message formulation. (https://github.com/grafbase/grafbase/pull/3292)
- Directives composed with `@composeDirective` no longer appear in the API SDL, only the federated SDL. API SDL matches the introspection output of the running gateway, which can only return `@deprecated` directives' contents as per the October 2021 version of the GraphQL specification. (https://github.com/grafbase/grafbase/pull/3319)

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
