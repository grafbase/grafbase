mod composite;
mod nested;
mod oneof;
mod oneof_composite;

use integration_tests::{gateway::Gateway, runtime};

use super::super::{EchoArgs, gql_id};

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
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])
                    @init

                type Query {
                    productBatch(a: [ID!]! @is(field: "[id]"), b: [ID!]): [Product!]! @lookup
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
                  "a": [
                    "1"
                  ]
                }
              }
            ]
          }
        }
        "#);
    })
}

#[test]
fn arg_type_compatibility_nullable_list() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_id())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])
                    @init

                type Query {
                    productBatch(a: [ID!] @is(field: "[id]"), b: [ID!]): [Product!]! @lookup
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
                  "a": [
                    "1"
                  ]
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
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])
                    @init

                type Query {
                    productBatch(a: [ID]! @is(field: "[id]"), b: [ID!]): [Product!]! @lookup
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
                  "a": [
                    "1"
                  ]
                }
              }
            ]
          }
        }
        "#);
    })
}

#[test]
fn arg_type_compatibility_inner_and_list_nullable() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_id())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])
                    @init

                type Query {
                    productBatch(a: [ID] @is(field: "[id]"), b: [ID!]): [Product!]! @lookup
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
                  "a": [
                    "1"
                  ]
                }
              }
            ]
          }
        }
        "#);
    })
}

#[test]
fn arg_with_default_value() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_id())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])
                    @init

                type Query {
                    productBatch(a: [ID]! @is(field: "[id]"), b: [ID!], extra: Boolean! = true): [Product!]! @lookup
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

        let response = engine.post("query { products { args } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "args": {
                  "a": [
                    "1"
                  ],
                  "extra": true
                }
              }
            ]
          }
        }
        "#);
    })
}

#[test]
fn arg_with_default_value_coercion() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_id())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])
                    @init

                type Query {
                    productBatch(a: [ID]! @is(field: "[id]"), b: [ID!], extra: [Boolean!]! = true): [Product!]! @lookup
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

        let response = engine.post("query { products { args } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "args": {
                  "a": [
                    "1"
                  ],
                  "extra": [
                    true
                  ]
                }
              }
            ]
          }
        }
        "#);
    })
}

#[test]
fn multiple_injections() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_id())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])
                    @init

                type Query {
                    productBatch(a: [ID!] @is(field: "[id]"), b: [ID!] @is(field: "[id]")): [Product!]! @lookup
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

        let response = engine.post("query { products { args } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "args": {
                  "a": [
                    "1"
                  ],
                  "b": [
                    "1"
                  ]
                }
              }
            ]
          }
        }
        "#);
    })
}

#[test]
fn no_matching_key() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph(gql_id())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])
                    @init

                type Query {
                    productBatch(something: [JSON] @is(field: "[args]")): [Product!]! @lookup
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
        See schema at 33:3:
        productBatch(something: [JSON] @composite__is(graph: EXT, field: "[args]")): [Product!]! @composite__lookup(graph: EXT) @join__field(graph: EXT)
        "#);
    })
}

#[test]
fn cannot_inject_nullable_into_required() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph(gql_id())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])
                    @init

                type Query {
                    productBatch(id: [ID!] @is(field: "[id]")): [Product!]! @lookup
                }

                type Product @key(fields: "id") {
                    id: ID
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoArgs)
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Query.productBatch, for directive @lookup for associated @is directive: Incompatible wrapping, cannot map Product.id (ID) into Query.productBatch.id (ID!)
        See schema at 33:40:
        (graph: EXT, field: "[id]")
        "#);
    })
}

#[test]
fn bad_type() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph(gql_id())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])
                    @init

                type Query {
                    productBatch(id: [Int] @is(field: "[id]")): [Product!]! @lookup
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
        At site Query.productBatch, for directive @lookup for associated @is directive: Cannot map Product.id (ID!) into Query.productBatch.id (Int)
        See schema at 33:40:
        (graph: EXT, field: "[id]")
        "#);
    })
}

#[test]
fn not_a_list() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph(gql_id())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])
                    @init

                type Query {
                    productBatch(id: ID! @is(field: "id")): [Product!]! @lookup
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
        At site Query.productBatch, for directive @lookup for associated @is directive: Cannot select a field from [Product!]!, it's a list
        See schema at 33:38:
        (graph: EXT, field: "id")
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
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])
                    @init

                type Query {
                    productBatch(ids: [ID!] @is(field: "[id]"), required: Boolean!): [Product!]! @lookup
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
        At site Query.productBatch, for directive @lookup Argument 'required' is required but is not injected any @is directive.
        See schema at 33:3:
        productBatch(ids: [ID!] @composite__is(graph: EXT, field: "[id]"), required: Boolean!): [Product!]! @composite__lookup(graph: EXT) @join__field(graph: EXT)
        "#);
    })
}
