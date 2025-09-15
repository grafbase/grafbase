use integration_tests::{gateway::Gateway, runtime};

use super::super::super::{EchoLookup, gql_ab, gql_ab_id_int};

#[test]
fn basic() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_ab())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])


                type Query {
                    productBatch(key: Key! @is(field: "{ a b }")): Product! @lookup @echo
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
fn renames() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_ab())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])


                type Query {
                    productBatch(key: Key! @is(field: "{ aa: a bb: b }")): Product! @lookup @echo
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
                    "aa": "A1",
                    "bb": "B1"
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
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])


                type Query {
                    productBatch(key: Key @is(field: "{ a b }")): Product! @lookup @echo
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
fn all_nullable() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_ab())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])


                type Query {
                    productBatch(key: Key @is(field: "{ a b }")): Product! @lookup @echo
                }

                input Key {
                    a: ID
                    b: ID
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
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])


                type Query {
                    productBatch(key: Key! @is(field: "{ a b }"), x: [ID!]): Product! @lookup @echo
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
fn arg_with_same_name_and_extra_optional_input_field() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_ab())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])


                type Query {
                    productBatch(key: Key! @is(field: "{ a b }")): Product! @lookup @echo
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
fn different_input_field_types() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_ab_id_int())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])


                type Query {
                    productBatch(key: Key! @is(field: "{ a b }")): Product! @lookup @echo
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
            .build()
            .await;

        let response = engine.post("query { products { args } }").await;
        insta::assert_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "args": {
                  "key": {
                    "a": "A1",
                    "b": 1
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
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])


                type Query {
                    productBatch(key: Key! @is(field: "{ a b }")): Product! @lookup @echo
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
                    "b": 1,
                    "extra": false
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
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])


                type Query {
                    productBatch(key: Key! @is(field: "{ a b }")): Product! @lookup @echo
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
                    "b": 1,
                    "extra": [
                      false
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
fn cannot_inject_nullable_field_into_required() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph(gql_ab_id_int())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])


                type Query {
                    productBatch(key: Key! @is(field: "{ a b }")): Product! @lookup @echo
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
        * At site Query.productBatch, for directive @lookup for associated @is directive: Incompatible wrapping, cannot map Product.b (Int) into Key.b (Int!)
        41 | {
        42 |   productBatch(key: Key! @composite__is(graph: EXT, field: "{ a b }")): Product! @composite__lookup(graph: EXT) @extension__directive(graph: EXT, extension: ECHO, name: "echo", arguments: {}) @join__field(graph: EXT)
                                                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        43 |   products: [Product!]! @join__field(graph: GQL)
        "#);
    })
}

#[test]
fn invalid_single() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph(gql_ab_id_int())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])


                type Query {
                    productBatch(key: Key! @is(field: "{ a b }")): Product! @lookup @echo
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
        * At site Query.productBatch, for directive @lookup for associated @is directive: Incompatible wrapping, cannot map Product.b (Int) into Key.b (Int!)
        41 | {
        42 |   productBatch(key: Key! @composite__is(graph: EXT, field: "{ a b }")): Product! @composite__lookup(graph: EXT) @extension__directive(graph: EXT, extension: ECHO, name: "echo", arguments: {}) @join__field(graph: EXT)
                                                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        43 |   products: [Product!]! @join__field(graph: GQL)
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
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])


                type Query {
                    productBatch(ids: Key! @is(field: "{ a b }"), required: Boolean!): Product! @lookup @echo
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
        * At site Query.productBatch, for directive @lookup Argument 'required' is required but is not injected by any @is directive.
        41 | {
        42 |   productBatch(ids: Key! @composite__is(graph: EXT, field: "{ a b }"), required: Boolean!): Product! @composite__lookup(graph: EXT) @extension__directive(graph: EXT, extension: ECHO, name: "echo", arguments: {}) @join__field(graph: EXT)
               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        43 |   products: [Product!]! @join__field(graph: GQL)
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
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])


                type Query {
                    productBatch(key: Key! @is(field: "{ a b }")): Product! @lookup @echo
                }

                input Key {
                    a: ID!
                    b: Int!
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
        * At site Query.productBatch, for directive @lookup for associated @is directive: For Query.productBatch.key, field 'x' is required but it's missing from the FieldSelectionMap
        41 | {
        42 |   productBatch(key: Key! @composite__is(graph: EXT, field: "{ a b }")): Product! @composite__lookup(graph: EXT) @extension__directive(graph: EXT, extension: ECHO, name: "echo", arguments: {}) @join__field(graph: EXT)
                                                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        43 |   products: [Product!]! @join__field(graph: GQL)
        "#);
    })
}
