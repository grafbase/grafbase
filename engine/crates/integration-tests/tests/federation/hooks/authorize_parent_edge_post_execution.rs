use engine_v2::Engine;
use graphql_mocks::TeaShop;
use http::HeaderMap;
use integration_tests::{federation::EngineV2Ext, runtime};
use runtime::{
    error::{PartialErrorCode, PartialGraphqlError},
    hooks::{DynHookContext, DynHooks, EdgeDefinition},
};

use super::with_engine_for_auth;

#[test]
fn after_pre_execution_hook() {
    struct TestHooks;

    #[derive(serde::Deserialize)]
    struct User {
        id: usize,
    }

    #[async_trait::async_trait]
    impl DynHooks for TestHooks {
        async fn authorize_edge_pre_execution(
            &self,
            _context: &DynHookContext,
            _definition: EdgeDefinition<'_>,
            arguments: serde_json::Value,
            _metadata: Option<serde_json::Value>,
        ) -> Result<(), PartialGraphqlError> {
            if let Ok(user) = serde_json::from_value::<User>(arguments) {
                if user.id == 1 {
                    return Ok(());
                }
            }
            Err(PartialGraphqlError::new(
                "Not authorized",
                PartialErrorCode::Unauthorized,
            ))
        }

        async fn authorize_parent_edge_post_execution(
            &self,
            _context: &DynHookContext,
            _definition: EdgeDefinition<'_>,
            parents: Vec<serde_json::Value>,
            _metadata: Option<serde_json::Value>,
        ) -> Result<Vec<Result<(), PartialGraphqlError>>, PartialGraphqlError> {
            Ok(parents
                .into_iter()
                .map(|parent| {
                    if let Ok(user) = serde_json::from_value::<User>(parent) {
                        if user.id == 1 {
                            return Ok(());
                        }
                    }
                    Err(PartialGraphqlError::new(
                        "Not authorized",
                        PartialErrorCode::Unauthorized,
                    ))
                })
                .collect())
        }
    }

    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(TeaShop::with_sdl(
                r###"
                type Query {
                    user(id: Int!): User @authorized(arguments: "id")
                }

                type User {
                    id: Int!
                    address: Address! @authorized(fields: "id")
                }

                type Address {
                    street: String!
                }
            "###,
            ))
            .with_mock_hooks(TestHooks)
            .build()
            .await;

        let response = engine
            .execute(
                r#"
                query {
                    user(id: 1) {
                        address {
                            street
                        }
                    }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "user": {
              "address": {
                "street": "5678 Oak St"
              }
            }
          }
        }
        "###);

        // This used to panic because the response modifier was expecting a set of user objects
        // to exists (its input), but with pre-execution authorization of `user` we never request
        // it from the subgraph.
        let response = engine
            .execute(
                r#"
                query {
                    user(id: 2) {
                        address {
                            street
                        }
                    }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "user": null
          },
          "errors": [
            {
              "message": "Not authorized",
              "path": [
                "user"
              ],
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "###);
    });
}

#[test]
fn parents_are_provided() {
    struct TestHooks;

    #[async_trait::async_trait]
    impl DynHooks for TestHooks {
        async fn authorize_parent_edge_post_execution(
            &self,
            _context: &DynHookContext,
            _definition: EdgeDefinition<'_>,
            parents: Vec<serde_json::Value>,
            _metadata: Option<serde_json::Value>,
        ) -> Result<Vec<Result<(), PartialGraphqlError>>, PartialGraphqlError> {
            Ok(parents
                .into_iter()
                .map(|value| {
                    if value["id"] == "edge#1" {
                        Ok(())
                    } else {
                        Err(PartialGraphqlError::new(
                            "Unauthorized role",
                            PartialErrorCode::Unauthorized,
                        ))
                    }
                })
                .collect())
        }
    }

    with_engine_for_auth(TestHooks, |engine| async move {
        let response = engine
            .execute(
                r#"
                query {
                    yes: nullableCheck {
                       authorizedEdgeWithFields(prefix: "edge#", id: "1") {
                           withId
                       }
                    }
                    no: nullableCheck {
                       authorizedEdgeWithFields(prefix: "edge#", id: "2") {
                           withId
                       }
                    }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "yes": {
              "authorizedEdgeWithFields": {
                "withId": "You have access"
              }
            },
            "no": null
          },
          "errors": [
            {
              "message": "Unauthorized role",
              "path": [
                "no",
                "authorizedEdgeWithFields",
                "withId"
              ],
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "###);
    });
}

#[test]
fn metadata_is_provided() {
    struct TestHooks;

    const NULL: serde_json::Value = serde_json::Value::Null;

    fn extract_role(metadata: Option<&serde_json::Value>) -> Option<&str> {
        metadata
            .unwrap_or(&NULL)
            .as_array()?
            .first()?
            .as_array()?
            .first()?
            .as_str()
    }

    #[async_trait::async_trait]
    impl DynHooks for TestHooks {
        async fn authorize_parent_edge_post_execution(
            &self,
            _context: &DynHookContext,
            _definition: EdgeDefinition<'_>,
            parents: Vec<serde_json::Value>,
            metadata: Option<serde_json::Value>,
        ) -> Result<Vec<Result<(), PartialGraphqlError>>, PartialGraphqlError> {
            let Some(role) = extract_role(metadata.as_ref()) else {
                return Err(PartialGraphqlError::new(
                    "Unauthorized role",
                    PartialErrorCode::Unauthorized,
                ));
            };
            Ok(parents
                .into_iter()
                .map(|value| {
                    if value["id"].as_str().unwrap().starts_with(role) {
                        Ok(())
                    } else {
                        Err(PartialGraphqlError::new(
                            "Unauthorized role",
                            PartialErrorCode::Unauthorized,
                        ))
                    }
                })
                .collect())
        }
    }

    with_engine_for_auth(TestHooks, |engine| async move {
        let response = engine
            .execute(
                r#"
                query {
                    ok: nullableCheck {
                       authorizedEdgeWithFields(prefix: "rusty#", id: "1") {
                           withIdAndMetadata
                       }
                    }
                    wrongPrefix: nullableCheck {
                       authorizedEdgeWithFields(prefix: "edge#", id: "1") {
                            withIdAndMetadata
                       }
                    }
                    noMetadata: nullableCheck {
                       authorizedEdgeWithFields(prefix: "rusty#", id: "1") {
                            withId
                       }
                    }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "ok": {
              "authorizedEdgeWithFields": {
                "withIdAndMetadata": "You have access"
              }
            },
            "wrongPrefix": null,
            "noMetadata": null
          },
          "errors": [
            {
              "message": "Unauthorized role",
              "path": [
                "noMetadata",
                "authorizedEdgeWithFields",
                "withId"
              ],
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            },
            {
              "message": "Unauthorized role",
              "path": [
                "wrongPrefix",
                "authorizedEdgeWithFields",
                "withIdAndMetadata"
              ],
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "###);
    });
}

#[test]
fn definition_is_provided() {
    struct TestHooks;

    #[async_trait::async_trait]
    impl DynHooks for TestHooks {
        async fn authorize_parent_edge_post_execution(
            &self,
            _context: &DynHookContext,
            definition: EdgeDefinition<'_>,
            parents: Vec<serde_json::Value>,
            _metadata: Option<serde_json::Value>,
        ) -> Result<Vec<Result<(), PartialGraphqlError>>, PartialGraphqlError> {
            if definition.parent_type_name == "AuthorizedEdgeWithFields" && definition.field_name == "withId" {
                Ok(vec![Ok(()); parents.len()])
            } else {
                Err(PartialGraphqlError::new(
                    "Wrong definition",
                    PartialErrorCode::Unauthorized,
                ))
            }
        }
    }

    with_engine_for_auth(TestHooks, |engine| async move {
        let response = engine
            .execute(
                r#"
                query {
                    ok: nullableCheck {
                       authorizedEdgeWithFields(prefix: "edge#", id: "1") {
                           withId
                       }
                    }
                    wrongField: nullableCheck {
                       authorizedEdgeWithFields(prefix: "edge#", id: "1") {
                           withIdAndMetadata
                       }
                    }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "ok": {
              "authorizedEdgeWithFields": {
                "withId": "You have access"
              }
            },
            "wrongField": null
          },
          "errors": [
            {
              "message": "Wrong definition",
              "path": [
                "wrongField",
                "authorizedEdgeWithFields",
                "withIdAndMetadata"
              ],
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "###);
    });
}

#[test]
fn context_is_propagated() {
    struct TestHooks;

    #[async_trait::async_trait]
    impl DynHooks for TestHooks {
        async fn on_gateway_request(
            &self,
            context: &mut DynHookContext,
            headers: HeaderMap,
        ) -> Result<HeaderMap, PartialGraphqlError> {
            if let Some(client) = headers
                .get("x-grafbase-client-name")
                .and_then(|value| value.to_str().ok())
            {
                context.insert("client", client);
            }
            Ok(headers)
        }

        async fn authorize_parent_edge_post_execution(
            &self,
            context: &DynHookContext,
            _definition: EdgeDefinition<'_>,
            parents: Vec<serde_json::Value>,
            _metadata: Option<serde_json::Value>,
        ) -> Result<Vec<Result<(), PartialGraphqlError>>, PartialGraphqlError> {
            if context.get("client").is_some() {
                Ok(vec![Ok(()); parents.len()])
            } else {
                Err(PartialGraphqlError::new(
                    "Missing client",
                    PartialErrorCode::Unauthorized,
                ))
            }
        }
    }

    with_engine_for_auth(TestHooks, |engine| async move {
        let response = engine
            .execute(r###"query { check { authorizedEdgeWithFields(prefix: "edge#", id: "1") { withId } } }"###)
            .by_client("hi", "")
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "check": {
              "authorizedEdgeWithFields": {
                "withId": "You have access"
              }
            }
          }
        }
        "###);

        let response = engine
            .execute(r###"query { check { authorizedEdgeWithFields(prefix: "edge#", id: "1") { withId } } }"###)
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Missing client",
              "path": [
                "check",
                "authorizedEdgeWithFields",
                "withId"
              ],
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "###);
    });
}

#[test]
fn error_propagation() {
    struct TestHooks;

    #[async_trait::async_trait]
    impl DynHooks for TestHooks {
        async fn authorize_parent_edge_post_execution(
            &self,
            _context: &DynHookContext,
            _definition: EdgeDefinition<'_>,
            _parents: Vec<serde_json::Value>,
            _metadata: Option<serde_json::Value>,
        ) -> Result<Vec<Result<(), PartialGraphqlError>>, PartialGraphqlError> {
            Err(PartialGraphqlError::new("Broken", PartialErrorCode::HookError))
        }
    }

    with_engine_for_auth(TestHooks, |engine| async move {
        let response = engine
            .execute(
                r#"
                query {
                    check {
                        authorizedEdgeWithFields(prefix: "edge#", id: "1") { withId }
                    }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Broken",
              "path": [
                "check",
                "authorizedEdgeWithFields",
                "withId"
              ],
              "extensions": {
                "code": "HOOK_ERROR"
              }
            }
          ]
        }
        "###);
    });
}
