use engine::Engine;
use graphql_mocks::{EchoSchema, Schema as _};
use integration_tests::{federation::EngineExt, runtime};

use crate::federation::extensions::authorization::{AuthorizationExt, InsertTokenAsHeader};

#[test]
fn sdk_080() {
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
            .with_toml_config(
                r#"
                [[authentication.providers]]

                [authentication.providers.extension]
                extension = "auth-08"
                config = { cache_config = "test" }

                [extensions.auth-08]
                version = "1.0.0"
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
            .header("Authorization", "valid")
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "header": "�ckeygdefault"
          }
        }
        "#);

        let response = engine
            .post(r#"query { header(name: "token") }"#)
            .header("Authorization", "valid")
            .header("value", "Hi!")
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "header": "�ckeygdefault"
          }
        }
        "#);
    });
}

#[test]
fn sdk_090() {
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
            .with_extension("auth-09")
            .with_toml_config(
                r#"
                [[authentication.providers]]

                [authentication.providers.extension]
                extension = "auth-09"
                config = { cache_config = "test" }

                [extensions.auth-09]
                version = "1.0.0"
                "#,
            )
            .build()
            .await;

        let response = engine.post(r#"query { header(name: "token") }"#).await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "errors": [
            {
              "message": "Not passing through on my watch! SDK-09",
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
            "header": "{\"key\":\"default\"}"
          }
        }
        "#);

        let response = engine
            .post(r#"query { header(name: "token") }"#)
            .header("Authorization", "valid")
            .header("value", "Hi!")
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "header": "{\"key\":\"default\"}"
          }
        }
        "#);
    });
}
