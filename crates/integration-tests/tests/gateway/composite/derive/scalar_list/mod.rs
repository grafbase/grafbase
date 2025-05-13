mod authorization;
mod is;
mod join;
mod skip_include;

use graphql_mocks::dynamic::{DynamicSchema, DynamicSubgraph};
use integration_tests::{gateway::Gateway, runtime};
use serde_json::json;

fn gql_subgraph() -> DynamicSubgraph {
    DynamicSchema::builder(
        r#"
            extend schema
                @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@derive", "@key"])

            type Query {
                post: Post!
            }

            type Post {
                id: ID!
                commentIds: [ID!]!
                comments: [Comment!]! @derive
            }

            type Comment @key(fields: "id") {
                id: ID!
            }
            "#,
    )
    .with_resolver("Query", "post", json!({"id": "post_1", "commentIds": ["c1", "c2"]}))
    .into_subgraph("x")
}

#[test]
fn basic() {
    runtime().block_on(async {
        let engine = Gateway::builder().with_subgraph(gql_subgraph()).build().await;

        let response = engine.post("query { post { id comments { id } } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "post": {
              "id": "post_1",
              "comments": [
                {
                  "id": "c1"
                },
                {
                  "id": "c2"
                }
              ]
            }
          }
        }
        "#);
    })
}

#[test]
fn both_derive_and_original_field() {
    runtime().block_on(async {
        let engine = Gateway::builder().with_subgraph(gql_subgraph()).build().await;

        let response = engine.post("query { post { id commentIds comments { id } } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "post": {
              "id": "post_1",
              "commentIds": [
                "c1",
                "c2"
              ],
              "comments": [
                {
                  "id": "c1"
                },
                {
                  "id": "c2"
                }
              ]
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

        let response = engine.post("query { post { id comments { __typename id } } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "post": {
              "id": "post_1",
              "comments": [
                {
                  "__typename": "Comment",
                  "id": "c1"
                },
                {
                  "__typename": "Comment",
                  "id": "c2"
                }
              ]
            }
          }
        }
        "#);
    })
}

#[test]
fn aliases() {
    runtime().block_on(async {
        let engine = Gateway::builder().with_subgraph(gql_subgraph()).build().await;

        let response = engine
            .post("query { post { id c: comments { t: __typename i: id } } }")
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "post": {
              "id": "post_1",
              "c": [
                {
                  "t": "Comment",
                  "i": "c1"
                },
                {
                  "t": "Comment",
                  "i": "c2"
                }
              ]
            }
          }
        }
        "#);
    })
}

#[test]
fn snake_case() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
            extend schema
                @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@derive", "@key"])

            type Query {
                post: Post!
            }

            type Post {
                id: ID!
                comment_ids: [ID!]!
                comments: [Comment!]! @derive
            }

            type Comment @key(fields: "id") {
                id: ID!
            }
            "#,
                )
                .with_resolver("Query", "post", json!({"id": "post_1", "comment_ids": ["c1", "c2"]}))
                .into_subgraph("x"),
            )
            .build()
            .await;

        let response = engine.post("query { post { id comments { id } } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "post": {
              "id": "post_1",
              "comments": [
                {
                  "id": "c1"
                },
                {
                  "id": "c2"
                }
              ]
            }
          }
        }
        "#);
    })
}
