use graphql_mocks::{EchoSchema, Schema as _};
use integration_tests::{
    gateway::{AuthenticationExt, AuthorizationExt, Gateway},
    runtime,
};

use crate::gateway::extensions::{authentication::static_token::StaticToken, authorization::InsertTokenAsHeader};

#[test]
fn deny_default_no_extension_404() {
    runtime().block_on(async move {
        let gateway = Gateway::builder()
            .with_subgraph(EchoSchema.with_sdl(
                r#"
                type Query {
                    header(name: String): String
                }
                "#,
            ))
            .with_toml_config(
                r#"
                [authentication]
                default = "deny"
                "#,
            )
            .build()
            .await;

        let request = http::Request::builder()
            .method(http::Method::GET)
            // Does not exist.
            .uri("/.well-known/oauth-protected-resource/mcp")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = gateway.raw_execute(request).await;

        assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
    });
}

#[test]
fn deny_default_with_extensions_404() {
    runtime().block_on(async move {
        let gateway = Gateway::builder()
            .with_subgraph(EchoSchema.with_sdl(
                r#"
                extend schema @link(url: "authorization-1.0.0", import: ["@auth"])

                type Query {
                    header(name: String): String @auth
                }
                "#,
            ))
            .with_extension(AuthenticationExt::new(StaticToken::bytes(b"Hi")))
            .with_extension(AuthorizationExt::new(InsertTokenAsHeader))
            .with_toml_config(
                r#"
                [authentication]
                default = "deny"
                "#,
            )
            .build()
            .await;

        let request = http::Request::builder()
            .method(http::Method::GET)
            // Does not exist.
            .uri("/.well-known/oauth-protected-resource/mcp")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = gateway.raw_execute(request).await;

        assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
    });
}
