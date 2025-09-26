use graphql_mocks::{EchoSchema, Schema as _};
use integration_tests::{
    gateway::{AuthorizationExt, Gateway},
    runtime,
};

use crate::gateway::extensions::authorization::InsertTokenAsHeader;

#[test]
fn unauthorized() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(EchoSchema::default().with_sdl(
                r#"
                extend schema @link(url: "authorization", import: ["@auth"])

                type Query {
                    header(name: String): String @auth
                }
                "#,
            ))
            .with_extension(AuthorizationExt::new(InsertTokenAsHeader))
            .with_extension("auth-10")
            .build()
            .await;

        let response = engine.post(r#"query { header(name: "token") }"#).await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "errors": [
            {
              "message": "Not passing through on my watch! SDK-10",
              "extensions": {
                "code": "UNAUTHENTICATED"
              }
            }
          ]
        }
        "#);

        let sent = engine.drain_graphql_requests_sent_to_by_name("subgraph");
        insta::assert_json_snapshot!(sent, @"[]");

        let response = engine
            .post(r#"query { header(name: "token") }"#)
            .header("Authorization", "valid")
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "header": "sdk10:valid:default"
          }
        }
        "#);
    });
}

#[test]
fn config() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(EchoSchema::default().with_sdl(
                r#"
                extend schema @link(url: "authorization", import: ["@auth"])

                type Query {
                    header(name: String): String @auth
                }
                "#,
            ))
            .with_extension(AuthorizationExt::new(InsertTokenAsHeader))
            .with_extension("auth-10")
            .with_toml_config(
                r#"
                [extensions.auth-10.config]
                header_name = "auth"
                "#,
            )
            .build()
            .await;

        let response = engine.post(r#"query { header(name: "token") }"#).await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "errors": [
            {
              "message": "Not passing through on my watch! SDK-10",
              "extensions": {
                "code": "UNAUTHENTICATED"
              }
            }
          ]
        }
        "#);

        let sent = engine.drain_graphql_requests_sent_to_by_name("subgraph");
        insta::assert_json_snapshot!(sent, @"[]");

        let response = engine
            .post(r#"query { header(name: "token") }"#)
            .header("auth", "valid")
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "header": "sdk10:valid:default"
          }
        }
        "#);
    });
}

#[test]
fn reads_headers_and_cache_key() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(EchoSchema::default().with_sdl(
                r#"
                extend schema @link(url: "authorization", import: ["@auth"])

                type Query {
                    header(name: String): String @auth
                }
                "#,
            ))
            .with_extension(AuthorizationExt::new(InsertTokenAsHeader))
            .with_extension("auth-10")
            .build()
            .await;

        let response = engine
            .post(r#"query { header(name: "token") }"#)
            .header("Authorization", "valid")
            .header("key", "Hi!")
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "header": "sdk10:valid:Hi!"
          }
        }
        "#);

        // Key is already cached
        let response = engine
            .post(r#"query { header(name: "token") }"#)
            .header("Authorization", "valid")
            .header("key", "Something else")
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "header": "sdk10:valid:Hi!"
          }
        }
        "#);
    });
}
