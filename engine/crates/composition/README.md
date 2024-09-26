# graphql-composition

[![crates.io](https://img.shields.io/crates/v/graphql-composition)](https://crates.io/crates/graphql-composition)
[![docs.rs](https://img.shields.io/docsrs/graphql-composition)](https://docs.rs/graphql-composition/)

An implementation of [GraphQL federated schema composition](https://www.apollographql.com/docs/federation/federated-types/composition/).

## Example

```rust
use graphql_composition::{Subgraphs, compose};

let user_subgraph = r#"
  extend schema
    @link(url: "https://specs.apollo.dev/federation/v2.3",
          import: ["@key"])

  type Query {
    findUserByEmail(email: String!): User
  }

  type User @key(fields: "id") {
    id: ID!
    name: String!
  }
"#;

let cart_subgraph = r#"
  extend schema
    @link(url: "https://specs.apollo.dev/federation/v2.3",
          import: ["@key", "@shareable"])

  type User @key(fields: "id") {
    id: ID!
    cart: Cart
  }

  type Cart @shareable {
    items: [String!]!
  }
"#;

let [user_subgraph, cart_subgraph] = [user_subgraph, cart_subgraph]
  .map(|sdl| async_graphql_parser::parse_schema(&sdl).unwrap());

let mut subgraphs = Subgraphs::default();

subgraphs.ingest(&user_subgraph, "users-service", "http://users.example.com");
subgraphs.ingest(&cart_subgraph, "carts-service", "http://carts.example.com");

let composed = compose(&subgraphs).into_result().unwrap().into_federated_sdl();

let expected = r#"
directive @core(feature: String!) repeatable on SCHEMA

directive @join__owner(graph: join__Graph!) on OBJECT

directive @join__type(
    graph: join__Graph!
    key: String!
    resolvable: Boolean = true
) repeatable on OBJECT | INTERFACE

directive @join__field(
    graph: join__Graph
    requires: String
    provides: String
) on FIELD_DEFINITION

directive @join__graph(name: String!, url: String!) on ENUM_VALUE

directive @join__implements(graph: join__Graph!, interface: String!) repeatable on OBJECT | INTERFACE

enum join__Graph {
    USERS_SERVICE @join__graph(name: "users-service", url: "http://users.example.com")
    CARTS_SERVICE @join__graph(name: "carts-service", url: "http://carts.example.com")
}

type User
    @join__type(graph: USERS_SERVICE, key: "id")
    @join__type(graph: CARTS_SERVICE, key: "id")
{
    cart: Cart @join__field(graph: CARTS_SERVICE)
    id: ID!
    name: String! @join__field(graph: USERS_SERVICE)
}

type Cart {
    items: [String!]!
}

type Query {
    findUserByEmail(email: String!): User @join__field(graph: USERS_SERVICE)
}
  "#;

assert_eq!(expected.trim(), composed.trim());

```

## Features

- The output is a structured, serializable format with all relevant relationships (FederatedGraph).
- FederatedGraph can be rendered to GraphQL SDL (see the example above).
- Good composition error messages.

## Status

The crate is being actively developed and maintained.
