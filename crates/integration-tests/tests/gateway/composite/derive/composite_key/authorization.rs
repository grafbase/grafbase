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
                        @link(url: "authorization-1.0.0", import: ["@auth"])
                        @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@key", "@derive"])

                    type Query {
                        post: Post!
                    }

                    type Post {
                        id: ID!
                        authorId: ID!
                        authorX: ID!
                        author: User! @derive
                    }

                    type User @key(fields: "id x") {
                        id: ID! @auth
                        x: ID!
                    }
                "#,
                )
                .with_resolver(
                    "Query",
                    "post",
                    json!({"id": "post_1", "authorId": "user_1", "authorX": "user_x"}),
                )
                .into_subgraph("x"),
            )
            .with_extension(AuthorizationExt::new(DenySites::query(vec!["User.id"])))
            .build()
            .await;

        let response = engine.post("query { post { id author { id } } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": null,
          "errors": [
            {
              "message": "Unauthorized at query stage",
              "locations": [
                {
                  "line": 1,
                  "column": 28
                }
              ],
              "path": [
                "post",
                "author",
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
                        @link(url: "authorization-1.0.0", import: ["@auth"])
                        @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@key", "@derive"])

                    type Query {
                        post: Post!
                    }

                    type Post {
                        id: ID!
                        authorId: ID!
                        authorX: ID!
                        author: User! @derive @auth
                    }

                    type User @key(fields: "id x") {
                        id: ID!
                        x: ID!
                    }
                "#,
                )
                .with_resolver(
                    "Query",
                    "post",
                    json!({"id": "post_1", "authorId": "user_1", "authorX": "user_x"}),
                )
                .into_subgraph("x"),
            )
            .with_extension(AuthorizationExt::new(DenySites::query(vec!["Post.author"])))
            .build()
            .await;

        let response = engine.post("query { post { id author { id } } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": null,
          "errors": [
            {
              "message": "Unauthorized at query stage",
              "locations": [
                {
                  "line": 1,
                  "column": 19
                }
              ],
              "path": [
                "post",
                "author"
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
