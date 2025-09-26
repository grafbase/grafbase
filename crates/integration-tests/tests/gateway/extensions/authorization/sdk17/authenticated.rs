use graphql_mocks::dynamic::DynamicSchema;
use integration_tests::{
    gateway::{AuthenticationExt, Gateway},
    runtime,
};

use crate::gateway::extensions::authentication::static_auth::StaticAuth;

#[test]
fn can_load_authenticated_extension() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                extend schema @link(url: "authenticated-17", import: ["@authenticated"])

                type Query {
                    private: String @authenticated
                    public: String
                }
                "#,
                )
                .with_resolver("Query", "private", serde_json::Value::String("Oh no!".to_owned()))
                .with_resolver("Query", "public", serde_json::Value::String("Hi!".to_owned()))
                .into_subgraph("x"),
            )
            .with_extension("authenticated-17")
            .build()
            .await;

        let response = engine.post("query { public private }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "public": "Hi!",
            "private": null
          },
          "errors": [
            {
              "message": "Not authenticated",
              "locations": [
                {
                  "line": 1,
                  "column": 16
                }
              ],
              "path": [
                "private"
              ],
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "#);

        let sent = engine.drain_graphql_requests_sent_to_by_name("x");
        insta::assert_json_snapshot!(sent, @r#"
        [
          {
            "query": "query { public }",
            "operationName": null,
            "variables": {},
            "extensions": {}
          }
        ]
        "#)
    });
}

#[test]
fn can_access_token() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                extend schema @link(url: "authenticated-17", import: ["@authenticated"])

                type Query {
                    private: String @authenticated
                    public: String
                }
                "#,
                )
                .with_resolver("Query", "private", serde_json::Value::String("Oh no!".to_owned()))
                .with_resolver("Query", "public", serde_json::Value::String("Hi!".to_owned()))
                .into_subgraph("x"),
            )
            .with_extension(AuthenticationExt::new(StaticAuth::bytes(Vec::new())))
            .with_extension("authenticated-17")
            .with_toml_config(
                r#"
            [extensions."authenticated-17"]
            stdout = true
            stderr = true
            "#,
            )
            .build()
            .await;

        let response = engine.post("query { public private }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "public": "Hi!",
            "private": "Oh no!"
          }
        }
        "#);
    });
}
