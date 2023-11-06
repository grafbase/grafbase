# grafbase-validation

This crate implements GraphQL SDL schema validation according to the [2021
version of the GraphQL spec](http://spec.graphql.org/October2021/).

Scope:

- All the spec and nothing but the spec.
- Query documents are out of scope.
- The error messages should be as close as possible to the style of other
  GraphQL schema validation libraries.
