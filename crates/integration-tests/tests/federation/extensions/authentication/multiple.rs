use engine::Engine;
use graphql_mocks::{EchoSchema, Schema as _};
use integration_tests::{federation::EngineExt, runtime};

use crate::federation::extensions::authorization::{AuthorizationExt, InsertTokenAsHeader};

#[test]
fn double_authentication() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(EchoSchema.with_sdl(
                r#"
                extend schema @link(url: "authorization-1.0.0", import: ["@auth"])

                type Query {
                    header(name: String): String @auth
                }
                "#,
            ))
            .with_extension(AuthorizationExt::new(InsertTokenAsHeader))
            .with_extension("auth-08")
            .with_extension("auth-09")
            .with_toml_config(
                r#"
                [extensions.auth-08.config]
                header_name = "auth08"

                [extensions.auth-09.config]
                header_name = "auth09"
                "#,
            )
            .build()
            .await;

        let response = engine.post(r#"query { header(name: "token") }"#).await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "errors": [
            {
              "message": "Not passing through on my watch! SDK-08",
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
            .header("auth08", "valid")
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "header": "ssdk08:valid:default"
          }
        }
        "#);

        let response = engine
            .post(r#"query { header(name: "token") }"#)
            .header("auth09", "valid")
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "header": "sdk09:valid:default"
          }
        }
        "#);

        let response = engine
            .post(r#"query { header(name: "token") }"#)
            .header("auth09", "valid")
            .header("auth08", "valid")
            .await;
        insta::assert_json_snapshot!(response, { ".data.header" => "ANYTHING" }, @r#"
        {
          "data": {
            "header": "ANYTHING"
          }
        }
        "#);
    });
}

#[test]
fn double_authentication_with_deny_default() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(EchoSchema.with_sdl(
                r#"
                extend schema @link(url: "authorization-1.0.0", import: ["@auth"])

                type Query {
                    header(name: String): String @auth
                }
                "#,
            ))
            .with_extension(AuthorizationExt::new(InsertTokenAsHeader))
            .with_extension("auth-08")
            .with_extension("auth-09")
            .with_toml_config(
                r#"
                [authentication]
                default = "deny"

                [extensions.auth-08.config]
                header_name = "auth08"

                [extensions.auth-09.config]
                header_name = "auth09"
                "#,
            )
            .build()
            .await;

        let response = engine.post(r#"query { header(name: "token") }"#).await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "errors": [
            {
              "message": "Not passing through on my watch! SDK-08",
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
            .header("auth08", "valid")
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "header": "ssdk08:valid:default"
          }
        }
        "#);

        let response = engine
            .post(r#"query { header(name: "token") }"#)
            .header("auth09", "valid")
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "header": "sdk09:valid:default"
          }
        }
        "#);
    });
}

#[test]
fn double_authentication_with_anonymous_default() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(EchoSchema.with_sdl(
                r#"
                extend schema @link(url: "authorization-1.0.0", import: ["@auth"])

                type Query {
                    header(name: String): String @auth
                }
                "#,
            ))
            .with_extension(AuthorizationExt::new(InsertTokenAsHeader))
            .with_extension("auth-08")
            .with_extension("auth-09")
            .with_toml_config(
                r#"
                [authentication]
                default = "anonymous"

                [extensions.auth-08.config]
                header_name = "auth08"

                [extensions.auth-09.config]
                header_name = "auth09"
                "#,
            )
            .build()
            .await;

        let response = engine.post(r#"query { header(name: "token") }"#).await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "header": ""
          }
        }
        "#);

        let response = engine
            .post(r#"query { header(name: "token") }"#)
            .header("auth08", "valid")
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "header": "ssdk08:valid:default"
          }
        }
        "#);

        let response = engine
            .post(r#"query { header(name: "token") }"#)
            .header("auth09", "valid")
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "header": "sdk09:valid:default"
          }
        }
        "#);
    });
}
