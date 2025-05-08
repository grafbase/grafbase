mod authorization;
mod null;
mod skip_include;

use graphql_mocks::dynamic::{DynamicSchema, DynamicSubgraph};
use integration_tests::{gateway::Gateway, runtime};
use serde_json::json;

fn gql_subgraph() -> DynamicSubgraph {
    DynamicSchema::builder(
        r#"
            extend schema
                @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@is"])

            type Query {
                post: Post!
            }

            type Post {
                id: ID!
                author_id: ID!
                author: User! @is(field: "{ id: author_id }")
            }

            type User {
                id: ID!
            }
            "#,
    )
    .with_resolver("Query", "post", json!({"id": "post_1", "author_id": "user_1"}))
    .into_subgraph("x")
}

#[test]
fn basic() {
    runtime().block_on(async {
        let engine = Gateway::builder().with_subgraph(gql_subgraph()).build().await;

        let response = engine.post("query { post { id author { id } } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "post": {
              "id": "post_1",
              "author": {
                "id": "user_1"
              }
            }
          }
        }
        "#);
    })
}

#[test]
fn composite_keys() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                extend schema
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@is"])

                type Query {
                    post: Post!
                }

                type Post {
                    id: ID!
                    author_id: ID!
                    author_x: ID!
                    author: User! @is(field: "{ id: author_id x: author_x}")
                }

                type User {
                    id: ID!
                    x: ID!
                }
                "#,
                )
                .with_resolver(
                    "Query",
                    "post",
                    json!({"id": "post_1", "author_id": "user_1", "author_x": "user_x"}),
                )
                .into_subgraph("x"),
            )
            .build()
            .await;

        let response = engine.post("query { post { id author { id x } } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "post": {
              "id": "post_1",
              "author": {
                "id": "user_1",
                "x": "user_x"
              }
            }
          }
        }
        "#);
    })
}

#[test]
fn typename() {
    runtime().block_on(async {
        let engine = Gateway::builder().with_subgraph(gql_subgraph()).build().await;

        let response = engine.post("query { post { id author { __typename id } } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "post": {
              "id": "post_1",
              "author": {
                "__typename": "User",
                "id": "user_1"
              }
            }
          }
        }
        "#);
    })
}

#[test]
fn both_derived_and_direct_field() {
    runtime().block_on(async {
        let engine = Gateway::builder().with_subgraph(gql_subgraph()).build().await;

        let response = engine.post("query { post { id author_id author { id } } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "post": {
              "id": "post_1",
              "author_id": "user_1",
              "author": {
                "id": "user_1"
              }
            }
          }
        }
        "#);
    })
}
