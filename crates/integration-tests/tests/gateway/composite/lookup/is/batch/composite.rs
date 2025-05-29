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
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])
                    @init

                type Query {
                    productBatch(key: [Key!]! @is(field: "[{ a b }]")): [Product!]! @lookup
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
                  "key": [
                    {
                      "a": "A1",
                      "b": "B1"
                    }
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
fn renames() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_ab())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])
                    @init

                type Query {
                    productBatch(key: [Key!]! @is(field: "[{ aa: a bb: b }]")): [Product!]! @lookup
                }

                input Key {
                    aa: ID!
                    bb: ID!
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
                  "key": [
                    {
                      "aa": "A1",
                      "bb": "B1"
                    }
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
            .with_subgraph(gql_ab())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])
                    @init

                type Query {
                    productBatch(key: [Key!] @is(field: "[{ a b }]")): [Product!]! @lookup
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
                  "key": [
                    {
                      "a": "A1",
                      "b": "B1"
                    }
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
            .with_subgraph(gql_ab())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])
                    @init

                type Query {
                    productBatch(key: [Key]! @is(field: "[{ a b }]")): [Product!]! @lookup
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
                  "key": [
                    {
                      "a": "A1",
                      "b": "B1"
                    }
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
            .with_subgraph(gql_ab())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])
                    @init

                type Query {
                    productBatch(key: [Key] @is(field: "[{ a b }]")): [Product!]! @lookup
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
                  "key": [
                    {
                      "a": "A1",
                      "b": "B1"
                    }
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
fn arg_with_same_name_and_extra_optional_input_argument() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_ab())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])
                    @init

                type Query {
                    productBatch(key: [Key!]! @is(field: "[{ a b }]"), x: [ID!]): [Product!]! @lookup
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
                  "key": [
                    {
                      "a": "A1",
                      "b": "B1"
                    }
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
fn arg_with_same_name_and_extra_optional_input_field() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_ab())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])
                    @init

                type Query {
                    productBatch(key: [Key!]! @is(field: "[{ a b }]")): [Product!]! @lookup
                }

                input Key {
                    a: ID!
                    b: ID!
                    x: ID
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
                  "key": [
                    {
                      "a": "A1",
                      "b": "B1"
                    }
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
fn different_input_field_types() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_ab_id_int())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])
                    @init

                type Query {
                    productBatch(key: [Key!]! @is(field: "[{ a b }]")): [Product!]! @lookup
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
            .build()
            .await;

        let response = engine.post("query { products { args } }").await;
        insta::assert_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "args": {
                  "key": [
                    {
                      "a": "A1",
                      "b": 1
                    }
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
fn field_with_default_value() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_ab_id_int())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])
                    @init

                type Query {
                    productBatch(key: [Key!]! @is(field: "[{ a b }]")): [Product!]! @lookup
                }

                input Key {
                    a: ID!
                    b: Int!
                    extra: Boolean! = false
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
                  "key": [
                    {
                      "extra": false,
                      "a": "A1",
                      "b": 1
                    }
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
fn field_with_default_value_coercion() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_ab_id_int())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])
                    @init

                type Query {
                    productBatch(key: [Key!]! @is(field: "[{ a b }]")): [Product!]! @lookup
                }

                input Key {
                    a: ID!
                    b: Int!
                    extra: [Boolean!]! = false
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
                  "key": [
                    {
                      "extra": [
                        false
                      ],
                      "a": "A1",
                      "b": 1
                    }
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
fn no_arguments() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph(gql_ab())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])
                    @init

                type Query {
                    productBatch: [Product!]! @lookup
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
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Query.productBatch, for directive @lookup no matching @key directive was found
        See schema at 30:3:
        productBatch: [Product!]! @composite__lookup(graph: EXT) @join__field(graph: EXT)
        "#);
    })
}

#[test]
fn no_matching_argument() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph(gql_ab())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])
                    @init

                type Query {
                    productBatch(somethign: Int): [Product!]! @lookup
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
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Query.productBatch, for directive @lookup no matching @key directive was found
        See schema at 30:3:
        productBatch(somethign: Int): [Product!]! @composite__lookup(graph: EXT) @join__field(graph: EXT)
        "#);
    })
}

#[test]
fn cannot_inject_nullable_field_into_required() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph(gql_ab_id_int())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])
                    @init

                type Query {
                    productBatch(key: [Key!] @is(field: "[{ a b }]")): [Product!]! @lookup
                }

                input Key {
                    a: ID!
                    b: Int!
                }

                type Product @key(fields: "a b") {
                    a: ID!
                    b: Int
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoArgs)
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Query.productBatch, for directive @lookup for associated @is directive: Incompatible wrapping, cannot map Product.b (Int) into Key.b (Int!)
        See schema at 34:42:
        (graph: EXT, field: "[{ a b }]")
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
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])
                    @init

                type Query {
                    productBatch(key: Key! @is(field: "{ a b }")): [Product!]! @lookup
                }

                input Key {
                    a: ID!
                    b: Int!
                }

                type Product @key(fields: "a b") {
                    a: ID!
                    b: Int
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoArgs)
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Query.productBatch, for directive @lookup for associated @is directive: Cannot select object from [Product!]!, it's a list
        See schema at 34:40:
        (graph: EXT, field: "{ a b }")
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
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])
                    @init

                type Query {
                    productBatch(ids: [Key!], required: Boolean!): [Product!]! @lookup
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
        productBatch(ids: [Key!], required: Boolean!): [Product!]! @composite__lookup(graph: EXT) @join__field(graph: EXT)
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
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])
                    @init

                type Query {
                    productBatch(key: [Key!]): [Product!]! @lookup
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
        productBatch(key: [Key!]): [Product!]! @composite__lookup(graph: EXT) @join__field(graph: EXT)
        "#);
    })
}
