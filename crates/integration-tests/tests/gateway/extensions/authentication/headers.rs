use graphql_mocks::{EchoSchema, Schema as _};
use integration_tests::{
    gateway::{AuthorizationExt, Gateway},
    runtime,
};

use crate::gateway::extensions::authorization::InsertTokenAsHeader;

#[test]
fn authorization_failure_with_response_header_from_extension() {
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
            .with_extension("auth-17")
            .with_toml_config(
                r#"
                [extensions.auth-17.config]
                header_name = "auth17"
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

        insta::assert_debug_snapshot!(response.headers, @r#"
        {
            "content-type": "application/json",
            "www-authenticate": "Bearer test_author=grafbase",
            "content-length": "106",
            "vary": "accept-encoding",
            "vary": "origin, access-control-request-method, access-control-request-headers",
            "access-control-allow-origin": "*",
            "access-control-expose-headers": "*",
        }
        "#);

        let response = engine
            .post(r#"query { header(name: "token") }"#)
            .header("auth17", "valid")
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
