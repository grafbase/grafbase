use graphql_mocks::dynamic::DynamicSchema;
use integration_tests::{gateway::Gateway, runtime};
use serde_json::json;

use crate::gateway::extensions::selection_set_resolver::StaticSelectionSetResolverExt;

#[test]
fn required_entity_nullable_field_null_entity() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                        type Query {
                            products: [Product!]!
                        }

                        type Product @key(fields: "id") {
                            id: ID!
                        }
                    "#,
                )
                .with_resolver("Query", "products", json!([{"id": "1"}, {"id": "2"}]))
                .into_subgraph("gql"),
            )
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])
                    @init

                type Query {
                    productBatch(ids: [ID!]!): [Product] @lookup
                }

                type Product @key(fields: "id") {
                    id: ID!
                    code: String
                }
                "#,
            )
            .with_extension(StaticSelectionSetResolverExt::json(json!([{"code": "C1"}, null])))
            .build()
            .await;

        let response = engine.post("query { products { id code } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "id": "1",
                "code": "C1"
              },
              {
                "id": "2",
                "code": null
              }
            ]
          }
        }
        "#);
    })
}

#[test]
fn nullable_entity_required_entity_null_entity() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                        type Query {
                            products: [Product]!
                        }

                        type Product @key(fields: "id") {
                            id: ID!
                        }
                    "#,
                )
                .with_resolver("Query", "products", json!([{"id": "1"}, {"id": "2"}]))
                .into_subgraph("gql"),
            )
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])
                    @init

                type Query {
                    productBatch(ids: [ID!]!): [Product] @lookup
                }

                type Product @key(fields: "id") {
                    id: ID!
                    code: String!
                }
                "#,
            )
            .with_extension(StaticSelectionSetResolverExt::json(json!([{"code": "C1"}, null])))
            .build()
            .await;

        let response = engine.post("query { products { id code } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "id": "1",
                "code": "C1"
              },
              null
            ]
          }
        }
        "#);
    })
}

#[test]
fn required_field_null_entity() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                        type Query {
                            products: [Product!]!
                        }

                        type Product @key(fields: "id") {
                            id: ID!
                        }
                    "#,
                )
                .with_resolver("Query", "products", json!([{"id": "1"}, {"id": "2"}]))
                .into_subgraph("gql"),
            )
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])
                    @init

                type Query {
                    productBatch(ids: [ID!]!): [Product] @lookup
                }

                type Product @key(fields: "id") {
                    id: ID!
                    code: String!
                }
                "#,
            )
            .with_extension(StaticSelectionSetResolverExt::json(json!([{"code": "C1"}, null])))
            .build()
            .await;

        let response = engine.post("query { products { id code } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": null
        }
        "#);
    })
}
