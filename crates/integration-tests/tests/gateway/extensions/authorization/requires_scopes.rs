use engine::{ErrorCode, ErrorResponse, GraphqlError};
use graphql_mocks::dynamic::DynamicSchema;
use integration_tests::{
    gateway::{AuthenticationExt, AuthorizationExt, AuthorizationTestExtension, ExtContext, Gateway},
    runtime,
};
use runtime::extension::{AuthorizationDecisions, QueryElement, TokenRef};
use serde::Deserialize;

use crate::gateway::extensions::authentication::static_token::StaticToken;

#[derive(Default)]
struct RequiresScopes;

#[derive(serde::Deserialize)]
struct Arguments {
    input: String,
}

#[async_trait::async_trait]
impl AuthorizationTestExtension for RequiresScopes {
    async fn authorize_query(
        &self,
        _ctx: &ExtContext,
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
        let engine = Gateway::builder()
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
            .with_extension(AuthenticationExt::new(StaticToken::anonymous()))
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
        let engine = Gateway::builder()
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
            .with_extension(AuthenticationExt::new(StaticToken::claims(&[("dummy", "claim")])))
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
        let engine = Gateway::builder()
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
            .with_extension(AuthenticationExt::new(StaticToken::claims(&[("scopes", "group")])))
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
        let engine = Gateway::builder()
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
            .with_extension(AuthenticationExt::new(StaticToken::claims(&[("scopes", "user")])))
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
