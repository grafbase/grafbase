use engine::Engine;
use graphql_mocks::{EchoSchema, Schema as _};
use integration_tests::{federation::EngineExt, runtime};

use crate::federation::extensions::authorization::{AuthorizationExt, InsertTokenAsHeader};

#[test]
fn no_extension() {
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
    });
}

#[test]
fn no_extension_with_anonymous_default() {
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
            .with_toml_config(
                r#"
                [authentication]
                default = "anonymous"
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
    });
}

#[test]
fn no_extension_with_deny_default() {
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
            .with_toml_config(
                r#"
                [authentication]
                default = "deny"
                "#,
            )
            .build()
            .await;

        let response = engine.post(r#"query { header(name: "token") }"#).await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "errors": [
            {
              "message": "Unauthenticated",
              "extensions": {
                "code": "UNAUTHENTICATED"
              }
            }
          ]
        }
        "#);

        let sent = engine.drain_graphql_requests_sent_to_by_name("subgraph");
        insta::assert_json_snapshot!(sent, @"[]");
    });
}
