use engine::{ErrorCode, GraphqlError};
use integration_tests::{
    gateway::{AuthenticationExt, Gateway},
    runtime,
};

use crate::gateway::extensions::authentication::static_token::StaticToken;

#[test]
fn simple() {
    runtime().block_on(async move {
        let gateway = Gateway::builder()
            .with_subgraph_sdl(
                "x",
                r#"
            type Query {
                user: User
            }

            type User {
                id: ID!
                name: String!
            }
            "#,
            )
            .with_toml_config(
                r#"
                [mcp]
                enabled = true
            "#,
            )
            .with_extension(AuthenticationExt::new(StaticToken::error_response(GraphqlError::new(
                "My error message",
                ErrorCode::Unauthenticated,
            ))))
            .build()
            .await;

        let response = gateway
            .raw_execute(http::Request::post("/mcp").body(Vec::new()).unwrap())
            .await;
        let (parts, body) = response.into_parts();
        let json = serde_json::from_slice::<serde_json::Value>(&body).unwrap();

        insta::assert_json_snapshot!(json, @r#"
        {
          "errors": [
            {
              "message": "My error message",
              "extensions": {
                "code": "UNAUTHENTICATED"
              }
            }
          ]
        }
        "#);
        assert_eq!(parts.status, http::StatusCode::UNAUTHORIZED);
    });
}
