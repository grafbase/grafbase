use std::{collections::HashMap, sync::Arc};

use engine::Engine;
use extension_catalog::Id;
use graphql_mocks::dynamic::DynamicSchema;
use integration_tests::{
    federation::{EngineExt, TestExtension, TestExtensionBuilder, TestExtensionConfig},
    runtime,
};
use runtime::{
    auth::LegacyToken,
    error::{ErrorResponse, PartialErrorCode, PartialGraphqlError},
    extension::{AuthorizationDecisions, QueryElement, Token},
};
use serde::Deserialize;

use crate::federation::extensions::authorization::SimpleAuthExt;

#[derive(Default, Clone)]
struct AuthExt {
    token: Option<Token>,
}

impl TestExtensionBuilder for AuthExt {
    fn id(&self) -> Id {
        Id {
            name: "authentication".to_string(),
            version: "1.0.0".parse().unwrap(),
        }
    }

    fn config(&self) -> TestExtensionConfig {
        TestExtensionConfig {
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
            .ok_or_else(|| PartialGraphqlError::new("No token found", PartialErrorCode::Unauthorized).into())
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
    async fn authorize_query<'a>(
        &self,
        ctx: Arc<engine::RequestContext>,
        elements_grouped_by_directive_name: Vec<(&str, Vec<QueryElement<'_, serde_json::Value>>)>,
    ) -> Result<AuthorizationDecisions, ErrorResponse> {
        let LegacyToken::Extension(Token::Bytes(bytes)) = ctx.token() else {
            return Err(PartialGraphqlError::new("No token found", PartialErrorCode::Unauthorized).into());
        };
        let token: serde_json::Value = serde_json::from_slice(bytes).unwrap();

        let Some(scopes) = token.get("scopes").and_then(|value| value.as_str()) else {
            return Err(
                PartialGraphqlError::new("No scopes claim found in token", PartialErrorCode::Unauthorized).into(),
            );
        };

        let mut element_to_error = Vec::new();
        let errors = vec![PartialGraphqlError::new(
            "Not authorized",
            PartialErrorCode::Unauthorized,
        )];

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
                extend schema @link(url: "simple-auth-1.0.0", import: ["@auth"])

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
            .with_extension(SimpleAuthExt::new(RequiresScopes))
            .with_extension(AuthExt::anonymous())
            .with_toml_config(
                r#"
                [[authentication.providers]]

                [authentication.providers.extension]
                extension = "authentication"
                "#,
            )
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
    });
}

#[test]
fn missing_claim() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                extend schema @link(url: "simple-auth-1.0.0", import: ["@auth"])

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
            .with_extension(SimpleAuthExt::new(RequiresScopes))
            .with_extension(AuthExt::claims(&[("dummy", "claim")]))
            .with_toml_config(
                r#"
                [[authentication.providers]]

                [authentication.providers.extension]
                extension = "authentication"
                "#,
            )
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
    });
}

#[test]
fn missing_scope() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                extend schema @link(url: "simple-auth-1.0.0", import: ["@auth"])

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
            .with_extension(SimpleAuthExt::new(RequiresScopes))
            .with_extension(AuthExt::claims(&[("scopes", "group")]))
            .with_toml_config(
                r#"
                [[authentication.providers]]

                [authentication.providers.extension]
                extension = "authentication"
                "#,
            )
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
    });
}

#[test]
fn valid_scope() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                extend schema @link(url: "simple-auth-1.0.0", import: ["@auth"])

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
            .with_extension(SimpleAuthExt::new(RequiresScopes))
            .with_extension(AuthExt::claims(&[("scopes", "user")]))
            .with_toml_config(
                r#"
                [[authentication.providers]]

                [authentication.providers.extension]
                extension = "authentication"
                "#,
            )
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
    });
}
