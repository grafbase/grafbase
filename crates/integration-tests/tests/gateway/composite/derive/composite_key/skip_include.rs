use integration_tests::{gateway::Gateway, runtime};
use serde_json::json;

use super::gql_subgraph;

#[test]
fn include_derived_field() {
    runtime().block_on(async {
        let engine = Gateway::builder().with_subgraph(gql_subgraph()).build().await;

        let query = r#"
        query($include: Boolean!) {
            post {
                id
                author {
                    id @include(if: $include)
                    x @skip(if: $include)
                }
            }
        }"#;

        let response = engine
            .post(query)
            .variables(json!({
                "include": true
            }))
            .await;
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

        let response = engine
            .post(query)
            .variables(json!({
                "include": false
            }))
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "post": {
              "id": "post_1",
              "author": {
                "x": "user_x"
              }
            }
          }
        }
        "#);
    })
}

#[test]
fn include_derived_entity() {
    runtime().block_on(async {
        let engine = Gateway::builder().with_subgraph(gql_subgraph()).build().await;

        let query = r#"
        query($include: Boolean!) {
            post {
                id
                author @include(if: $include) {
                    id
                    x
                }
            }
        }"#;

        let response = engine
            .post(query)
            .variables(json!({
                "include": true
            }))
            .await;
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

        let response = engine
            .post(query)
            .variables(json!({
                "include": false
            }))
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "post": {
              "id": "post_1"
            }
          }
        }
        "#);
    })
}

#[test]
fn include_original_field() {
    runtime().block_on(async {
        let engine = Gateway::builder().with_subgraph(gql_subgraph()).build().await;

        let query = r#"
        query($include: Boolean!) {
            post {
                id
                authorId @include(if: $include)
                authorX @skip(if: $include)
                author {
                    id
                    x
                }
            }
        }"#;

        let response = engine
            .post(query)
            .variables(json!({
                "include": true
            }))
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "post": {
              "id": "post_1",
              "authorId": "user_1",
              "author": {
                "id": "user_1",
                "x": "user_x"
              }
            }
          }
        }
        "#);

        let response = engine
            .post(query)
            .variables(json!({
                "include": false
            }))
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "post": {
              "id": "post_1",
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
