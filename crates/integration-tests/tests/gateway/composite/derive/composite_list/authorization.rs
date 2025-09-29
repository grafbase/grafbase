use graphql_mocks::dynamic::DynamicSchema;
use integration_tests::{
    gateway::{AuthorizationExt, Gateway},
    runtime,
};
use serde_json::json;

use crate::gateway::extensions::authorization::DenySites;

#[test]
fn error_on_derived_field() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                    extend schema
                        @link(url: "authorization", import: ["@auth"])
                        @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@key", "@derive"])

                    type Query {
                        product: Product!
                    }

                    type Product {
                        id: ID!
                        commentIds: [ID!]!
                        comments: [Comment!]! @derive
                    }

                    type Comment @key(fields: "id") {
                        id: ID! @auth
                    }
                "#,
                )
                .with_resolver(
                    "Query",
                    "product",
                    json!({"id": "product_1", "commentIds": ["c1", "c2"]}),
                )
                .into_subgraph("x"),
            )
            .with_extension(AuthorizationExt::new(DenySites::query(vec!["Comment.id"])))
            .build()
            .await;

        let response = engine.post("query { product { id comments { id } } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": null,
          "errors": [
            {
              "message": "Unauthorized at query stage",
              "locations": [
                {
                  "line": 1,
                  "column": 33
                }
              ],
              "path": [
                "product",
                "comments",
                0,
                "id"
              ],
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "#);
    })
}

#[test]
fn error_on_derived_entity() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                    extend schema
                        @link(url: "authorization", import: ["@auth"])
                        @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@key", "@derive"])

                    type Query {
                        product: Product!
                    }

                    type Product {
                        id: ID!
                        commentIds: [ID!]!
                        comments: [Comment!]! @derive @auth
                    }

                    type Comment @key(fields: "id") {
                        id: ID!
                    }
                "#,
                )
                .with_resolver(
                    "Query",
                    "product",
                    json!({"id": "product_1", "commentIds": ["c1", "c2"]}),
                )
                .into_subgraph("x"),
            )
            .with_extension(AuthorizationExt::new(DenySites::query(vec!["Product.comments"])))
            .build()
            .await;

        let response = engine.post("query { product { id comments { id } } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": null,
          "errors": [
            {
              "message": "Unauthorized at query stage",
              "locations": [
                {
                  "line": 1,
                  "column": 22
                }
              ],
              "path": [
                "product",
                "comments"
              ],
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "#);
    })
}
