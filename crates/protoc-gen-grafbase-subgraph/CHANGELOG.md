## 0.2.0 - 2025-07-24

### Added

- **GraphQL Directive Support**: Added support for all directive options defined in `options.proto`:
  - `object_directives` and `input_object_directives` for object-level directives
  - `input_field_directives` and `output_field_directives` for field-level directives
  - `enum_directives` for enum-level directives
  - `enum_value_directives` for enum value-level directives
- **Query Field Mapping**: Added support for mapping gRPC service methods to GraphQL Query fields instead of Mutations:
  - `is_graphql_query_field` and `is_graphql_mutation_field` options on individual methods
  - `graphql_default_to_query_fields` and `graphql_default_to_mutation_fields` options on services to make all methods default to Query (or Mutation) fields
- **Query Type Generation**: The generator now creates a `type Query` in addition to `type Mutation` and `type Subscription` based on method configurations

## 0.1.0 - 2025-04-15

- Initial release. The output matches the directives expected by version 0.1.0 of the grpc extension.
