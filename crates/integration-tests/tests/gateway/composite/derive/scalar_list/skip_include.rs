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
                comments {
                    id @include(if: $include)
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
              "comments": [
                {},
                {}
              ]
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
                comments @include(if: $include) {
                    id
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
                commentIds  @include(if: $include)
                comments {
                    id
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
