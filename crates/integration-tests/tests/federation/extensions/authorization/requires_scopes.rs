use std::{collections::HashMap, sync::Arc};

use engine::{Engine, ErrorCode, ErrorResponse, GraphqlError};
use extension_catalog::Id;
use graphql_mocks::dynamic::DynamicSchema;
use integration_tests::{
    federation::{DynHookContext, EngineExt, TestExtension, TestExtensionBuilder, TestManifest},
    runtime,
};
use runtime::extension::{AuthorizationDecisions, QueryElement, Token, TokenRef};
use serde::Deserialize;

use crate::federation::extensions::authorization::AuthorizationExt;

#[derive(Default, Clone)]
struct AuthExt {
    token: Option<Token>,
}

impl TestExtensionBuilder for AuthExt {
    fn manifest(&self) -> TestManifest {
        TestManifest {
            id: Id {
                name: "authentication".to_string(),
                version: "1.0.0".parse().unwrap(),
            },
            kind: extension_catalog::Kind::Authentication(Default::default()),
            sdl: None,
        }
    }

    fn build(&self, _schema_directives: Vec<(&str, serde_json::Value)>) -> std::sync::Arc<dyn TestExtension> {
        Arc::new(self.clone())
    }
}

impl AuthExt {
    fn anonymous() -> Self {
        Self {
            token: Some(Token::Anonymous),
        }
    }

    fn claims(claims: &[(&'static str, &'static str)]) -> Self {
        let claims: HashMap<&str, &str> = claims.iter().copied().collect();
        Self {
            token: Some(Token::Bytes(serde_json::to_vec(&claims).unwrap())),
        }
    }
}

#[async_trait::async_trait]
impl TestExtension for AuthExt {
    async fn authenticate(&self, _headers: &http::HeaderMap) -> Result<Token, ErrorResponse> {
        self.token
            .clone()
            .ok_or_else(|| GraphqlError::new("No token found", ErrorCode::Unauthorized).into())
    }
}

#[derive(Default)]
struct RequiresScopes;

#[derive(serde::Deserialize)]
struct Arguments {
    input: String,
}

#[async_trait::async_trait]
impl TestExtension for RequiresScopes {
    #[allow(clippy::manual_async_fn)]
    async fn authorize_query(
        &self,
        _wasm_context: &DynHookContext,
        _headers: &tokio::sync::RwLock<http::HeaderMap>,
        token: TokenRef<'_>,
        elements_grouped_by_directive_name: Vec<(&str, Vec<QueryElement<'_, serde_json::Value>>)>,
    ) -> Result<AuthorizationDecisions, ErrorResponse> {
        let Some(bytes) = token.as_bytes() else {
            return Err(GraphqlError::new("No token found", ErrorCode::Unauthorized).into());
        };
        let token: serde_json::Value = serde_json::from_slice(bytes).unwrap();

        let Some(scopes) = token.get("scopes").and_then(|value| value.as_str()) else {
            return Err(GraphqlError::new("No scopes claim found in token", ErrorCode::Unauthorized).into());
        };

        let mut element_to_error = Vec::new();
        let errors = vec![GraphqlError::unauthorized()];

        let mut i = 0;
        for (_, elements) in elements_grouped_by_directive_name {
            for element in elements {
                let args = Arguments::deserialize(element.arguments).unwrap();
                if !scopes.contains(&args.input) {
                    element_to_error.push((i, 0));
                }
                i += 1;
            }
        }

        Ok(AuthorizationDecisions::DenySome {
            element_to_error,
            errors,
        })
    }
}

#[test]
fn anonymous() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                extend schema @link(url: "authorization-1.0.0", import: ["@auth"])

                type Query {
                    requiresUser: String @auth(input: "user")
                    greeting: String
                }
                "#,
                )
                .with_resolver(
                    "Query",
                    "requiresUser",
                    serde_json::Value::String("I am a user".to_owned()),
                )
                .with_resolver("Query", "greeting", serde_json::Value::String("Hi!".to_owned()))
                .into_subgraph("x"),
            )
            .with_extension(AuthorizationExt::new(RequiresScopes))
            .with_extension(AuthExt::anonymous())
            .build()
            .await;

        let response = engine.post("query { greeting }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "greeting": "Hi!"
          }
        }
        "#);

        let response = engine.post("query { requiresUser }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "errors": [
            {
              "message": "No token found",
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "#);

        let sent = engine.drain_graphql_requests_sent_to_by_name("x");
        insta::assert_json_snapshot!(sent, @r#"
        [
          {
            "query": "query { greeting }",
            "operationName": null,
            "variables": {},
            "extensions": {}
          }
        ]
        "#)
    });
}

#[test]
fn missing_claim() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                extend schema @link(url: "authorization-1.0.0", import: ["@auth"])

                type Query {
                    requiresUser: String @auth(input: "user")
                    greeting: String
                }
                "#,
                )
                .with_resolver(
                    "Query",
                    "requiresUser",
                    serde_json::Value::String("I am a user".to_owned()),
                )
                .with_resolver("Query", "greeting", serde_json::Value::String("Hi!".to_owned()))
                .into_subgraph("x"),
            )
            .with_extension(AuthorizationExt::new(RequiresScopes))
            .with_extension(AuthExt::claims(&[("dummy", "claim")]))
            .build()
            .await;

        let response = engine.post("query { greeting }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "greeting": "Hi!"
          }
        }
        "#);

        let response = engine.post("query { requiresUser }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "errors": [
            {
              "message": "No scopes claim found in token",
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "#);

        let sent = engine.drain_graphql_requests_sent_to_by_name("x");
        insta::assert_json_snapshot!(sent, @r#"
        [
          {
            "query": "query { greeting }",
            "operationName": null,
            "variables": {},
            "extensions": {}
          }
        ]
        "#)
    });
}

#[test]
fn missing_scope() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                extend schema @link(url: "authorization-1.0.0", import: ["@auth"])

                type Query {
                    requiresUser: String @auth(input: "user")
                    greeting: String
                }
                "#,
                )
                .with_resolver(
                    "Query",
                    "requiresUser",
                    serde_json::Value::String("I am a user".to_owned()),
                )
                .with_resolver("Query", "greeting", serde_json::Value::String("Hi!".to_owned()))
                .into_subgraph("x"),
            )
            .with_extension(AuthorizationExt::new(RequiresScopes))
            .with_extension(AuthExt::claims(&[("scopes", "group")]))
            .build()
            .await;

        let response = engine.post("query { greeting }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "greeting": "Hi!"
          }
        }
        "#);

        let response = engine.post("query { requiresUser }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "requiresUser": null
          },
          "errors": [
            {
              "message": "Not authorized",
              "locations": [
                {
                  "line": 1,
                  "column": 9
                }
              ],
              "path": [
                "requiresUser"
              ],
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "#);

        let sent = engine.drain_graphql_requests_sent_to_by_name("x");
        insta::assert_json_snapshot!(sent, @r#"
        [
          {
            "query": "query { greeting }",
            "operationName": null,
            "variables": {},
            "extensions": {}
          },
          {
            "query": "query { __typename @skip(if: true) }",
            "operationName": null,
            "variables": {},
            "extensions": {}
          }
        ]
        "#)
    });
}

#[test]
fn valid_scope() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                extend schema @link(url: "authorization-1.0.0", import: ["@auth"])

                type Query {
                    requiresUser: String @auth(input: "user")
                    greeting: String
                }
                "#,
                )
                .with_resolver(
                    "Query",
                    "requiresUser",
                    serde_json::Value::String("I am a user".to_owned()),
                )
                .with_resolver("Query", "greeting", serde_json::Value::String("Hi!".to_owned()))
                .into_subgraph("x"),
            )
            .with_extension(AuthorizationExt::new(RequiresScopes))
            .with_extension(AuthExt::claims(&[("scopes", "user")]))
            .build()
            .await;

        let response = engine.post("query { greeting }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "greeting": "Hi!"
          }
        }
        "#);

        let response = engine.post("query { requiresUser }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "requiresUser": "I am a user"
          }
        }
        "#);

        let sent = engine.drain_graphql_requests_sent_to_by_name("x");
        insta::assert_json_snapshot!(sent, @r#"
        [
          {
            "query": "query { greeting }",
            "operationName": null,
            "variables": {},
            "extensions": {}
          },
          {
            "query": "query { requiresUser }",
            "operationName": null,
            "variables": {},
            "extensions": {}
          }
        ]
        "#)
    });
}
