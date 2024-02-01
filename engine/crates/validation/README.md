# graphql-schema-validation

[![docs.rs](https://img.shields.io/docsrs/graphql-schema-validation)](https://docs.rs/graphql-schema-validation)

This crate implements GraphQL SDL schema validation according to the [2021
version of the GraphQL spec](http://spec.graphql.org/October2021/).

Scope:

- All the spec and nothing but the spec.
- Query documents are out of scope, we only validate schemas.
- The error messages should be as close as possible to the style of other
  GraphQL schema validation libraries.

## Example

```rust
use graphql_schema_validation::validate;

fn main() {
  let graphql = "schema { query: MyQueryDoesNotExist }";

  let diagnostics = validate(graphql);

  assert!(diagnostics.has_errors());

  let formatted_diagnostics = diagnostics.iter().map(|err| format!("{}", err)).collect::<Vec<String>>();
  assert_eq!(formatted_diagnostics, ["Cannot set schema query root to unknown type `MyQueryDoesNotExist`"]);
}
```

## Status

The crate is being actively developed and maintained. It covers the spec
completely and faithfully, as far as we are aware.
