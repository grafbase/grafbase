use engine::{ErrorCode, GraphqlError};
use graphql_mocks::{EchoSchema, Schema as _};
use integration_tests::{
    gateway::{AuthenticationExt, AuthorizationExt, Gateway},
    runtime,
};

use crate::gateway::extensions::{authentication::static_token::StaticToken, authorization::InsertTokenAsHeader};

#[test]
fn no_extension() {
    runtime().block_on(async move {
        let gateway = Gateway::builder()
            .with_subgraph(EchoSchema::default().with_sdl(
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

        let response = gateway.post(r#"query { header(name: "token") }"#).await;
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
fn extension() {
    runtime().block_on(async move {
        let gateway = Gateway::builder()
            .with_subgraph(EchoSchema::default().with_sdl(
                r#"
                extend schema @link(url: "authorization-1.0.0", import: ["@auth"])

                type Query {
                    header(name: String): String @auth
                }
                "#,
            ))
            .with_extension(AuthenticationExt::new(StaticToken::error_response(GraphqlError::new(
                "My error message",
                ErrorCode::Unauthenticated,
            ))))
            .with_extension(AuthorizationExt::new(InsertTokenAsHeader))
            .build()
            .await;

        let response = gateway.post(r#"query { header(name: "token") }"#).await;
        insta::assert_json_snapshot!(response, @r#"
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
fn no_extension_with_anonymous_default() {
    runtime().block_on(async move {
        let gateway = Gateway::builder()
            .with_subgraph(EchoSchema::default().with_sdl(
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

        let response = gateway.post(r#"query { header(name: "token") }"#).await;
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
fn extension_with_anonymous_default() {
    runtime().block_on(async move {
        let gateway = Gateway::builder()
            .with_subgraph(EchoSchema::default().with_sdl(
                r#"
                extend schema @link(url: "authorization-1.0.0", import: ["@auth"])

                type Query {
                    header(name: String): String @auth
                }
                "#,
            ))
            .with_extension(AuthenticationExt::new(StaticToken::error_response(GraphqlError::new(
                "My error message",
                ErrorCode::Unauthenticated,
            ))))
            .with_extension(AuthorizationExt::new(InsertTokenAsHeader))
            .with_toml_config(
                r#"
                [authentication]
                default = "anonymous"
                "#,
            )
            .build()
            .await;

        let response = gateway.post(r#"query { header(name: "token") }"#).await;
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
        let gateway = Gateway::builder()
            .with_subgraph(EchoSchema::default().with_sdl(
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

        let response = gateway.post(r#"query { header(name: "token") }"#).await;
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

        let sent = gateway.drain_graphql_requests_sent_to_by_name("subgraph");
        insta::assert_json_snapshot!(sent, @"[]");
    });
}

#[test]
fn extension_with_deny_default() {
    runtime().block_on(async move {
        let gateway = Gateway::builder()
            .with_subgraph(EchoSchema::default().with_sdl(
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

        let response = gateway.post(r#"query { header(name: "token") }"#).await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "header": "Hi"
          }
        }
        "#);
    });
}

#[test]
fn graphql_no_extension() {
    runtime().block_on(async move {
        let gateway = Gateway::builder()
            .with_subgraph(EchoSchema::default().with_sdl(
                r#"
                extend schema @link(url: "authorization-1.0.0", import: ["@auth"])

                type Query {
                    header(name: String): String @auth
                }
                "#,
            ))
            .with_toml_config(
                r#"
                [authentication.protected_resources.graphql]
                extensions = []

                [mcp]
                enabled = true
            "#,
            )
            .with_extension(AuthenticationExt::new(StaticToken::error_response(GraphqlError::new(
                "My error message",
                ErrorCode::Unauthenticated,
            ))))
            .with_extension(AuthorizationExt::new(InsertTokenAsHeader))
            .build()
            .await;

        let response = gateway.post(r#"query { header(name: "token") }"#).await;
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
fn graphql_default_override() {
    runtime().block_on(async move {
        let gateway = Gateway::builder()
            .with_subgraph(EchoSchema::default().with_sdl(
                r#"
                extend schema @link(url: "authorization-1.0.0", import: ["@auth"])

                type Query {
                    header(name: String): String @auth
                }
                "#,
            ))
            .with_toml_config(
                r#"
                [authentication]
                default = "anonymous"

                [authentication.protected_resources.graphql]
                default = "deny"

                [mcp]
                enabled = true
            "#,
            )
            .with_extension(AuthenticationExt::new(StaticToken::error_response(GraphqlError::new(
                "My error message",
                ErrorCode::Unauthenticated,
            ))))
            .with_extension(AuthorizationExt::new(InsertTokenAsHeader))
            .build()
            .await;

        let response = gateway.post(r#"query { header(name: "token") }"#).await;
        insta::assert_json_snapshot!(response, @r#"
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
        let sent = gateway.drain_graphql_requests_sent_to_by_name("subgraph");
        insta::assert_json_snapshot!(sent, @"[]");

        let response = gateway
            .mcp_http("/mcp")
            .await
            .call_tool("search", serde_json::json!({"keywords": ["header"]}))
            .await;
        insta::assert_snapshot!(&response, @r##"
        # Incomplete fields
        type Query {
          header(name: String): String
        }
        "##);
    });
}

#[test]
fn graphql_extension() {
    runtime().block_on(async move {
        let gateway = Gateway::builder()
            .with_subgraph(EchoSchema::default().with_sdl(
                r#"
                extend schema @link(url: "authorization-1.0.0", import: ["@auth"])

                type Query {
                    header(name: String): String @auth
                }
                "#,
            ))
            .with_extension(AuthenticationExt::new(StaticToken::error_response(GraphqlError::new(
                "My error message",
                ErrorCode::Unauthenticated,
            ))))
            .with_extension(AuthorizationExt::new(InsertTokenAsHeader))
            .with_toml_config(
                r#"
                [authentication.protected_resources.graphql]
                extensions = ["authentication"]

                [mcp]
                enabled = true
            "#,
            )
            .build()
            .await;

        let response = gateway.post(r#"query { header(name: "token") }"#).await;
        insta::assert_json_snapshot!(response, @r#"
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
fn graphql_no_extension_with_anonymous_default() {
    runtime().block_on(async move {
        let gateway = Gateway::builder()
            .with_subgraph(EchoSchema::default().with_sdl(
                r#"
                extend schema @link(url: "authorization-1.0.0", import: ["@auth"])

                type Query {
                    header(name: String): String @auth
                }
                "#,
            ))
            .with_extension(AuthenticationExt::new(StaticToken::error_response(GraphqlError::new(
                "My error message",
                ErrorCode::Unauthenticated,
            ))))
            .with_extension(AuthorizationExt::new(InsertTokenAsHeader))
            .with_toml_config(
                r#"
                [authentication.protected_resources.graphql]
                extensions = []
                default = "anonymous"
                "#,
            )
            .build()
            .await;

        let response = gateway.post(r#"query { header(name: "token") }"#).await;
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
fn graphql_extension_with_anonymous_default() {
    runtime().block_on(async move {
        let gateway = Gateway::builder()
            .with_subgraph(EchoSchema::default().with_sdl(
                r#"
                extend schema @link(url: "authorization-1.0.0", import: ["@auth"])

                type Query {
                    header(name: String): String @auth
                }
                "#,
            ))
            .with_extension(AuthenticationExt::new(StaticToken::error_response(GraphqlError::new(
                "My error message",
                ErrorCode::Unauthenticated,
            ))))
            .with_extension(AuthorizationExt::new(InsertTokenAsHeader))
            .with_toml_config(
                r#"
                [authentication.protected_resources.graphql]
                extensions = ["authentication"]
                default = "anonymous"
                "#,
            )
            .build()
            .await;

        let response = gateway.post(r#"query { header(name: "token") }"#).await;
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
fn graphql_no_extension_with_deny_default() {
    runtime().block_on(async move {
        let gateway = Gateway::builder()
            .with_subgraph(EchoSchema::default().with_sdl(
                r#"
                extend schema @link(url: "authorization-1.0.0", import: ["@auth"])

                type Query {
                    header(name: String): String @auth
                }
                "#,
            ))
            .with_extension(AuthenticationExt::new(StaticToken::error_response(GraphqlError::new(
                "My error message",
                ErrorCode::Unauthenticated,
            ))))
            .with_extension(AuthorizationExt::new(InsertTokenAsHeader))
            .with_toml_config(
                r#"
                [authentication.protected_resources.graphql]
                extensions = []
                default = "deny"
                "#,
            )
            .build()
            .await;

        let response = gateway.post(r#"query { header(name: "token") }"#).await;
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

        let sent = gateway.drain_graphql_requests_sent_to_by_name("subgraph");
        insta::assert_json_snapshot!(sent, @"[]");
    });
}

#[test]
fn graphql_extension_with_deny_default() {
    runtime().block_on(async move {
        let gateway = Gateway::builder()
            .with_subgraph(EchoSchema::default().with_sdl(
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
                [authentication.protected_resources.graphql]
                extensions = ["authentication"]
                default = "deny"
                "#,
            )
            .build()
            .await;

        let response = gateway.post(r#"query { header(name: "token") }"#).await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "header": "Hi"
          }
        }
        "#);
    });
}

#[test]
fn mcp_no_extension() {
    runtime().block_on(async move {
        let gateway = Gateway::builder()
            .with_subgraph(EchoSchema::default().with_sdl(
                r#"
                extend schema @link(url: "authorization-1.0.0", import: ["@auth"])

                type Query {
                    header(name: String): String @auth
                }
                "#,
            ))
            .with_toml_config(
                r#"
                [mcp]
                enabled = true

                [authentication.protected_resources.mcp]
                extensions = []
            "#,
            )
            .with_extension(AuthenticationExt::new(StaticToken::error_response(GraphqlError::new(
                "My error message",
                ErrorCode::Unauthenticated,
            ))))
            .with_extension(AuthorizationExt::new(InsertTokenAsHeader))
            .build()
            .await;

        let response = gateway
            .mcp_http("/mcp")
            .await
            .call_tool("search", serde_json::json!({"keywords": ["header"]}))
            .await;
        insta::assert_snapshot!(&response, @r##"
        # Incomplete fields
        type Query {
          header(name: String): String
        }
        "##);

        let response = gateway.post(r#"query { header(name: "token") }"#).await;
        insta::assert_json_snapshot!(response, @r#"
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
fn mcp_extension() {
    runtime().block_on(async move {
        let gateway = Gateway::builder()
            .with_subgraph(EchoSchema::default().with_sdl(
                r#"
                extend schema @link(url: "authorization-1.0.0", import: ["@auth"])

                type Query {
                    header(name: String): String @auth
                }
                "#,
            ))
            .with_toml_config(
                r#"
                [mcp]
                enabled = true

                [authentication.protected_resources.mcp]
                extensions = ["authentication"]
            "#,
            )
            .with_extension(AuthenticationExt::new(StaticToken::error_response(GraphqlError::new(
                "My error message",
                ErrorCode::Unauthenticated,
            ))))
            .with_extension(AuthorizationExt::new(InsertTokenAsHeader))
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

        let response = gateway.post(r#"query { header(name: "token") }"#).await;
        insta::assert_json_snapshot!(response, @r#"
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
fn mcp_default_override() {
    runtime().block_on(async move {
        let gateway = Gateway::builder()
            .with_subgraph(EchoSchema::default().with_sdl(
                r#"
                extend schema @link(url: "authorization-1.0.0", import: ["@auth"])

                type Query {
                    header(name: String): String @auth
                }
                "#,
            ))
            .with_toml_config(
                r#"
                [authentication]
                default = "anonymous"

                [authentication.protected_resources.mcp]
                default = "deny"

                [mcp]
                enabled = true
            "#,
            )
            .with_extension(AuthenticationExt::new(StaticToken::error_response(GraphqlError::new(
                "My error message",
                ErrorCode::Unauthenticated,
            ))))
            .with_extension(AuthorizationExt::new(InsertTokenAsHeader))
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

        let response = gateway.post(r#"query { header(name: "token") }"#).await;
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
fn mcp_no_extension_with_anonymous_default() {
    runtime().block_on(async move {
        let gateway = Gateway::builder()
            .with_subgraph(EchoSchema::default().with_sdl(
                r#"
                extend schema @link(url: "authorization-1.0.0", import: ["@auth"])

                type Query {
                    header(name: String): String @auth
                }
                "#,
            ))
            .with_extension(AuthenticationExt::new(StaticToken::error_response(GraphqlError::new(
                "My error message",
                ErrorCode::Unauthenticated,
            ))))
            .with_extension(AuthorizationExt::new(InsertTokenAsHeader))
            .with_toml_config(
                r#"
                [mcp]
                enabled = true

                [authentication.protected_resources.mcp]
                extensions = []
                default = "anonymous"
                "#,
            )
            .build()
            .await;

        let response = gateway
            .mcp_http("/mcp")
            .await
            .call_tool("search", serde_json::json!({"keywords": ["header"]}))
            .await;
        insta::assert_snapshot!(&response, @r##"
        # Incomplete fields
        type Query {
          header(name: String): String
        }
        "##);

        let response = gateway.post(r#"query { header(name: "token") }"#).await;
        insta::assert_json_snapshot!(response, @r#"
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
fn mcp_extension_with_anonymous_default() {
    runtime().block_on(async move {
        let gateway = Gateway::builder()
            .with_subgraph(EchoSchema::default().with_sdl(
                r#"
                extend schema @link(url: "authorization-1.0.0", import: ["@auth"])

                type Query {
                    header(name: String): String @auth
                }
                "#,
            ))
            .with_extension(AuthenticationExt::new(StaticToken::error_response(GraphqlError::new(
                "My error message",
                ErrorCode::Unauthenticated,
            ))))
            .with_extension(AuthorizationExt::new(InsertTokenAsHeader))
            .with_toml_config(
                r#"
                [mcp]
                enabled = true

                [authentication.protected_resources.mcp]
                extensions = ["authentication"]
                default = "anonymous"
                "#,
            )
            .build()
            .await;

        let response = gateway
            .mcp_http("/mcp")
            .await
            .call_tool("search", serde_json::json!({"keywords": ["header"]}))
            .await;
        insta::assert_snapshot!(&response, @r##"
        # Incomplete fields
        type Query {
          header(name: String): String
        }
        "##);

        let response = gateway.post(r#"query { header(name: "token") }"#).await;
        insta::assert_json_snapshot!(response, @r#"
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
fn mcp_no_extension_with_deny_default() {
    runtime().block_on(async move {
        let gateway = Gateway::builder()
            .with_subgraph(EchoSchema::default().with_sdl(
                r#"
                extend schema @link(url: "authorization-1.0.0", import: ["@auth"])

                type Query {
                    header(name: String): String @auth
                }
                "#,
            ))
            .with_extension(AuthenticationExt::new(StaticToken::error_response(GraphqlError::new(
                "My error message",
                ErrorCode::Unauthenticated,
            ))))
            .with_extension(AuthorizationExt::new(InsertTokenAsHeader))
            .with_toml_config(
                r#"
                [mcp]
                enabled = true

                [authentication.protected_resources.mcp]
                extensions = []
                default = "deny"
                "#,
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
              "message": "Unauthenticated",
              "extensions": {
                "code": "UNAUTHENTICATED"
              }
            }
          ]
        }
        "#);
        assert_eq!(parts.status, http::StatusCode::UNAUTHORIZED);

        let response = gateway.post(r#"query { header(name: "token") }"#).await;
        insta::assert_json_snapshot!(response, @r#"
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

        let sent = gateway.drain_graphql_requests_sent_to_by_name("subgraph");
        insta::assert_json_snapshot!(sent, @"[]");
    });
}

#[test]
fn mcp_extension_with_deny_default() {
    runtime().block_on(async move {
        let gateway = Gateway::builder()
            .with_subgraph(EchoSchema::default().with_sdl(
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
                [mcp]
                enabled = true

                [authentication.protected_resources.mcp]
                extensions = ["authentication"]
                default = "deny"
                "#,
            )
            .build()
            .await;

        let response = gateway
            .mcp_http("/mcp")
            .await
            .call_tool("search", serde_json::json!({"keywords": ["header"]}))
            .await;
        insta::assert_snapshot!(&response, @r##"
        # Incomplete fields
        type Query {
          header(name: String): String
        }
        "##);

        let response = gateway.post(r#"query { header(name: "token") }"#).await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "header": "Hi"
          }
        }
        "#);
    });
}
