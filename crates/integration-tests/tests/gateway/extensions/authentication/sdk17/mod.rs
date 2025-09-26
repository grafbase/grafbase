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
            .with_extension("auth-17")
            .build()
            .await;

        let response = engine.post(r#"query { header(name: "token") }"#).await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "errors": [
            {
              "message": "Not passing through on my watch! SDK-17",
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
            "header": "sdk17:valid:default"
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
            .with_extension("auth-17")
            .with_toml_config(
                r#"
                [extensions.auth-17.config]
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
              "message": "Not passing through on my watch! SDK-17",
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
            "header": "sdk17:valid:default"
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
            .with_extension("auth-17")
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
            "header": "sdk17:valid:Hi!"
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
            "header": "sdk17:valid:Hi!"
          }
        }
        "#);
    });
}

#[test]
fn authentication_with_public_metadata() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(EchoSchema::default().with_sdl(
                r#"
                type Query {
                    header(name: String): String
                }
                "#,
            ))
            .with_extension("auth-17")
            .with_toml_config(
                r#"
                [extensions.auth-17.config]
                header_name = "auth17"
                oauth.resource = "https://my-domain.example.com"
                "#,
            )
            .build()
            .await;

        let metadata_request = http::Request::builder()
            .method(http::Method::GET)
            .uri("/.well-known/protected-resource")
            .body(axum::body::Body::empty())
            .unwrap();

        let metadata_response = engine.raw_execute(metadata_request).await;

        insta::assert_debug_snapshot!(metadata_response, @r#"
        Response {
            status: 200,
            version: HTTP/1.1,
            headers: {
                "x-test": "works",
                "vary": "accept-encoding",
                "vary": "origin, access-control-request-method, access-control-request-headers",
                "access-control-allow-origin": "*",
                "access-control-expose-headers": "*",
                "content-length": "44",
            },
            body: b"{\"resource\":\"https://my-domain.example.com\"}",
        }
        "#)
    });
}
