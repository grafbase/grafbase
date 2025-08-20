use integration_tests::{gateway::Gateway, runtime};

use super::super::super::{EchoLookup, gql_nested};

#[test]
fn arg_with_same_name() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@shareable"])


                type Query {
                    productBatch(nested: [NestedInput!]!): [Product!]! @lookup @echo
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
            .with_extension(EchoLookup::batch())
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
fn arg_type_compatibility_nullable_list() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@shareable"])


                type Query {
                    productBatch(nested: [NestedInput!]): [Product!]! @lookup @echo
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
            .with_extension(EchoLookup::batch())
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
fn arg_type_compatibility_inner_nullable() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@shareable"])


                type Query {
                    productBatch(nested: [NestedInput]!): [Product!]! @lookup @echo
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
            .with_extension(EchoLookup::batch())
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
fn arg_type_compatibility_nested_nullable() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@shareable"])


                type Query {
                    productBatch(nested: [NestedInput!]!): [Product!]! @lookup @echo
                }

                input NestedInput {
                    id: ID
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
            .with_extension(EchoLookup::batch())
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
fn arg_type_compatibility_all_nullable() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@shareable"])


                type Query {
                    productBatch(nested: [NestedInput]): [Product!]! @lookup @echo
                }

                input NestedInput {
                    id: ID
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
            .with_extension(EchoLookup::batch())
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
fn arg_with_same_name_and_extra_optional_arg_with_matching_type() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@shareable"])


                type Query {
                    productBatch(nested: [NestedInput!], anything: [NestedInput!]): [Product!]! @lookup @echo
                }

                input NestedInput {
                    id: ID!
                    something: ID
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
            .with_extension(EchoLookup::batch())
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
fn arg_with_different_name() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@shareable"])


                type Query {
                    productBatch(x: [NestedInput!]): [Product!]! @lookup @echo
                }

                input NestedInput {
                    y: ID!
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
            .with_extension(EchoLookup::batch())
            .build()
            .await;

        let response = engine.post("query { products { args } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "args": {
                  "x": [
                    {
                      "y": "1"
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
fn arg_with_different_name_and_extra_optional_arg_with_matching_name() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@shareable"])


                type Query {
                    productBatch(x: [NestedInput!], nested: ID): [Product!]! @lookup @echo
                }

                input NestedInput {
                    y: ID!
                    id: Int
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
            .with_extension(EchoLookup::batch())
            .build()
            .await;

        let response = engine.post("query { products { args } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "args": {
                  "x": [
                    {
                      "y": "1"
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
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@shareable"])


                type Query {
                    productBatch(nested: [NestedInput!], extra: Boolean! = true): [Product!]! @lookup @echo
                }

                input NestedInput {
                    id: ID!
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
            .with_extension(EchoLookup::batch())
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
                      "extra": true,
                      "id": "1"
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
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@shareable"])


                type Query {
                    productBatch(nested: [NestedInput!], extra: [Boolean!]! = true): [Product!]! @lookup @echo
                }

                input NestedInput {
                    id: ID!
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
            .with_extension(EchoLookup::batch())
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
                      "extra": [
                        true
                      ],
                      "id": "1"
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
fn no_arguments() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@shareable"])


                type Query {
                    productBatch: [Product!]! @lookup @echo
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
            .with_extension(EchoLookup::batch())
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Query.productBatch, for directive @lookup no matching @key directive was found
        35 | {
        36 |   productBatch: [Product!]! @composite__lookup(graph: EXT) @extension__directive(graph: EXT, extension: ECHO, name: "echo", arguments: {}) @join__field(graph: EXT)
               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        37 |   products: [Product!]! @join__field(graph: GQL)
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
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@shareable"])


                type Query {
                    productBatch(somethign: Int): [Product!]! @lookup @echo
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
            .with_extension(EchoLookup::batch())
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Query.productBatch, for directive @lookup no matching @key directive was found
        35 | {
        36 |   productBatch(somethign: Int): [Product!]! @composite__lookup(graph: EXT) @extension__directive(graph: EXT, extension: ECHO, name: "echo", arguments: {}) @join__field(graph: EXT)
               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        37 |   products: [Product!]! @join__field(graph: GQL)
        "#);
    })
}

#[test]
fn no_matching_nested_field() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@shareable"])


                type Query {
                    productBatch(nested: [NestedInput!]): [Product!]! @lookup @echo
                }

                input NestedInput {
                    something: Int
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
            .with_extension(EchoLookup::batch())
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Query.productBatch, for directive @lookup no matching @key directive was found
        35 | {
        36 |   productBatch(nested: [NestedInput!]): [Product!]! @composite__lookup(graph: EXT) @extension__directive(graph: EXT, extension: ECHO, name: "echo", arguments: {}) @join__field(graph: EXT)
               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        37 |   products: [Product!]! @join__field(graph: GQL)
        "#);
    })
}

#[test]
fn arg_good_name_bad_type() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@shareable"])


                type Query {
                    productBatch(nested: [Int]): [Product!]! @lookup @echo
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
            .with_extension(EchoLookup::batch())
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Query.productBatch, for directive @lookup no matching @key directive was found
        35 | {
        36 |   productBatch(nested: [Int]): [Product!]! @composite__lookup(graph: EXT) @extension__directive(graph: EXT, extension: ECHO, name: "echo", arguments: {}) @join__field(graph: EXT)
               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        37 |   products: [Product!]! @join__field(graph: GQL)
        "#);
    })
}

#[test]
fn field_good_name_bad_type() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@shareable"])


                type Query {
                    productBatch(nested: [NestedInput!]): [Product!]! @lookup @echo
                }

                input NestedInput {
                    id: Int!
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
            .with_extension(EchoLookup::batch())
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Query.productBatch, for directive @lookup no matching @key directive was found
        35 | {
        36 |   productBatch(nested: [NestedInput!]): [Product!]! @composite__lookup(graph: EXT) @extension__directive(graph: EXT, extension: ECHO, name: "echo", arguments: {}) @join__field(graph: EXT)
               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        37 |   products: [Product!]! @join__field(graph: GQL)
        "#);
    })
}

#[test]
fn good_name_not_a_list() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@shareable"])


                type Query {
                    productBatch(nested: NestedInput!): [Product!]! @lookup @echo
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
            .with_extension(EchoLookup::batch())
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Query.productBatch, for directive @lookup no matching @key directive was found
        35 | {
        36 |   productBatch(nested: NestedInput!): [Product!]! @composite__lookup(graph: EXT) @extension__directive(graph: EXT, extension: ECHO, name: "echo", arguments: {}) @join__field(graph: EXT)
               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        37 |   products: [Product!]! @join__field(graph: GQL)
        "#);
    })
}

#[test]
fn ambiguous_multiple_arg_matches() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@shareable"])


                type Query {
                    productBatch(a: [NestedInput!], b: [NestedInput!]): [Product!]! @lookup @echo
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
            .with_extension(EchoLookup::batch())
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Query.productBatch, for directive @lookup no matching @key directive was found
        35 | {
        36 |   productBatch(a: [NestedInput!], b: [NestedInput!]): [Product!]! @composite__lookup(graph: EXT) @extension__directive(graph: EXT, extension: ECHO, name: "echo", arguments: {}) @join__field(graph: EXT)
               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        37 |   products: [Product!]! @join__field(graph: GQL)
        "#);
    })
}

#[test]
fn ambiguous_multiple_field_matches() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph(gql_nested())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@shareable"])


                type Query {
                    productBatch(a: [NestedInput!]): [Product!]! @lookup @echo
                }

                input NestedInput {
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
            .with_extension(EchoLookup::batch())
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Query.productBatch, for directive @lookup no matching @key directive was found
        35 | {
        36 |   productBatch(a: [NestedInput!]): [Product!]! @composite__lookup(graph: EXT) @extension__directive(graph: EXT, extension: ECHO, name: "echo", arguments: {}) @join__field(graph: EXT)
               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        37 |   products: [Product!]! @join__field(graph: GQL)
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
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@shareable"])


                type Query {
                    productBatch(nested: [NestedInput!], required: Boolean!): [Product!]! @lookup @echo
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
            .with_extension(EchoLookup::batch())
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Query.productBatch, for directive @lookup no matching @key directive was found
        35 | {
        36 |   productBatch(nested: [NestedInput!], required: Boolean!): [Product!]! @composite__lookup(graph: EXT) @extension__directive(graph: EXT, extension: ECHO, name: "echo", arguments: {}) @join__field(graph: EXT)
               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        37 |   products: [Product!]! @join__field(graph: GQL)
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
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@shareable"])


                type Query {
                    productBatch(nested: [NestedInput!]): [Product!]! @lookup @echo
                }

                input NestedInput {
                    id: ID!
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
            .with_extension(EchoLookup::batch())
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Query.productBatch, for directive @lookup no matching @key directive was found
        35 | {
        36 |   productBatch(nested: [NestedInput!]): [Product!]! @composite__lookup(graph: EXT) @extension__directive(graph: EXT, extension: ECHO, name: "echo", arguments: {}) @join__field(graph: EXT)
               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        37 |   products: [Product!]! @join__field(graph: GQL)
        "#);
    })
}
