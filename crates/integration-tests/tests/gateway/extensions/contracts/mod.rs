use integration_tests::{gateway::Gateway, runtime};

use crate::gateway::introspection::{PATHFINDER_INTROSPECTION_QUERY, introspection_to_sdl};

const USER_SDL: &str = r#"
extend schema @link(url: "https://specs.apollo.dev/federation/v2.3", import: ["@key","@shareable"])
extend schema @link(url: "contracts-19-0.1.0", import: ["@tag"])

type User implements Person @key(fields: "id") @tag(name: "public") {
  id: ID! @tag(name: "public")
  name: String! @tag(name: "public")
  email: String! @tag(name: "internal")
  points: Int @tag(name: "partner")
  beta: String @tag(name: "beta")
}

interface Person @tag(name: "public") {
  id: ID! @tag(name: "public")
  name: String! @tag(name: "public")
}

enum Role @tag(name: "internal") {
  ADMIN @tag(name: "internal")
  USER @tag(name: "public")
}

union Account @tag(name: "public") = User | Admin

type Admin @tag(name: "internal") {
  id: ID! @tag(name: "internal")
  permissions: [String!]! @tag(name: "internal")
}

input UserInput @tag(name: "public") {
  name: String! @tag(name: "public")
  email: String @tag(name: "internal")
}

scalar DateTime @tag(name: "public")

type Query {
  user(id: ID!): User @tag(name: "public")
  users: [User!]! @tag(name: "internal")
  betaUsers: [User!]! @tag(name: "beta")
}

type Mutation {
  createUser(input: UserInput!): User @tag(name: "public")
  deleteUser(id: ID!): Boolean! @tag(name: "internal")
}
"#;

const PRODUCT_SDL: &str = r#"
extend schema @link(url: "https://specs.apollo.dev/federation/v2.3", import: ["@key","@shareable"])
extend schema @link(url: "contracts-19-0.1.0", import: ["@tag"])

type Product implements Item @key(fields: "id") @tag(name: "public") {
  id: ID! @tag(name: "public")
  name: String! @tag(name: "public")
  price: Float! @tag(name: "public")
  cost: Float! @tag(name: "internal")
  wholesale: Float @tag(name: "partner")
  aiPrice: Float @tag(name: "beta")
}

interface Item @tag(name: "public") {
  id: ID! @tag(name: "public")
  price: Float! @tag(name: "public")
}

enum Status @tag(name: "public") {
  ACTIVE @tag(name: "public")
  HIDDEN @tag(name: "internal")
}

union ProductType @tag(name: "public") = Product | Service

type Service @tag(name: "partner") {
  id: ID! @tag(name: "partner")
  duration: Int! @tag(name: "partner")
}

input ProductInput @tag(name: "internal") {
  name: String! @tag(name: "public")
  price: Float! @tag(name: "public")
  cost: Float! @tag(name: "internal")
}

scalar JSON @tag(name: "beta")

extend type User @key(fields: "id") {
  id: ID!
  purchases: [Product!]! @tag(name: "partner")
}

type Query {
  product(id: ID!): Product @tag(name: "public")
  allProducts: [Product!]! @tag(name: "internal")
  recommendations: [Product!]! @tag(name: "beta")
}

type Mutation {
  createProduct(input: ProductInput!): Product @tag(name: "internal")
  updatePrice(id: ID!, price: Float!): Product @tag(name: "public")
}
"#;

#[test]
fn no_contract() {
    runtime().block_on(async move {
        let gateway = Gateway::builder()
            .with_subgraph_sdl("user", USER_SDL)
            .with_subgraph_sdl("product", PRODUCT_SDL)
            .with_extension("contracts-19")
            .with_toml_config(
                r#"
                [graph]
                introspection = true
                "#,
            )
            .build()
            .await;

        let response = gateway.post(PATHFINDER_INTROSPECTION_QUERY).await;
        insta::assert_snapshot!(introspection_to_sdl(response.into_data()), @r#"
        union Account = User | Admin

        type Admin {
          id: ID!
          permissions: [String!]!
        }

        scalar DateTime

        interface Item {
          id: ID!
          price: Float!
        }

        scalar JSON

        type Mutation {
          createProduct(input: ProductInput!): Product
          createUser(input: UserInput!): User
          deleteUser(id: ID!): Boolean!
          updatePrice(id: ID!, price: Float!): Product
        }

        interface Person {
          id: ID!
          name: String!
        }

        type Product implements Item {
          aiPrice: Float
          cost: Float!
          id: ID!
          name: String!
          price: Float!
          wholesale: Float
        }

        input ProductInput {
          name: String!
          price: Float!
          cost: Float!
        }

        union ProductType = Product | Service

        type Query {
          allProducts: [Product!]!
          betaUsers: [User!]!
          product(id: ID!): Product
          recommendations: [Product!]!
          user(id: ID!): User
          users: [User!]!
        }

        enum Role {
          ADMIN
          USER
        }

        type Service {
          duration: Int!
          id: ID!
        }

        enum Status {
          ACTIVE
          HIDDEN
        }

        type User implements Person {
          beta: String
          email: String!
          id: ID!
          name: String!
          points: Int
          purchases: [Product!]!
        }

        input UserInput {
          name: String!
          email: String
        }
        "#);
    });
}

#[test]
fn included_tags() {
    runtime().block_on(async move {
        let gateway = Gateway::builder()
            .with_subgraph_sdl("user", USER_SDL)
            .with_subgraph_sdl("product", PRODUCT_SDL)
            .with_extension("contracts-19")
            .with_toml_config(
                r#"
                [graph]
                introspection = true
                contract_key = "{\"includedTags\":[\"public\"]}"
                "#,
            )
            .build()
            .await;

        let response = gateway.post(PATHFINDER_INTROSPECTION_QUERY).await;
        insta::assert_snapshot!(introspection_to_sdl(response.into_data()), @r#"
        union Account = User

        scalar DateTime

        interface Item {
          id: ID!
          price: Float!
        }

        type Mutation {
          createUser(input: UserInput!): User
          updatePrice(id: ID!, price: Float!): Product
        }

        interface Person {
          id: ID!
          name: String!
        }

        type Product implements Item {
          id: ID!
          name: String!
          price: Float!
        }

        input ProductInput {
          name: String!
          price: Float!
        }

        union ProductType = Product

        type Query {
          product(id: ID!): Product
          user(id: ID!): User
        }

        enum Role {
          USER
        }

        enum Status {
          ACTIVE
        }

        type User implements Person {
          id: ID!
          name: String!
        }

        input UserInput {
          name: String!
        }
        "#);
    });
}

#[test]
fn hooks_contract() {
    runtime().block_on(async move {
        let gateway = Gateway::builder()
            .with_subgraph_sdl("user", USER_SDL)
            .with_subgraph_sdl("product", PRODUCT_SDL)
            .with_extension("contracts-19")
            .with_extension("hooks-19")
            .with_toml_config(
                r#"
                [graph]
                introspection = true
                "#,
            )
            .build()
            .await;

        let response = gateway
            .post(PATHFINDER_INTROSPECTION_QUERY)
            .header("contract-key", r#"{"includedTags":["public"]}"#)
            .await;
        insta::assert_snapshot!(introspection_to_sdl(response.into_data()), @r#"
        union Account = User

        scalar DateTime

        interface Item {
          id: ID!
          price: Float!
        }

        type Mutation {
          createUser(input: UserInput!): User
          updatePrice(id: ID!, price: Float!): Product
        }

        interface Person {
          id: ID!
          name: String!
        }

        type Product implements Item {
          id: ID!
          name: String!
          price: Float!
        }

        input ProductInput {
          name: String!
          price: Float!
        }

        union ProductType = Product

        type Query {
          product(id: ID!): Product
          user(id: ID!): User
        }

        enum Role {
          USER
        }

        enum Status {
          ACTIVE
        }

        type User implements Person {
          id: ID!
          name: String!
        }

        input UserInput {
          name: String!
        }
        "#);
    });
}

#[test]
fn excluded_tags() {
    runtime().block_on(async move {
        let gateway = Gateway::builder()
            .with_subgraph_sdl("user", USER_SDL)
            .with_subgraph_sdl("product", PRODUCT_SDL)
            .with_extension("contracts-19")
            .with_toml_config(
                r#"
                [graph]
                introspection = true
                contract_key = "{\"excludedTags\":[\"internal\"]}"
                "#,
            )
            .build()
            .await;

        let response = gateway.post(PATHFINDER_INTROSPECTION_QUERY).await;
        insta::assert_snapshot!(introspection_to_sdl(response.into_data()), @"");
    });
}
