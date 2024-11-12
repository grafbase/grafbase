# `graphql-lint`

A Rust based linter for GraphQL SDL schemas.

`graphql-lint` is used in the [Grafbase](https://grafbase.com) Platform and CLI.

## Currently Supported Lints

- Naming conventions
  - Types: `PascalCase`
    - Forbidden prefixes: `"Type"`
    - Forbidden suffixes: `"Type"`
  - Fields: `camelCase`
  - Input values: `camelCase`
  - Arguments: `camelCase`
  - Directives: `camelCase`
  - Enums: `PascalCase`
    - Forbidden prefixes: `"Enum"`
    - Forbidden suffixes: `"Enum"`
  - Unions
    - Forbidden prefixes: `"Union"`
    - Forbidden suffixes: `"Union"`
  - Enum values: `SCREAMING_SNAKE_CASE`
  - Interfaces
    - Forbidden prefixes: `"Interface"`
    - Forbidden suffixes: `"Interface"`
  - Query fields
    - Forbidden prefixes: `["query", "get", "list"]`
    - Forbidden suffixes: `"Query"`
  - Mutation fields
    - Forbidden prefixes: `["mutation", "put", "post", "patch"]`
    - Forbidden suffixes: `"Mutation"`
  - Subscription fields
    - Forbidden prefixes: `"subscription"`
    - Forbidden suffixes: `"Subscription"`
- Usage of the `@deprecated` directive requires specifying the `reason` argument

## Usage

```toml
[dependencies]
graphql-lint = "0.1.3"
```

```rust
use graphql_lint::lint;

fn main () {
    let schema = r#"
        type Query {
          hello: String!
        }
    "#;

    let violations = lint(schema).unwrap();
}
```
