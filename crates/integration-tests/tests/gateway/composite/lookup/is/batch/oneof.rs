use integration_tests::{gateway::Gateway, runtime};

use super::super::super::{EchoArgs, gql_id, gql2_name};

#[test]
fn basic() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_id())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])
                    @init

                type Query {
                    productBatch(input: Lookup! @is(field: "{ id: [id] }")): [Product!]! @lookup
                }

                input Lookup @oneOf {
                    id: [ID!]
                }

                type Product @key(fields: "id") {
                    id: ID!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoArgs)
            .build()
            .await;

        let response = engine.post("query { products { id args } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "id": "1",
                "args": {
                  "input": {
                    "id": [
                      "1"
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
fn multiple_keys() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_id())
            .with_subgraph(gql2_name())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])
                    @init

                type Query {
                    productBatch(input: Lookup! @is(field: "{ id: [id] } | { name: [name] }")): [Product!]! @lookup
                }

                input Lookup @oneOf {
                    id: [ID!]
                    name: [String!]
                }

                type Product @key(fields: "id") @key(fields: "name") {
                    id: ID!
                    name: String!
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
                    "id": [
                      "1"
                    ]
                  }
                }
              }
            ]
          }
        }
        "#);

        let response = engine.post("query { products2 { args } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products2": [
              {
                "args": {
                  "input": {
                    "name": [
                      "name1"
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
            .with_subgraph(gql_id())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])
                    @init

                type Query {
                    productBatch(input: Lookup @is(field: "{ id: [id] }")): [Product!]! @lookup
                }

                input Lookup @oneOf {
                    id: [ID!]
                }

                type Product @key(fields: "id") {
                    id: ID!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoArgs)
            .build()
            .await;

        let response = engine.post("query { products { id args } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "id": "1",
                "args": {
                  "input": {
                    "id": [
                      "1"
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
            .with_subgraph(gql_id())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])
                    @init

                type Query {
                    productBatch(input: Lookup! @is(field: "{ id: [id] }")): [Product!]! @lookup
                }

                input Lookup @oneOf {
                    id: [ID]
                }

                type Product @key(fields: "id") {
                    id: ID!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoArgs)
            .build()
            .await;

        let response = engine.post("query { products { id args } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "id": "1",
                "args": {
                  "input": {
                    "id": [
                      "1"
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
fn arg_with_same_name_and_extra_input_field_with_matching_type() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_id())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])
                    @init

                type Query {
                    productBatch(input: Lookup! @is(field: "{ id: [id] }")): [Product!]! @lookup
                }

                input Lookup @oneOf {
                    id: [ID!]
                    anything: [ID!]
                }

                type Product @key(fields: "id") {
                    id: ID!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoArgs)
            .build()
            .await;

        let response = engine.post("query { products { id args } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "id": "1",
                "args": {
                  "input": {
                    "id": [
                      "1"
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
            .with_subgraph(gql_id())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])
                    @init

                type Query {
                    productBatch(input: Lookup! @is(field: "{ id: [id] }")): [Product!]! @lookup
                }

                input Lookup @oneOf {
                    ids: [ID!]
                }

                type Product @key(fields: "id") {
                    id: ID!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoArgs)
            .build()
            .await;

        let response = engine.post("query { products { id args } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "id": "1",
                "args": {
                  "input": {
                    "ids": [
                      "1"
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
fn other_input_field() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_id())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])
                    @init

                type Query {
                    productBatch(input: Lookup! @is(field: "{ id: [id] }")): [Product!]! @lookup
                }

                input Lookup @oneOf {
                    ids: [ID!]
                    id: ID
                }

                type Product @key(fields: "id") {
                    id: ID!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoArgs)
            .build()
            .await;

        let response = engine.post("query { products { id args } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "id": "1",
                "args": {
                  "input": {
                    "ids": [
                      "1"
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
            .with_subgraph(gql_id())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])
                    @init

                type Query {
                    productBatch(id: Lookup! @is(field: "{ id: id }")): [Product!]! @lookup
                }

                input Lookup @oneOf {
                    id: ID
                }

                type Product @key(fields: "id") {
                    id: ID!
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
        See schema at 29:3:
        productBatch(id: Lookup!): [Product!]! @composite__lookup(graph: EXT) @join__field(graph: EXT)
        "#);
    })
}

#[test]
fn multiple_injections_oneof() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph(gql_id())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])
                    @init

                type Query {
                    productBatch(a: Lookup!  @is(field: "{ id: id }"), b: Lookup!  @is(field: "{ id: id }")): [Product!]! @lookup
                }

                input Lookup @oneOf {
                    id: [ID!]
                }

                type Product @key(fields: "id") {
                    id: ID!
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
        See schema at 29:3:
        productBatch(a: Lookup!, b: Lookup!): [Product!]! @composite__lookup(graph: EXT) @join__field(graph: EXT)
        "#);
    })
}

#[test]
fn multiple_oneof_single_is() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph(gql_id())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])
                    @init

                type Query {
                    productBatch(a: Lookup! @is(field: "{ id: id }"), b: Lookup!): [Product!]! @lookup
                }

                input Lookup @oneOf {
                    id: [ID!]
                }

                type Product @key(fields: "id") {
                    id: ID!
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
        See schema at 29:3:
        productBatch(a: Lookup!, b: Lookup!): [Product!]! @composite__lookup(graph: EXT) @join__field(graph: EXT)
        "#);
    })
}

#[test]
fn extra_required_argument() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph(gql_id())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])
                    @init

                type Query {
                    productBatch(lookup: Lookup! @is(field: "{ id: id }"), required: Boolean!): [Product!]! @lookup
                }

                input Lookup @oneOf {
                    id: ID
                }

                type Product @key(fields: "id") {
                    id: ID!
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
        See schema at 29:3:
        productBatch(lookup: Lookup!, required: Boolean!): [Product!]! @composite__lookup(graph: EXT) @join__field(graph: EXT)
        "#);
    })
}
