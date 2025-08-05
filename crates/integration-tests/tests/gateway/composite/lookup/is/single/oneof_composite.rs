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
                    productBatch(input: Lookup! @is(field: "{ key: { a b } }")): Product! @lookup @echo
                }

                input Lookup @oneOf {
                    key: Key
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
                  "input": {
                    "key": {
                      "a": "A1",
                      "b": "B1"
                    }
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
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])


                type Query {
                    productBatch(input: Lookup @is(field: "{ key: { a b } }")): Product! @lookup @echo
                }

                input Lookup @oneOf {
                    key: Key
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
                  "input": {
                    "key": {
                      "a": "A1",
                      "b": "B1"
                    }
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
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])


                type Query {
                    productBatch(input: Lookup! @is(field: "{ key: { a b } }")): Product! @lookup @echo
                }

                input Lookup @oneOf {
                    key: Key
                    anything: ID
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
                  "input": {
                    "key": {
                      "a": "A1",
                      "b": "B1"
                    }
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
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])


                type Query {
                    productBatch(input: Lookup! @is(field: "{ key: { c: a d: b } }")): Product! @lookup @echo
                }

                input Lookup @oneOf {
                    key: Key
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
                  "input": {
                    "key": {
                      "c": "A1",
                      "d": 1
                    }
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
                    productBatch(input: Lookup! @is(field: "{ key: [{ a b }] }")): Product! @lookup @echo
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
            .with_extension(EchoLookup::single())
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Query.productBatch, for directive @lookup for associated @is directive: Product! is not a list but treated as such
        See schema at 34:45:
        (graph: EXT, field: "{ key: [{ a b }] }")
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
                    productBatch(lookup: Lookup! @is(field: "{ key: { a b } }"), required: Boolean!): Product! @lookup @echo
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
            .with_extension(EchoLookup::single())
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Query.productBatch, for directive @lookup Argument 'required' is required but is not injected by any @is directive.
        See schema at 34:3:
        productBatch(lookup: Lookup! @composite__is(graph: EXT, field: "{ key: { a b } }"), required: Boolean!): Product! @composite__lookup(graph: EXT) @extension__directive(graph: EXT, extension: ECHO, name: "echo", arguments: {}) @join__field(graph: EXT)
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
                    productBatch(lookup: Lookup! @is(field: "{ key: { a b } }")): Product! @lookup @echo
                }

                input Lookup @oneOf {
                    key: Key
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
        At site Query.productBatch, for directive @lookup for associated @is directive: For Lookup.key, field 'x' is required but it's missing from the FieldSelectionMap
        See schema at 34:46:
        (graph: EXT, field: "{ key: { a b } }")
        "#);
    })
}
