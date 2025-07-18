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

#[test]
fn mcp_is_not_authenticated_but_graphql_is() {
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
                [authentication.protected_resources]
                mcp.extensions = []

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

        let mut stream = gateway.mcp_http("/mcp").await;

        let response = stream
            .call_tool("search", serde_json::json!({"keywords": ["User"]}))
            .await;
        insta::assert_snapshot!(&response, @r##"
        # Incomplete fields
        type Query {
          user: User
        }

        type User {
          id: ID!
          name: String!
        }
        "##);

        let response = gateway.post("query { __typename }").await;
        insta::assert_json_snapshot!(&response, @r#"
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
    });
}

#[test]
fn mcp_is_authenticated_but_graphql_isnt() {
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
                [authentication.protected_resources]
                graphql.extensions = []

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

        let response = gateway.post("query { __typename }").await;
        insta::assert_json_snapshot!(&response, @r#"
        {
          "data": {
            "__typename": "Query"
          }
        }
        "#);
    });
}

#[test]
fn different_authentication() {
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
                [authentication.protected_resources]
                mcp.extensions = ["auth1"]
                graphql.extensions = ["auth2"]

                [mcp]
                enabled = true
            "#,
            )
            .with_extension(
                AuthenticationExt::new(StaticToken::error_response(GraphqlError::new(
                    "Auth1",
                    ErrorCode::Unauthenticated,
                )))
                .with_name("auth1"),
            )
            .with_extension(
                AuthenticationExt::new(StaticToken::error_response(GraphqlError::new(
                    "Auth2",
                    ErrorCode::Unauthenticated,
                )))
                .with_name("auth2"),
            )
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
              "message": "Auth1",
              "extensions": {
                "code": "UNAUTHENTICATED"
              }
            }
          ]
        }
        "#);
        assert_eq!(parts.status, http::StatusCode::UNAUTHORIZED);

        let response = gateway.post("query { __typename }").await;
        insta::assert_json_snapshot!(&response, @r#"
        {
          "errors": [
            {
              "message": "Auth2",
              "extensions": {
                "code": "UNAUTHENTICATED"
              }
            }
          ]
        }
        "#);
    });
}
