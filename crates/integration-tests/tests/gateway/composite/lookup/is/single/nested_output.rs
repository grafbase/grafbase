use integration_tests::{gateway::Gateway, runtime};

use super::super::super::{EchoLookup, gql_id};

#[test]
fn single_field() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_id())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is", "@shareable"])


                type Query {
                    productLookup(id: ID! @is(field: "id")): Namespace! @lookup @echo
                }

                type Namespace {
                    product: Product!
                }

                type Product @key(fields: "id") {
                    id: ID!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoLookup::single().namespaced("product"))
            .build()
            .await;

        let response = engine.post("query { products { args } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "args": {
                  "id": "1"
                }
              }
            ]
          }
        }
        "#);
    })
}

#[test]
fn other_unrelated_fields() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_id())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is", "@shareable"])


                type Query {
                    productLookup(id: ID! @is(field: "id")): Namespace! @lookup @echo
                }

                type Namespace {
                    product: Product!
                    other: String
                    anything: JSON
                }

                type Product @key(fields: "id") {
                    id: ID!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoLookup::single().namespaced("product"))
            .build()
            .await;

        let response = engine.post("query { products { args } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "args": {
                  "id": "1"
                }
              }
            ]
          }
        }
        "#);
    })
}

#[test]
fn invalid_namespace_key_at_runtime() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_id())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is", "@shareable"])


                type Query {
                    productLookup(id: ID! @is(field: "id")): Namespace! @lookup @echo
                }

                type Namespace {
                    product: Product!
                }

                type Product @key(fields: "id") {
                    id: ID!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoLookup::single().namespaced("wrong key"))
            .build()
            .await;

        let response = engine.post("query { products { args } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "args": null
              }
            ]
          }
        }
        "#);
    })
}

#[test]
fn nested_but_unknown_fields() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph(gql_id())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is", "@shareable"])


                type Query {
                    productLookup(id: ID! @is(field: "id")): Namespace! @lookup @echo
                }

                type Namespace {
                    other: String
                    anything: JSON
                }

                type Product @key(fields: "id") {
                    id: ID!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoLookup::single().namespaced("product"))
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Query.productLookup, for directive @lookup Type Namespace doesn't define any keys with @key directive that may be used for @lookup. Tried treating it as a namespace type, but it didn't have any fields that may be used for @lookup.
        30 | 
        31 | type Namespace
             ^^^^^^^^^^^^^^
        32 |   @join__type(graph: EXT)
        "#);
    })
}

#[test]
fn multiple_entities() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph(gql_id())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "echo", import: ["@echo"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key", "@is", "@shareable"])


                type Query {
                    productLookup(id: ID! @is(field: "id")): Namespace! @lookup @echo
                }

                type Namespace {
                    product: Product!
                    account: Account!
                }

                type Product @key(fields: "id") {
                    id: ID!
                    args: JSON
                }

                type Account @key(fields: "id") {
                    id: ID!
                    args: JSON
                }

                scalar JSON
                "#,
            )
            .with_extension(EchoLookup::single().namespaced("product"))
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Query.productLookup, for directive @lookup Type Namespace doesn't define any keys with @key directive that may be used for @lookup. Tried treating it as a namespace type, but it has multiple fields that may be used for @lookup: product and account
        30 | 
        31 | type Namespace
             ^^^^^^^^^^^^^^
        32 |   @join__type(graph: EXT)
        "#);
    })
}
