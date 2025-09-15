use integration_tests::{gateway::Gateway, runtime};

use super::super::super::{EchoLookup, gql_nested};

#[test]
fn object_input() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])


                type Query {
                    productBatch(input: DummyInput! @is(field: "{ a: nested.id }")): Product! @lookup @echo
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
            .with_extension(EchoLookup::single())
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
                    "a": "1"
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
fn scalar_input() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])


                type Query {
                    productBatch(ids: ID! @is(field: "nested.id")): Product! @lookup @echo
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
            .with_extension(EchoLookup::single())
            .build()
            .await;

        let response = engine.post("query { products { args } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "args": {
                  "ids": "1"
                }
              }
            ]
          }
        }
        "#);
    })
}

#[test]
fn object_input_arg_type_compatibility_nullable() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])


                type Query {
                    productBatch(input: DummyInput @is(field: "{ a: nested.id }")): Product! @lookup @echo
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
            .with_extension(EchoLookup::single())
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
                    "a": "1"
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
fn scalar_input_arg_type_compatibility_nullable() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])


                type Query {
                    productBatch(ids: ID @is(field: "nested.id")): Product! @lookup @echo
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
            .with_extension(EchoLookup::single())
            .build()
            .await;

        let response = engine.post("query { products { args } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "args": {
                  "ids": "1"
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
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])


                type Query {
                    productBatch(input: DummyInput! @is(field: "{ a: nested.id }")): Product! @lookup @echo
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
            .with_extension(EchoLookup::single())
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
                    "a": "1"
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
fn object_input_arg_type_compatibility_nested_field_nullable_with_nullable_input_field() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])


                type Query {
                    productBatch(input: DummyInput! @is(field: "{ a: nested.id }")): Product! @lookup @echo
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
            .with_extension(EchoLookup::single())
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
                    "a": "1"
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
fn object_input_arg_type_compatibility_nested_type_nullable_with_nullable_input_object() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])


                type Query {
                    productBatch(input: DummyInput @is(field: "{ a: nested.id }")): Product! @lookup @echo
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
            .with_extension(EchoLookup::single())
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
                    "a": "1"
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
fn scalar_input_arg_type_compatibility_nested_type_nullable() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])


                type Query {
                    productBatch(input: ID @is(field: "nested.id")): Product! @lookup @echo
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
            .with_extension(EchoLookup::single())
            .build()
            .await;

        let response = engine.post("query { products { args } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "args": {
                  "input": "1"
                }
              }
            ]
          }
        }
        "#);
    })
}

#[test]
fn scalar_input_arg_type_compatibility_nested_field_nullable() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])


                type Query {
                    productBatch(input: ID @is(field: "nested.id")): Product! @lookup @echo
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
            .with_extension(EchoLookup::single())
            .build()
            .await;

        let response = engine.post("query { products { args } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "args": {
                  "input": "1"
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
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])


                type Query {
                    productBatch(input: DummyInput @is(field: "{ a: nested.id }")): Product! @lookup @echo
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
            .with_extension(EchoLookup::single())
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
                    "a": "1"
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
fn scalar_input_arg_type_compatibility_all_nullable() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])


                type Query {
                    productBatch(ids: ID @is(field: "nested.id")): Product! @lookup @echo
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
            .with_extension(EchoLookup::single())
            .build()
            .await;

        let response = engine.post("query { products { args } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "args": {
                  "ids": "1"
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
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])


                type Query {
                    productBatch(input: DummyInput @is(field: "{ a: nested.id }"), extra: Boolean): Product! @lookup @echo
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
            .with_extension(EchoLookup::single())
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
                    "a": "1"
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
fn arg_with_default_value() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])


                type Query {
                    productBatch(input: DummyInput @is(field: "{ a: nested.id }"), extra: Boolean! = true): Product! @lookup @echo
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
            .with_extension(EchoLookup::single())
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
                    "a": "1",
                    "extra": true
                  },
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
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])


                type Query {
                    productBatch(input: DummyInput @is(field: "{ a: nested.id }"), extra: [Boolean!]! = true): Product! @lookup @echo
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
            .with_extension(EchoLookup::single())
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
                    "a": "1",
                    "extra": [
                      true
                    ]
                  },
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
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])


                type Query {
                    productBatch(
                        input: DummyInput! @is(field: "{ a: nested.id }"),
                        ids: ID! @is(field: "nested.id")
                    ): Product! @lookup @echo
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
            .with_extension(EchoLookup::single())
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
                    "a": "1"
                  },
                  "ids": "1"
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
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])


                type Query {
                    productBatch(something: JSON @is(field: "args")): Product! @lookup @echo
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
            .with_extension(EchoLookup::single())
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Query.productBatch, for directive @lookup no matching @key directive was found
        47 | {
        48 |   productBatch(something: JSON @composite__is(graph: EXT, field: "args")): Product! @composite__lookup(graph: EXT) @extension__directive(graph: EXT, extension: ECHO, name: "echo", arguments: {}) @join__field(graph: EXT)
               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        49 |   products: [Product!]! @join__field(graph: GQL)
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
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])


                type Query {
                    productBatch(input: DummyInput! @is(field: "{ a: nested.id }"), required: Boolean!): Product! @lookup @echo
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
            .with_extension(EchoLookup::single())
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Query.productBatch, for directive @lookup Argument 'required' is required but is not injected by any @is directive.
        47 | {
        48 |   productBatch(input: DummyInput! @composite__is(graph: EXT, field: "{ a: nested.id }"), required: Boolean!): Product! @composite__lookup(graph: EXT) @extension__directive(graph: EXT, extension: ECHO, name: "echo", arguments: {}) @join__field(graph: EXT)
               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        49 |   products: [Product!]! @join__field(graph: GQL)
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
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])


                type Query {
                    productBatch(input: DummyInput! @is(field: "{ a: nested.id }")): Product! @lookup @echo
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
            .with_extension(EchoLookup::single())
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Query.productBatch, for directive @lookup for associated @is directive: For Query.productBatch.input, field 'required' is required but it's missing from the FieldSelectionMap
        47 | {
        48 |   productBatch(input: DummyInput! @composite__is(graph: EXT, field: "{ a: nested.id }")): Product! @composite__lookup(graph: EXT) @extension__directive(graph: EXT, extension: ECHO, name: "echo", arguments: {}) @join__field(graph: EXT)
                                                             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        49 |   products: [Product!]! @join__field(graph: GQL)
        "#);
    })
}

#[test]
fn invalid_single() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@is", "@key", "@shareable"])


                type Query {
                    productBatch(input: [DummyInput!] @is(field: "[{ a: nested.id }]")): Product! @lookup @echo
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
            .with_extension(EchoLookup::single())
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Query.productBatch, for directive @lookup for associated @is directive: Product! is not a list but treated as such
        47 | {
        48 |   productBatch(input: [DummyInput!] @composite__is(graph: EXT, field: "[{ a: nested.id }]")): Product! @composite__lookup(graph: EXT) @extension__directive(graph: EXT, extension: ECHO, name: "echo", arguments: {}) @join__field(graph: EXT)
                                                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        49 |   products: [Product!]! @join__field(graph: GQL)
        "#);
    })
}
