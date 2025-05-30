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
                  "input": [
                    {
                      "a": "1"
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
                  "ids": [
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
                  "input": [
                    {
                      "a": "1"
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
                  "ids": [
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
                  "input": [
                    {
                      "a": "1"
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
                  "ids": [
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
                  "input": [
                    {
                      "a": "1"
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
                  "input": [
                    {
                      "a": "1"
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
                  "input": [
                    {
                      "a": "1"
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
                  "input": [
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
                  "input": [
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
                    nested: Nested
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
                  "input": [
                    {
                      "a": "1"
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
                    nested: Nested
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
                  "ids": [
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
                  "input": [
                    {
                      "a": "1"
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
                  "input": [
                    {
                      "a": "1"
                    }
                  ],
                  "ids": [
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
        See schema at 40:3:
        productBatch(something: [JSON] @composite__is(graph: EXT, field: "[args]")): [Product!]! @composite__lookup(graph: EXT) @join__field(graph: EXT)
        "#);
    })
}

#[test]
fn extra_required_argument() {
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
                    productBatch(input: [DummyInput!] @is(field: "[{ a: nested.id }]"), required: Boolean!): [Product!]! @lookup
                }

                input DummyInput {
                    a: ID!
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
        At site Query.productBatch, for directive @lookup Argument 'required' is required but is not injected any @is directive.
        See schema at 40:3:
        productBatch(input: [DummyInput!] @composite__is(graph: EXT, field: "[{ a: nested.id }]"), required: Boolean!): [Product!]! @composite__lookup(graph: EXT) @join__field(graph: EXT)
        "#);
    })
}

#[test]
fn extra_required_field() {
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
                    productBatch(input: [DummyInput!] @is(field: "[{ a: nested.id }]")): [Product!]! @lookup
                }

                input DummyInput {
                    a: ID!
                    required: Boolean!
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
        At site Query.productBatch, for directive @lookup for associated @is directive: For Query.productBatch.input, field 'required' is required but it's missing from the FieldSelectionMap
        See schema at 40:51:
        (graph: EXT, field: "[{ a: nested.id }]")
        "#);
    })
}

#[test]
fn invalid_batch() {
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
                    productBatch(input: DummyInput! @is(field: "{ a: nested.id }")): [Product!]! @lookup
                }

                input DummyInput {
                    a: ID!
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
        At site Query.productBatch, for directive @lookup for associated @is directive: Cannot select object from [Product!]!, it's a list
        See schema at 40:49:
        (graph: EXT, field: "{ a: nested.id }")
        "#);
    })
}
