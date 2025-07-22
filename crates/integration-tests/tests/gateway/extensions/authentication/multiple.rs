use graphql_mocks::{EchoSchema, Schema as _};
use integration_tests::{
    gateway::{AuthorizationExt, Gateway},
    runtime,
};

use crate::gateway::extensions::authorization::InsertTokenAsHeader;

#[test]
fn double_authentication() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(EchoSchema::default().with_sdl(
                r#"
                extend schema @link(url: "authorization-1.0.0", import: ["@auth"])

                type Query {
                    header(name: String): String @auth
                }
                "#,
            ))
            .with_extension(AuthorizationExt::new(InsertTokenAsHeader))
            .with_extension("auth-10")
            .with_extension("auth-15")
            .with_toml_config(
                r#"
                [extensions.auth-10.config]
                header_name = "auth09"

                [extensions.auth-15.config]
                header_name = "auth15"
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
            .header("auth15", "valid")
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "header": "sdk15:valid:default"
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
            "header": "sdk10:valid:default"
          }
        }
        "#);

        let response = engine
            .post(r#"query { header(name: "token") }"#)
            .header("auth09", "valid")
            .header("auth15", "valid")
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
        let engine = Gateway::builder()
            .with_subgraph(EchoSchema::default().with_sdl(
                r#"
                extend schema @link(url: "authorization-1.0.0", import: ["@auth"])

                type Query {
                    header(name: String): String @auth
                }
                "#,
            ))
            .with_extension(AuthorizationExt::new(InsertTokenAsHeader))
            .with_extension("auth-10")
            .with_extension("auth-15")
            .with_toml_config(
                r#"
                [authentication]
                default = "deny"

                [extensions.auth-15.config]
                header_name = "auth15"

                [extensions.auth-10.config]
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
            .header("auth15", "valid")
            .await;

        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "header": "sdk15:valid:default"
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
            "header": "sdk10:valid:default"
          }
        }
        "#);
    });
}

#[test]
fn double_authentication_with_anonymous_default() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(EchoSchema::default().with_sdl(
                r#"
                extend schema @link(url: "authorization-1.0.0", import: ["@auth"])

                type Query {
                    header(name: String): String @auth
                }
                "#,
            ))
            .with_extension(AuthorizationExt::new(InsertTokenAsHeader))
            .with_extension("auth-10")
            .with_extension("auth-15")
            .with_toml_config(
                r#"
                [authentication]
                default = "anonymous"

                [extensions.auth-10.config]
                header_name = "auth09"

                [extensions.auth-15.config]
                header_name = "auth15"
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
            .header("auth09", "valid")
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "header": "sdk10:valid:default"
          }
        }
        "#);

        let response = engine
            .post(r#"query { header(name: "token") }"#)
            .header("auth15", "valid")
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "header": "sdk15:valid:default"
          }
        }
        "#);
    });
}
