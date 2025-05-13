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
            authorId: ID!
            authorX: ID!
            author: User! @derive
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
    .into_subgraph("x")
}

#[test]
fn basic() {
    runtime().block_on(async {
        let engine = Gateway::builder().with_subgraph(gql_subgraph()).build().await;

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
fn underscore_case_insensitive_mapping() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
            extend schema
                @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@derive", "@key", "@is"])

            type Query {
                post: Post!
            }

            type Post {
                id: ID!
                author_i_D: ID!
                author_x: ID!
                author: User! @derive
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
                    json!({"id": "post_1", "author_i_D": "user_1", "author_x": "user_x"}),
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

        let response = engine.post("query { post { id author { __typename id x } } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "post": {
              "id": "post_1",
              "author": {
                "__typename": "User",
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
fn aliases() {
    runtime().block_on(async {
        let engine = Gateway::builder().with_subgraph(gql_subgraph()).build().await;

        let response = engine
            .post("query { post { id a: author { t: __typename i: id y: x } } }")
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "post": {
              "id": "post_1",
              "a": {
                "t": "User",
                "i": "user_1",
                "y": "user_x"
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

        let response = engine
            .post("query { post { id authorId authorX author { id x } } }")
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "post": {
              "id": "post_1",
              "authorId": "user_1",
              "authorX": "user_x",
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
