use integration_tests::{gateway::Gateway, runtime};

use super::super::super::{EchoArgs, gql_ab, gql_ab_id_int};

#[test]
fn basic() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_ab())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])
                    @init

                type Query {
                    productBatch(input: Lookup! @is(field: "{ key: [{ a b }]}")): [Product!]! @lookup
                }

                input Lookup @oneOf {
                    key: [Key!]
                }

                input Key {
                    a: ID!
                    b: ID!
                }

                type Product @key(fields: "a b") {
                    a: ID!
                    b: ID!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoArgs)
            .build()
            .await;

        let response = engine.post("query { products { args } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "args": {
                  "input": {
                    "key": [
                      {
                        "a": "A1",
                        "b": "B1"
                      }
                    ]
                  }
                }
              }
            ]
          }
        }
        "#);
    })
}

#[test]
fn nullable_lookup() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_ab())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])
                    @init

                type Query {
                    productBatch(input: Lookup @is(field: "{ key: [{ a b }]}")): [Product!]! @lookup
                }

                input Lookup @oneOf {
                    key: [Key!]
                }

                input Key {
                    a: ID!
                    b: ID!
                }

                type Product @key(fields: "a b") {
                    a: ID!
                    b: ID!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoArgs)
            .build()
            .await;

        let response = engine.post("query { products { args } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "args": {
                  "input": {
                    "key": [
                      {
                        "a": "A1",
                        "b": "B1"
                      }
                    ]
                  }
                }
              }
            ]
          }
        }
        "#);
    })
}

#[test]
fn arg_type_compatibility_inner_nullable() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_ab())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])
                    @init

                type Query {
                    productBatch(input: Lookup! @is(field: "{ key: [{ a b }]}")): [Product!]! @lookup
                }

                input Lookup @oneOf {
                    key: [Key]
                }

                input Key {
                    a: ID!
                    b: ID!
                }

                type Product @key(fields: "a b") {
                    a: ID!
                    b: ID!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoArgs)
            .build()
            .await;

        let response = engine.post("query { products { args } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "args": {
                  "input": {
                    "key": [
                      {
                        "a": "A1",
                        "b": "B1"
                      }
                    ]
                  }
                }
              }
            ]
          }
        }
        "#);
    })
}

#[test]
fn arg_with_same_name_and_extra_input_field() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_ab())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])
                    @init

                type Query {
                    productBatch(input: Lookup! @is(field: "{ key: [{ a b }]}")): [Product!]! @lookup
                }

                input Lookup @oneOf {
                    key: [Key!]
                    anything: [ID!]
                }

                input Key {
                    a: ID!
                    b: ID!
                }

                type Product @key(fields: "a b") {
                    a: ID!
                    b: ID!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoArgs)
            .build()
            .await;

        let response = engine.post("query { products { args } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "args": {
                  "input": {
                    "key": [
                      {
                        "a": "A1",
                        "b": "B1"
                      }
                    ]
                  }
                }
              }
            ]
          }
        }
        "#);
    })
}

#[test]
fn arg_with_different_name() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_ab_id_int())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])
                    @init

                type Query {
                    productBatch(input: Lookup! @is(field: "{ key: [{ c: a d: b }]}")): [Product!]! @lookup
                }

                input Lookup @oneOf {
                    key: [Key!]
                }

                input Key {
                    c: ID!
                    d: Int!
                }

                type Product @key(fields: "a b") {
                    a: ID!
                    b: Int!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoArgs)
            .build()
            .await;

        let response = engine.post("query { products { args } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "args": {
                  "input": {
                    "key": [
                      {
                        "c": "A1",
                        "d": 1
                      }
                    ]
                  }
                }
              }
            ]
          }
        }
        "#);
    })
}

#[test]
fn invalid_batch() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph(gql_ab_id_int())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])
                    @init

                type Query {
                    productBatch(input: Lookup! @is(field: "{ key: { a b } }")): [Product!]! @lookup
                }

                input Lookup @oneOf {
                    key: Key
                }

                input Key {
                    a: ID!
                    b: Int!
                }

                type Product @key(fields: "a b") {
                    a: ID!
                    b: Int!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoArgs)
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Query.productBatch, for directive @lookup no matching @key directive was found
        See schema at 30:3:
        productBatch(input: Lookup!): [Product!]! @composite__lookup(graph: EXT) @join__field(graph: EXT)
        "#);
    })
}

#[test]
fn extra_required_argument() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph(gql_ab_id_int())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])
                    @init

                type Query {
                    productBatch(lookup: Lookup! @is(field: "{ key: [{ a b }]}"), required: Boolean!): [Product!]! @lookup
                }

                input Lookup @oneOf {
                    key: [Key!]
                }

                input Key {
                    a: ID!
                    b: Int!
                }

                type Product @key(fields: "a b") {
                    a: ID!
                    b: Int!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoArgs)
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Query.productBatch, for directive @lookup no matching @key directive was found
        See schema at 30:3:
        productBatch(lookup: Lookup!, required: Boolean!): [Product!]! @composite__lookup(graph: EXT) @join__field(graph: EXT)
        "#);
    })
}

#[test]
fn extra_required_field() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph(gql_ab_id_int())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])
                    @init

                type Query {
                    productBatch(lookup: Lookup! @is(field: "{ key: [{ a b }]}")): [Product!]! @lookup
                }

                input Lookup @oneOf {
                    key: [Key!]
                }

                input Key {
                    a: ID!
                    c: Int!
                    x: Boolean!
                }

                type Product @key(fields: "a b") {
                    a: ID!
                    b: Int!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoArgs)
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Query.productBatch, for directive @lookup no matching @key directive was found
        See schema at 30:3:
        productBatch(lookup: Lookup!): [Product!]! @composite__lookup(graph: EXT) @join__field(graph: EXT)
        "#);
    })
}
