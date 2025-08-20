use integration_tests::{gateway::Gateway, runtime};

use super::super::super::{EchoLookup, gql_ab, gql_ab_id_int};

#[test]
fn arg_with_same_name() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_ab())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])


                type Query {
                    productBatch(key: Key!): Product! @lookup @echo
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
                  "key": {
                    "a": "A1",
                    "b": "B1"
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
fn arg_type_compatibility_nullable() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_ab())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])


                type Query {
                    productBatch(key: Key): Product! @lookup @echo
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
                  "key": {
                    "a": "A1",
                    "b": "B1"
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
fn arg_with_same_name_and_extra_optional_input_argument() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_ab())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])


                type Query {
                    productBatch(key: Key!, x: [ID!]): Product! @lookup @echo
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
                  "key": {
                    "a": "A1",
                    "b": "B1"
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
fn arg_with_same_name_and_extra_optional_input_field_with_matching_type() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_ab())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])


                type Query {
                    productBatch(key: Key!, x: Key): Product! @lookup @echo
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
                  "key": {
                    "a": "A1",
                    "b": "B1"
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
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])


                type Query {
                    productBatch(productKey: Key!): Product! @lookup @echo
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
                  "productKey": {
                    "c": "A1",
                    "d": 1
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
fn arg_with_different_name_and_extra_optional_arg_with_matching_name() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_ab_id_int())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])


                type Query {
                    productBatch(productKey: Key!, key: Int): Product! @lookup @echo
                }

                input Key {
                    c: ID!
                    d: Int!
                    a: String
                    b: Float
                }

                type Product @key(fields: "a b") {
                    a: ID!
                    b: Int!
                    args: JSON
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
                  "productKey": {
                    "c": "A1",
                    "d": 1
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
fn field_with_default_value() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_ab_id_int())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])


                type Query {
                    productBatch(key: Key!, x: Int = 0): Product! @lookup @echo
                }

                input Key {
                    c: ID!
                    d: Int!
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
                  "key": {
                    "extra": false,
                    "c": "A1",
                    "d": 1
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
fn field_with_default_value_coercion() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_ab_id_int())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])


                type Query {
                    productBatch(key: Key!, x: [Int!] = 0): Product! @lookup @echo
                }

                input Key {
                    c: ID!
                    d: Int!
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
                  "key": {
                    "extra": [
                      false
                    ],
                    "c": "A1",
                    "d": 1
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
fn no_arguments() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph(gql_ab())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])


                type Query {
                    productBatch: Product! @lookup @echo
                }

                type Product @key(fields: "a b") {
                    a: ID!
                    b: ID!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoLookup::single())
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Query.productBatch, for directive @lookup no matching @key directive was found
        29 | {
        30 |   productBatch: Product! @composite__lookup(graph: EXT) @extension__directive(graph: EXT, extension: ECHO, name: "echo", arguments: {}) @join__field(graph: EXT)
               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        31 |   products: [Product!]! @join__field(graph: GQL)
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
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])


                type Query {
                    productBatch(somethign: Int): Product! @lookup @echo
                }

                type Product @key(fields: "a b") {
                    a: ID!
                    b: ID!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoLookup::single())
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Query.productBatch, for directive @lookup no matching @key directive was found
        29 | {
        30 |   productBatch(somethign: Int): Product! @composite__lookup(graph: EXT) @extension__directive(graph: EXT, extension: ECHO, name: "echo", arguments: {}) @join__field(graph: EXT)
               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        31 |   products: [Product!]! @join__field(graph: GQL)
        "#);
    })
}

#[test]
fn good_name_bad_type() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph(gql_ab_id_int())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])


                type Query {
                    productBatch(key: Key!): Product! @lookup @echo
                }

                type Key {
                    a: String
                    b: Float
                }

                type Product @key(fields: "a b") {
                    a: ID!
                    b: Int!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoLookup::single())
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Query.productBatch, for directive @lookup no matching @key directive was found
        36 | {
        37 |   productBatch(key: Key!): Product! @composite__lookup(graph: EXT) @extension__directive(graph: EXT, extension: ECHO, name: "echo", arguments: {}) @join__field(graph: EXT)
               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        38 |   products: [Product!]! @join__field(graph: GQL)
        "#);
    })
}

#[test]
fn good_name_but_a_list() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph(gql_ab_id_int())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])


                type Query {
                    productBatch(key: [Key!]): Product! @lookup @echo
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
            .with_extension(EchoLookup::single())
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Query.productBatch, for directive @lookup no matching @key directive was found
        29 | {
        30 |   productBatch(key: [Key!]): Product! @composite__lookup(graph: EXT) @extension__directive(graph: EXT, extension: ECHO, name: "echo", arguments: {}) @join__field(graph: EXT)
               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        31 |   products: [Product!]! @join__field(graph: GQL)
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
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])


                type Query {
                    productBatch(key: Key!): Product! @lookup @echo
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
            .with_extension(EchoLookup::single())
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Query.productBatch, for directive @lookup no matching @key directive was found
        29 | {
        30 |   productBatch(key: Key!): Product! @composite__lookup(graph: EXT) @extension__directive(graph: EXT, extension: ECHO, name: "echo", arguments: {}) @join__field(graph: EXT)
               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        31 |   products: [Product!]! @join__field(graph: GQL)
        "#);
    })
}

#[test]
fn ambiguous_multiple_arg_matches() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph(gql_ab_id_int())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])


                type Query {
                    productBatch(a: Key!, b: Key!): Product! @lookup @echo
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
            .with_extension(EchoLookup::single())
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Query.productBatch, for directive @lookup no matching @key directive was found
        29 | {
        30 |   productBatch(a: Key!, b: Key!): Product! @composite__lookup(graph: EXT) @extension__directive(graph: EXT, extension: ECHO, name: "echo", arguments: {}) @join__field(graph: EXT)
               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        31 |   products: [Product!]! @join__field(graph: GQL)
        "#);
    })
}

#[test]
fn ambiguous_multiple_input_field_matches() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph(gql_ab_id_int())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])


                type Query {
                    productBatch(key: Key!): Product! @lookup @echo
                }

                input Key {
                    x: ID
                    y: ID
                    b: Int
                }

                type Product @key(fields: "a b") {
                    a: ID!
                    b: Int!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoLookup::single())
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Query.productBatch, for directive @lookup no matching @key directive was found
        29 | {
        30 |   productBatch(key: Key!): Product! @composite__lookup(graph: EXT) @extension__directive(graph: EXT, extension: ECHO, name: "echo", arguments: {}) @join__field(graph: EXT)
               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        31 |   products: [Product!]! @join__field(graph: GQL)
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
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])


                type Query {
                    productBatch(ids: Key!, required: Boolean!): Product! @lookup @echo
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
            .with_extension(EchoLookup::single())
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Query.productBatch, for directive @lookup no matching @key directive was found
        29 | {
        30 |   productBatch(ids: Key!, required: Boolean!): Product! @composite__lookup(graph: EXT) @extension__directive(graph: EXT, extension: ECHO, name: "echo", arguments: {}) @join__field(graph: EXT)
               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        31 |   products: [Product!]! @join__field(graph: GQL)
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
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])


                type Query {
                    productBatch(key: Key!): Product! @lookup @echo
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
            .with_extension(EchoLookup::single())
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Query.productBatch, for directive @lookup no matching @key directive was found
        29 | {
        30 |   productBatch(key: Key!): Product! @composite__lookup(graph: EXT) @extension__directive(graph: EXT, extension: ECHO, name: "echo", arguments: {}) @join__field(graph: EXT)
               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        31 |   products: [Product!]! @join__field(graph: GQL)
        "#);
    })
}
