use graphql_mocks::{EchoSchema, Schema as _};
use integration_tests::{gateway::Gateway, runtime};

#[test]
fn authentication_with_public_metadata() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(EchoSchema.with_sdl(
                r#"
                type Query {
                    header(name: String): String
                }
                "#,
            ))
            .with_extension("auth-017")
            .with_toml_config(
                r#"
                [extensions.auth-017.config]
                header_name = "auth017"
                oauth.resource = "https://my-domain.example.com"
                "#,
            )
            .build()
            .await;

        let response = engine.post(r#"query { header(name: "token") }"#).await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "errors": [
            {
              "message": "Not passing through on my watch! SDK-017",
              "extensions": {
                "code": "UNAUTHENTICATED"
              }
            }
          ]
        }
        "#);

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
