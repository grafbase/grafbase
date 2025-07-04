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
                    productBatch(input: Lookup! @is(field: "{ ids: [id] }")): [Product!]! @lookup @echo
                }

                input Lookup @oneOf {
                    ids: [ID!]
                }

                type Product @key(fields: "id") {
                    id: ID!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoLookup { batch: true })
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
                    "ids": [
                      "1"
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
                    productBatch(input: Lookup! @is(field: "{ ids: [id] } | { names: [name] }")): [Product!]! @lookup @echo
                }

                input Lookup @oneOf {
                    ids: [ID!]
                    names: [String!]
                }

                type Product @key(fields: "id") @key(fields: "name") {
                    id: ID!
                    name: String!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoLookup { batch: true })
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
                    "ids": [
                      "1"
                    ]
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
                    "names": [
                      "name1"
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
                    productBatch(input: Lookup @is(field: "{ ids: [id] }")): [Product!]! @lookup @echo
                }

                input Lookup @oneOf {
                    ids: [ID!]
                }

                type Product @key(fields: "id") {
                    id: ID!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoLookup { batch: true })
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
                    "ids": [
                      "1"
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
                    productBatch(input: Lookup! @is(field: "{ ids: [id] }")): [Product!]! @lookup @echo
                }

                input Lookup @oneOf {
                    ids: [ID]
                }

                type Product @key(fields: "id") {
                    id: ID!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoLookup { batch: true })
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
                    "ids": [
                      "1"
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
                    productBatch(input: Lookup! @is(field: "{ ids: [id] }")): [Product!]! @lookup @echo
                }

                input Lookup @oneOf {
                    ids: [ID!]
                    anything: [ID!]
                }

                type Product @key(fields: "id") {
                    id: ID!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoLookup { batch: true })
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
                    "ids": [
                      "1"
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
                    productBatch(input: Lookup! @is(field: "{ ids: [id] }")): [Product!]! @lookup @echo
                }

                input Lookup @oneOf {
                    ids: [ID!]
                    id: ID
                }

                type Product @key(fields: "id") {
                    id: ID!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoLookup { batch: true })
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
                    "ids": [
                      "1"
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
fn invalid_batch() {
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
                    productBatch(id: Lookup! @is(field: "{ ids: id }")): [Product!]! @lookup @echo
                }

                input Lookup @oneOf {
                    ids: ID
                }

                type Product @key(fields: "id") {
                    id: ID!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoLookup { batch: true })
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Query.productBatch, for directive @lookup for associated @is directive: Cannot select a field from [Product!]!, it's a list
        See schema at 33:42:
        (graph: EXT, field: "{ ids: id }")
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
                    productBatch(a: Lookup!  @is(field: "{ ids: [id] }"), b: Lookup!  @is(field: "{ ids: [id] }")): [Product!]! @lookup @echo
                }

                input Lookup @oneOf {
                    ids: [ID!]
                }

                type Product @key(fields: "id") {
                    id: ID!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoLookup { batch: true })
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Query.productBatch, for directive @lookup With a @oneOf argument, only one @is directive is supported for @lookup.
        See schema at 33:3:
        productBatch(a: Lookup! @composite__is(graph: EXT, field: "{ ids: [id] }"), b: Lookup! @composite__is(graph: EXT, field: "{ ids: [id] }")): [Product!]! @composite__lookup(graph: EXT) @extension__directive(graph: EXT, extension: ECHO, name: "echo", arguments: {}) @join__field(graph: EXT)
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
                    productBatch(lookup: Lookup! @is(field: "{ ids: [id] }"), required: Boolean!): [Product!]! @lookup @echo
                }

                input Lookup @oneOf {
                    ids: [ID!]
                }

                type Product @key(fields: "id") {
                    id: ID!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoLookup { batch: true })
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Query.productBatch, for directive @lookup Argument 'required' is required but is not injected by any @is directive.
        See schema at 33:3:
        productBatch(lookup: Lookup! @composite__is(graph: EXT, field: "{ ids: [id] }"), required: Boolean!): [Product!]! @composite__lookup(graph: EXT) @extension__directive(graph: EXT, extension: ECHO, name: "echo", arguments: {}) @join__field(graph: EXT)
        "#);
    })
}
