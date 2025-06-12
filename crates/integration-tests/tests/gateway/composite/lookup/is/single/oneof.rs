use integration_tests::{gateway::Gateway, runtime};

use super::super::super::{EchoLookup, gql_id, gql2_name};

#[test]
fn basic() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_id())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])


                type Query {
                    productBatch(input: Lookup! @is(field: "{ id }")): Product! @lookup @echo
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
            .with_extension(EchoLookup { batch: false })
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
                    "id": "1"
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
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])


                type Query {
                    productBatch(input: Lookup! @is(field: "{ id } | { name }")): Product! @lookup @echo
                }

                input Lookup @oneOf {
                    id: ID
                    name: String
                }

                type Product @key(fields: "id") @key(fields: "name") {
                    id: ID!
                    name: String!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoLookup { batch: false })
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
                    "id": "1"
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
                    "name": "name1"
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
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])


                type Query {
                    productBatch(input: Lookup @is(field: "{ id }")): Product! @lookup @echo
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
            .with_extension(EchoLookup { batch: false })
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
                    "id": "1"
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
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])


                type Query {
                    productBatch(input: Lookup! @is(field: "{ id }")): Product! @lookup @echo
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
            .with_extension(EchoLookup { batch: false })
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
                    "id": "1"
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
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])


                type Query {
                    productBatch(input: Lookup! @is(field: "{ id }")): Product! @lookup @echo
                }

                input Lookup @oneOf {
                    id: ID
                    anything: ID
                }

                type Product @key(fields: "id") {
                    id: ID!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoLookup { batch: false })
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
                    "id": "1"
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
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])


                type Query {
                    productBatch(input: Lookup! @is(field: "{ productId: id }")): Product! @lookup @echo
                }

                input Lookup @oneOf {
                    productId: ID
                }

                type Product @key(fields: "id") {
                    id: ID!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoLookup { batch: false })
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
                    "productId": "1"
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
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])


                type Query {
                    productBatch(input: Lookup! @is(field: "{ id }")): Product! @lookup @echo
                }

                input Lookup @oneOf {
                    name: String
                    id: ID
                }

                type Product @key(fields: "id") {
                    id: ID!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoLookup { batch: false })
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
                    "id": "1"
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
            .with_subgraph(gql_id())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])


                type Query {
                    productBatch(id: Lookup! @is(field: "{ id: [id] }")): Product! @lookup @echo
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
            .with_extension(EchoLookup { batch: false })
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Query.productBatch, for directive @lookup for associated @is directive: Product! is not a list but treated as such
        See schema at 33:42:
        (graph: EXT, field: "{ id: [id] }")
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
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])


                type Query {
                    productBatch(a: Lookup!  @is(field: "{ id }"), b: Lookup!  @is(field: "{ id }")): Product! @lookup @echo
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
            .with_extension(EchoLookup { batch: false })
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Query.productBatch, for directive @lookup With a @oneOf argument, only one @is directive is supported for @lookup.
        See schema at 33:3:
        productBatch(a: Lookup! @composite__is(graph: EXT, field: "{ id }"), b: Lookup! @composite__is(graph: EXT, field: "{ id }")): Product! @composite__lookup(graph: EXT) @extension__directive(graph: EXT, extension: ECHO, name: "echo", arguments: {}) @join__field(graph: EXT)
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
                    @link(url: "echo-1.0.0", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is"])


                type Query {
                    productBatch(lookup: Lookup! @is(field: "{ id }"), required: Boolean!): Product! @lookup @echo
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
            .with_extension(EchoLookup { batch: false })
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Query.productBatch, for directive @lookup Argument 'required' is required but is not injected by any @is directive.
        See schema at 33:3:
        productBatch(lookup: Lookup! @composite__is(graph: EXT, field: "{ id }"), required: Boolean!): Product! @composite__lookup(graph: EXT) @extension__directive(graph: EXT, extension: ECHO, name: "echo", arguments: {}) @join__field(graph: EXT)
        "#);
    })
}
