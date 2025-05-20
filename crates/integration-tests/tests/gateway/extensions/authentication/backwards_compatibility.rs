use graphql_mocks::{EchoSchema, Schema as _};
use integration_tests::{
    gateway::{AuthorizationExt, Gateway},
    runtime,
};

use crate::gateway::extensions::authorization::InsertTokenAsHeader;

#[test]
fn sdk_090() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
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
                [extensions.auth-09.config]
                cache_key_prefix = "test"
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
            "header": "sdk09:valid:default"
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
            "header": "sdk09:valid:default"
          }
        }
        "#);
    });
}
