use integration_tests::{gateway::Gateway, runtime};

use super::super::super::{EchoArgs, gql_nested};

#[test]
fn object_input() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])
                    @init

                type Query {
                    productBatch(input: [DummyInput!]! @is(field: "[{ a: nested.id }]")): [Product!]! @lookup
                }

                input DummyInput {
                    a: ID!
                    b: ID
                }

                type Product @key(fields: "nested { id }") {
                    nested: Nested!
                    args: JSON
                }

                type Nested @shareable {
                    id: ID!
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
                  "nested": [
                    {
                      "id": "1"
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
fn list_input() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])
                    @init

                type Query {
                    productBatch(ids: [ID!]! @is(field: "[nested.id]")): [Product!]! @lookup
                }

                type Product @key(fields: "nested { id }") {
                    nested: Nested!
                    args: JSON
                }

                type Nested @shareable {
                    id: ID!
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
                  "nested": [
                    {
                      "id": "1"
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
fn object_input_arg_type_compatibility_nullable_list() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])
                    @init

                type Query {
                    productBatch(input: [DummyInput!] @is(field: "[{ a: nested.id }]")): [Product!]! @lookup
                }

                input DummyInput {
                    a: ID!
                    b: ID
                }

                type Product @key(fields: "nested { id }") {
                    nested: Nested!
                    args: JSON
                }

                type Nested @shareable {
                    id: ID!
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
                  "nested": [
                    {
                      "id": "1"
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
fn list_input_arg_type_compatibility_nullable_list() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])
                    @init

                type Query {
                    productBatch(ids: [ID!] @is(field: "[nested.id]")): [Product!]! @lookup
                }

                type Product @key(fields: "nested { id }") {
                    nested: Nested!
                    args: JSON
                }

                type Nested @shareable {
                    id: ID!
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
                  "nested": [
                    {
                      "id": "1"
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
fn object_input_arg_type_compatibility_inner_nullable() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])
                    @init

                type Query {
                    productBatch(input: [DummyInput]! @is(field: "[{ a: nested.id }]")): [Product!]! @lookup
                }

                input DummyInput {
                    a: ID!
                    b: ID
                }

                type Product @key(fields: "nested { id }") {
                    nested: Nested!
                    args: JSON
                }

                type Nested @shareable {
                    id: ID!
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
                  "nested": [
                    {
                      "id": "1"
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
fn list_input_arg_type_compatibility_inner_nullable() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])
                    @init

                type Query {
                    productBatch(ids: [ID!]! @is(field: "[nested.id]")): [Product!]! @lookup
                }

                type Product @key(fields: "nested { id }") {
                    nested: Nested!
                    args: JSON
                }

                type Nested @shareable {
                    id: ID!
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
                  "nested": [
                    {
                      "id": "1"
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
fn object_input_arg_type_compatibility_nested_input_field_nullable() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])
                    @init

                type Query {
                    productBatch(input: [DummyInput!]! @is(field: "[{ a: nested.id }]")): [Product!]! @lookup
                }

                input DummyInput {
                    a: ID
                    b: ID
                }

                type Product @key(fields: "nested { id }") {
                    nested: Nested!
                    args: JSON
                }

                type Nested @shareable {
                    id: ID!
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
                  "nested": [
                    {
                      "id": "1"
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
fn object_input_arg_type_compatibility_nested_field_nullable_with_nullable_input_field() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])
                    @init

                type Query {
                    productBatch(input: [DummyInput!]! @is(field: "[{ a: nested.id }]")): [Product!]! @lookup
                }

                input DummyInput {
                    a: ID
                    b: ID
                }

                type Product @key(fields: "nested { id }") {
                    nested: Nested!
                    args: JSON
                }

                type Nested @shareable {
                    id: ID
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
                  "nested": [
                    {
                      "id": "1"
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
fn object_input_arg_type_compatibility_nested_type_nullable_with_nullable_input_object() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])
                    @init

                type Query {
                    productBatch(input: [DummyInput]! @is(field: "[{ a: nested.id }]")): [Product!]! @lookup
                }

                input DummyInput {
                    a: ID!
                    b: ID
                }

                type Product @key(fields: "nested { id }") {
                    nested: Nested
                    args: JSON
                }

                type Nested @shareable {
                    id: ID!
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
                  "nested": [
                    {
                      "id": "1"
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
fn list_input_arg_type_compatibility_nested_input_field_nullable() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])
                    @init

                type Query {
                    productBatch(input: [ID!]! @is(field: "[{ a: nested.id }]")): [Product!]! @lookup
                }

                input DummyInput {
                    a: ID
                    b: ID
                }

                type Product @key(fields: "nested { id }") {
                    nested: Nested!
                    args: JSON
                }

                type Nested @shareable {
                    id: ID!
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
                  "nested": [
                    {
                      "id": "1"
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
fn list_input_arg_type_compatibility_nested_type_nullable() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])
                    @init

                type Query {
                    productBatch(input: [ID]! @is(field: "[nested.id]")): [Product!]! @lookup
                }

                type Product @key(fields: "nested { id }") {
                    nested: Nested
                    args: JSON
                }

                type Nested @shareable {
                    id: ID!
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
                  "nested": [
                    {
                      "id": "1"
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
fn list_input_arg_type_compatibility_nested_field_nullable() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])
                    @init

                type Query {
                    productBatch(input: [ID]! @is(field: "[nested.id]")): [Product!]! @lookup
                }

                type Product @key(fields: "nested { id }") {
                    nested: Nested!
                    args: JSON
                }

                type Nested @shareable {
                    id: ID
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
                  "nested": [
                    {
                      "id": "1"
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
fn object_input_arg_type_compatibility_all_nullable() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])
                    @init

                type Query {
                    productBatch(input: [DummyInput] @is(field: "[{ a: nested.id }]")): [Product!]! @lookup
                }

                input DummyInput {
                    a: ID
                    b: ID
                }

                type Product @key(fields: "nested { id }") {
                    nested: Nested!
                    args: JSON
                }

                type Nested @shareable {
                    id: ID!
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
                  "nested": [
                    {
                      "id": "1"
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
fn list_input_arg_type_compatibility_all_nullable() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])
                    @init

                type Query {
                    productBatch(ids: [ID] @is(field: "[nested.id]")): [Product!]! @lookup
                }

                type Product @key(fields: "nested { id }") {
                    nested: Nested!
                    args: JSON
                }

                type Nested @shareable {
                    id: ID!
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
                  "nested": [
                    {
                      "id": "1"
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
fn object_input_extra_optional_argument() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])
                    @init

                type Query {
                    productBatch(input: [DummyInput] @is(field: "[{ a: nested.id }]"), extra: Boolean): [Product!]! @lookup
                }

                input DummyInput {
                    a: ID!
                    b: ID
                }

                type Product @key(fields: "nested { id }") {
                    nested: Nested!
                    args: JSON
                }

                type Nested @shareable {
                    id: ID!
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
                  "nested": [
                    {
                      "id": "1"
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
fn arg_with_default_value() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])
                    @init

                type Query {
                    productBatch(input: [DummyInput] @is(field: "[{ a: nested.id }]"), extra: Boolean! = true): [Product!]! @lookup
                }

                input DummyInput {
                    a: ID!
                    extra: Boolean! = true
                }

                type Product @key(fields: "nested { id }") {
                    nested: Nested!
                    args: JSON
                }

                type Nested @shareable {
                    id: ID!
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
                  "input": [
                    {
                      "extra": true,
                      "a": "1"
                    }
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
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])
                    @init

                type Query {
                    productBatch(input: [DummyInput] @is(field: "[{ a: nested.id }]"), extra: [Boolean!]! = true): [Product!]! @lookup
                }

                input DummyInput {
                    a: ID!
                    extra: [Boolean!]! = true
                }

                type Product @key(fields: "nested { id }") {
                    nested: Nested!
                    args: JSON
                }

                type Nested @shareable {
                    id: ID!
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
                  "input": [
                    {
                      "extra": [
                        true
                      ],
                      "a": "1"
                    }
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
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])
                    @init

                type Query {
                    productBatch(
                        input: [DummyInput!]! @is(field: "[{ a: nested.id }]"),
                        ids: [ID!]! @is(field: "[nested.id]")
                    ): [Product!]! @lookup
                }

                input DummyInput {
                    a: ID!
                    b: ID
                }

                type Product @key(fields: "nested { id }") {
                    nested: Nested!
                    args: JSON
                }

                type Nested @shareable {
                    id: ID!
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
                  "nested": [
                    {
                      "id": "1"
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
fn no_matching_argument() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])
                    @init

                type Query {
                    productBatch(something: [JSON] @is(field: "[args]")): [Product!]! @lookup
                }

                input NestedInput {
                    id: ID!
                }

                type Product @key(fields: "nested { id }") {
                    nested: Nested!
                    args: JSON
                }

                type Nested @shareable {
                    id: ID!
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoArgs)
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Query.productBatch, for directive @lookup no matching @key directive was found
        See schema at 36:3:
        productBatch(something: [JSON]): [Product!]! @composite__lookup(graph: EXT) @join__field(graph: EXT)
        "#);
    })
}

// #[test]
// fn arg_good_name_bad_type() {
//     runtime().block_on(async {
//         let result = Gateway::builder()
//             .with_subgraph(gql_nested())
//             .with_subgraph_sdl(
//                 "ext",
//                 r#"
//                 extend schema
//                     @link(url: "static-1.0.0", import: ["@init"])
//                     @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])
//                     @init
//
//                 type Query {
//                     productBatch(nested: [Int] @is(field: "[nested.id]")): [Product!]! @lookup
//                 }
//
//                 input NestedInput {
//                     id: ID!
//                 }
//
//                 type Product @key(fields: "nested { id }") {
//                     nested: Nested!
//                     args: JSON
//                 }
//
//                 type Nested @shareable {
//                     id: ID!
//                 }
//
//                 scalar JSON
//                 "#,
//             )
//             .with_extension(EchoArgs)
//             .try_build()
//             .await;
//
//         insta::assert_snapshot!(result.unwrap_err(), @r#"
//         At site Query.productBatch, for directive @lookup no matching @key directive was found
//         See schema at 36:3:
//         productBatch(nested: [Int]): [Product!]! @composite__lookup(graph: EXT) @join__field(graph: EXT)
//         "#);
//     })
// }
//
// #[test]
// fn field_good_name_bad_type() {
//     runtime().block_on(async {
//         let result = Gateway::builder()
//             .with_subgraph(gql_nested())
//             .with_subgraph_sdl(
//                 "ext",
//                 r#"
//                 extend schema
//                     @link(url: "static-1.0.0", import: ["@init"])
//                     @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])
//                     @init
//
//                 type Query {
//                     productBatch(nested: [NestedInput!]): [Product!]! @lookup
//                 }
//
//                 input NestedInput {
//                     id: Int!
//                 }
//
//                 type Product @key(fields: "nested { id }") {
//                     nested: Nested!
//                     args: JSON
//                 }
//
//                 type Nested @shareable {
//                     id: ID!
//                 }
//
//                 scalar JSON
//                 "#,
//             )
//             .with_extension(EchoArgs)
//             .try_build()
//             .await;
//
//         insta::assert_snapshot!(result.unwrap_err(), @r#"
//         At site Query.productBatch, for directive @lookup no matching @key directive was found
//         See schema at 36:3:
//         productBatch(nested: [NestedInput!]): [Product!]! @composite__lookup(graph: EXT) @join__field(graph: EXT)
//         "#);
//     })
// }
//
// #[test]
// fn good_name_not_a_list() {
//     runtime().block_on(async {
//         let result = Gateway::builder()
//             .with_subgraph(gql_nested())
//             .with_subgraph_sdl(
//                 "ext",
//                 r#"
//                 extend schema
//                     @link(url: "static-1.0.0", import: ["@init"])
//                     @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])
//                     @init
//
//                 type Query {
//                     productBatch(nested: NestedInput!): [Product!]! @lookup
//                 }
//
//                 input NestedInput {
//                     id: ID!
//                 }
//
//                 type Product @key(fields: "nested { id }") {
//                     nested: Nested!
//                     args: JSON
//                 }
//
//                 type Nested @shareable {
//                     id: ID!
//                 }
//
//                 scalar JSON
//                 "#,
//             )
//             .with_extension(EchoArgs)
//             .try_build()
//             .await;
//
//         insta::assert_snapshot!(result.unwrap_err(), @r#"
//         At site Query.productBatch, for directive @lookup no matching @key directive was found
//         See schema at 36:3:
//         productBatch(nested: NestedInput!): [Product!]! @composite__lookup(graph: EXT) @join__field(graph: EXT)
//         "#);
//     })
// }
//
// #[test]
// fn ambiguous_multiple_arg_matches() {
//     runtime().block_on(async {
//         let result = Gateway::builder()
//             .with_subgraph(gql_nested())
//             .with_subgraph_sdl(
//                 "ext",
//                 r#"
//                 extend schema
//                     @link(url: "static-1.0.0", import: ["@init"])
//                     @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])
//                     @init
//
//                 type Query {
//                     productBatch(a: [NestedInput!], b: [NestedInput!]): [Product!]! @lookup
//                 }
//
//                 input NestedInput {
//                     id: ID!
//                 }
//
//                 type Product @key(fields: "nested { id }") {
//                     nested: Nested!
//                     args: JSON
//                 }
//
//                 type Nested @shareable {
//                     id: ID!
//                 }
//
//                 scalar JSON
//                 "#,
//             )
//             .with_extension(EchoArgs)
//             .try_build()
//             .await;
//
//         insta::assert_snapshot!(result.unwrap_err(), @r#"
//         At site Query.productBatch, for directive @lookup no matching @key directive was found
//         See schema at 36:3:
//         productBatch(a: [NestedInput!], b: [NestedInput!]): [Product!]! @composite__lookup(graph: EXT) @join__field(graph: EXT)
//         "#);
//     })
// }
//
// #[test]
// fn ambiguous_multiple_field_matches() {
//     runtime().block_on(async {
//         let result = Gateway::builder()
//             .with_subgraph(gql_nested())
//             .with_subgraph_sdl(
//                 "ext",
//                 r#"
//                 extend schema
//                     @link(url: "static-1.0.0", import: ["@init"])
//                     @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])
//                     @init
//
//                 type Query {
//                     productBatch(a: [NestedInput!]): [Product!]! @lookup
//                 }
//
//                 input NestedInput {
//                     a: ID
//                     b: ID
//                 }
//
//                 type Product @key(fields: "nested { id }") {
//                     nested: Nested!
//                     args: JSON
//                 }
//
//                 type Nested @shareable {
//                     id: ID!
//                 }
//
//                 scalar JSON
//                 "#,
//             )
//             .with_extension(EchoArgs)
//             .try_build()
//             .await;
//
//         insta::assert_snapshot!(result.unwrap_err(), @r#"
//         At site Query.productBatch, for directive @lookup no matching @key directive was found
//         See schema at 36:3:
//         productBatch(a: [NestedInput!]): [Product!]! @composite__lookup(graph: EXT) @join__field(graph: EXT)
//         "#);
//     })
// }
//
// #[test]
// fn extra_required_argument() {
//     runtime().block_on(async {
//         let result = Gateway::builder()
//             .with_subgraph(gql_nested())
//             .with_subgraph_sdl(
//                 "ext",
//                 r#"
//                 extend schema
//                     @link(url: "static-1.0.0", import: ["@init"])
//                     @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])
//                     @init
//
//                 type Query {
//                     productBatch(nested: [NestedInput!], required: Boolean!): [Product!]! @lookup
//                 }
//
//                 input NestedInput {
//                     id: ID!
//                 }
//
//                 type Product @key(fields: "nested { id }") {
//                     nested: Nested!
//                     args: JSON
//                 }
//
//                 type Nested @shareable {
//                     id: ID!
//                 }
//
//                 scalar JSON
//                 "#,
//             )
//             .with_extension(EchoArgs)
//             .try_build()
//             .await;
//
//         insta::assert_snapshot!(result.unwrap_err(), @r#"
//         At site Query.productBatch, for directive @lookup no matching @key directive was found
//         See schema at 36:3:
//         productBatch(nested: [NestedInput!], required: Boolean!): [Product!]! @composite__lookup(graph: EXT) @join__field(graph: EXT)
//         "#);
//     })
// }
//
// #[test]
// fn extra_required_field() {
//     runtime().block_on(async {
//         let result = Gateway::builder()
//             .with_subgraph(gql_nested())
//             .with_subgraph_sdl(
//                 "ext",
//                 r#"
//                 extend schema
//                     @link(url: "static-1.0.0", import: ["@init"])
//                     @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])
//                     @init
//
//                 type Query {
//                     productBatch(nested: [NestedInput!] @is(field: "[{ id: nested.id }]")): [Product!]! @lookup
//                 }
//
//                 input NestedInput {
//                     id: ID!
//                     required: Boolean!
//                 }
//
//                 type Product @key(fields: "nested { id }") {
//                     nested: Nested!
//                     args: JSON
//                 }
//
//                 type Nested @shareable {
//                     id: ID!
//                 }
//
//                 scalar JSON
//                 "#,
//             )
//             .with_extension(EchoArgs)
//             .try_build()
//             .await;
//
//         insta::assert_snapshot!(result.unwrap_err(), @r#"
//         At site Query.productBatch, for directive @lookup no matching @key directive was found
//         See schema at 36:3:
//         productBatch(nested: [NestedInput!]): [Product!]! @composite__lookup(graph: EXT) @join__field(graph: EXT)
//         "#);
//     })
// }
