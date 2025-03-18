use std::sync::{Arc, atomic::AtomicBool};

use super::with_engine_for_auth;
use engine::{Engine, ErrorCode, ErrorResponse, GraphqlError};
use graphql_mocks::dynamic::{DynamicSchema, EntityResolverContext};
use http::HeaderMap;
use integration_tests::{
    federation::{DynHookContext, DynHooks, EngineExt},
    runtime,
};
use runtime::hooks::EdgeDefinition;

#[test]
fn single_decision_applies_to_all() {
    #[derive(Default, Clone)]
    struct TestHooks {
        is_authorized: Arc<AtomicBool>,
    }

    #[async_trait::async_trait]
    impl DynHooks for TestHooks {
        async fn authorize_edge_node_post_execution(
            &self,
            _context: &DynHookContext,
            _definition: EdgeDefinition<'_>,
            _nodes: Vec<serde_json::Value>,
            _metadata: Option<serde_json::Value>,
        ) -> Result<Vec<Result<(), GraphqlError>>, GraphqlError> {
            if self.is_authorized.as_ref().load(std::sync::atomic::Ordering::Relaxed) {
                Ok(vec![Ok(())])
            } else {
                Ok(vec![Err(GraphqlError::new("Unauthorized", ErrorCode::Unauthorized))])
            }
        }
    }

    runtime().block_on(async move {
        let hooks = TestHooks::default();

        let gateway = Engine::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                type Query {
                    users: [User]! @authorized(node: "id")
                }

                type User @key(fields: "id") {
                    id: ID!
                }
                "#,
                )
                .with_resolver("Query", "users", serde_json::json!([{"id": "1"}, {"id": "2"}]))
                .into_subgraph("x"),
            )
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                scalar Any

                type User @key(fields: "id") {
                    id: ID!
                    name: String!
                }
                "#,
                )
                .with_entity_resolver("User", |ctx: EntityResolverContext<'_>| match ctx.representation["id"]
                    .as_str()
                    .unwrap()
                {
                    "1" => Some(serde_json::json!({"__typename": "User", "name": "Alice"})),
                    "2" => Some(serde_json::json!({"__typename": "User", "name": "Bob"})),
                    _ => unreachable!(),
                })
                .into_subgraph("y"),
            )
            .with_mock_hooks(hooks.clone())
            .build()
            .await;

        hooks.is_authorized.store(true, std::sync::atomic::Ordering::Relaxed);
        let response = gateway.post("{ users { name } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "users": [
              {
                "name": "Alice"
              },
              {
                "name": "Bob"
              }
            ]
          }
        }
        "#);

        hooks.is_authorized.store(false, std::sync::atomic::Ordering::Relaxed);
        let response = gateway.post("{ users { name } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "users": [
              null,
              null
            ]
          },
          "errors": [
            {
              "message": "Unauthorized",
              "path": [
                "users",
                0
              ],
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            },
            {
              "message": "Unauthorized",
              "path": [
                "users",
                1
              ],
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "#);
    })
}
#[test]
fn continue_execution() {
    struct TestHooks;

    #[async_trait::async_trait]
    impl DynHooks for TestHooks {
        async fn authorize_edge_node_post_execution(
            &self,
            _context: &DynHookContext,
            _definition: EdgeDefinition<'_>,
            nodes: Vec<serde_json::Value>,
            _metadata: Option<serde_json::Value>,
        ) -> Result<Vec<Result<(), GraphqlError>>, GraphqlError> {
            Ok(nodes
                .into_iter()
                .map(|value| {
                    if value["id"] == "1" {
                        Ok(())
                    } else {
                        Err(GraphqlError::new("Unauthorized id", ErrorCode::Unauthorized))
                    }
                })
                .collect())
        }
    }

    runtime().block_on(async move {
        let gateway = Engine::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                type Query {
                    users: [User]! @authorized(node: "id")
                }

                type User @key(fields: "id") {
                    id: ID!
                }
                "#,
                )
                .with_resolver("Query", "users", serde_json::json!([{"id": "1"}, {"id": "2"}]))
                .into_subgraph("x"),
            )
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                scalar Any

                type User @key(fields: "id") {
                    id: ID!
                    name: String!
                }
                "#,
                )
                .with_entity_resolver("User", |ctx: EntityResolverContext<'_>| match ctx.representation["id"]
                    .as_str()
                    .unwrap()
                {
                    "1" => Some(serde_json::json!({"__typename": "User", "name": "Alice"})),
                    "2" => Some(serde_json::json!({"__typename": "User", "name": "Bob"})),
                    _ => unreachable!(),
                })
                .into_subgraph("y"),
            )
            .with_mock_hooks(TestHooks)
            .build()
            .await;

        let response = gateway.post("{ users { name } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "users": [
              {
                "name": "Alice"
              },
              null
            ]
          },
          "errors": [
            {
              "message": "Unauthorized id",
              "path": [
                "users",
                1
              ],
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "#);
    })
}

#[test]
fn nodes_are_provided() {
    struct TestHooks;

    #[async_trait::async_trait]
    impl DynHooks for TestHooks {
        async fn authorize_edge_node_post_execution(
            &self,
            _context: &DynHookContext,
            _definition: EdgeDefinition<'_>,
            nodes: Vec<serde_json::Value>,
            _metadata: Option<serde_json::Value>,
        ) -> Result<Vec<Result<(), GraphqlError>>, GraphqlError> {
            Ok(nodes
                .into_iter()
                .map(|value| {
                    if value["id"] == "1" {
                        Ok(())
                    } else {
                        Err(GraphqlError::new("Unauthorized id", ErrorCode::Unauthorized))
                    }
                })
                .collect())
        }
    }

    with_engine_for_auth(TestHooks, |engine| async move {
        let response = engine
            .post(
                r#"
                query {
                    yes: nullableCheck {
                       authorizedEdgeWithNode(ids: ["1"]) {
                           withId { id }
                       }
                    }
                    no: nullableCheck {
                       authorizedEdgeWithNode(ids: ["2"]) {
                           withId { id }
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
              "authorizedEdgeWithNode": {
                "withId": {
                  "id": "1"
                }
              }
            },
            "no": null
          },
          "errors": [
            {
              "message": "Unauthorized id",
              "path": [
                "no",
                "authorizedEdgeWithNode",
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
        async fn authorize_edge_node_post_execution(
            &self,
            _context: &DynHookContext,
            _definition: EdgeDefinition<'_>,
            nodes: Vec<serde_json::Value>,
            metadata: Option<serde_json::Value>,
        ) -> Result<Vec<Result<(), GraphqlError>>, GraphqlError> {
            let Some(role) = extract_role(metadata.as_ref()) else {
                return Err(GraphqlError::new("Unauthorized role", ErrorCode::Unauthorized));
            };
            Ok(nodes
                .into_iter()
                .map(|value| {
                    if value["id"].as_str().unwrap().starts_with(role) {
                        Ok(())
                    } else {
                        Err(GraphqlError::new("Unauthorized role", ErrorCode::Unauthorized))
                    }
                })
                .collect())
        }
    }

    with_engine_for_auth(TestHooks, |engine| async move {
        let response = engine
            .post(
                r#"
                query {
                    ok: nullableCheck {
                       authorizedEdgeWithNode(ids: ["rusty"]) {
                           withIdAndMetadata { id }
                       }
                    }
                    wrongId: nullableCheck {
                       authorizedEdgeWithNode(ids: ["anonymous"]) {
                            withIdAndMetadata { id }
                       }
                    }
                    noMetadata: nullableCheck {
                       authorizedEdgeWithNode(ids: ["rusty"]) {
                            withId { id }
                       }
                    }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "ok": {
              "authorizedEdgeWithNode": {
                "withIdAndMetadata": {
                  "id": "rusty"
                }
              }
            },
            "wrongId": null,
            "noMetadata": null
          },
          "errors": [
            {
              "message": "Unauthorized role",
              "path": [
                "noMetadata",
                "authorizedEdgeWithNode",
                "withId"
              ],
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            },
            {
              "message": "Unauthorized role",
              "path": [
                "wrongId",
                "authorizedEdgeWithNode",
                "withIdAndMetadata"
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
fn definition_is_provided() {
    struct TestHooks;

    #[async_trait::async_trait]
    impl DynHooks for TestHooks {
        async fn authorize_edge_node_post_execution(
            &self,
            _context: &DynHookContext,
            definition: EdgeDefinition<'_>,
            nodes: Vec<serde_json::Value>,
            _metadata: Option<serde_json::Value>,
        ) -> Result<Vec<Result<(), GraphqlError>>, GraphqlError> {
            if definition.parent_type_name == "AuthorizedEdgeWithNode" && definition.field_name == "withId" {
                Ok(vec![Ok(()); nodes.len()])
            } else {
                Err(GraphqlError::new("Wrong definition", ErrorCode::Unauthorized))
            }
        }
    }

    with_engine_for_auth(TestHooks, |engine| async move {
        let response = engine
            .post(
                r#"
                query {
                    ok: nullableCheck {
                       authorizedEdgeWithNode(ids: ["1"]) {
                           withId { id }
                       }
                    }
                    wrongField: nullableCheck {
                       authorizedEdgeWithNode(ids: ["1"]) {
                           withIdAndMetadata { id }
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
              "authorizedEdgeWithNode": {
                "withId": {
                  "id": "1"
                }
              }
            },
            "wrongField": null
          },
          "errors": [
            {
              "message": "Wrong definition",
              "path": [
                "wrongField",
                "authorizedEdgeWithNode",
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
            _url: &str,
            headers: HeaderMap,
        ) -> Result<HeaderMap, ErrorResponse> {
            if let Some(client) = headers
                .get("x-grafbase-client-name")
                .and_then(|value| value.to_str().ok())
            {
                context.insert("client", client);
            }

            Ok(headers)
        }

        async fn authorize_edge_node_post_execution(
            &self,
            context: &DynHookContext,
            _definition: EdgeDefinition<'_>,
            nodes: Vec<serde_json::Value>,
            _metadata: Option<serde_json::Value>,
        ) -> Result<Vec<Result<(), GraphqlError>>, GraphqlError> {
            if context.get("client").is_some() {
                Ok(vec![Ok(()); nodes.len()])
            } else {
                Err(GraphqlError::new("Missing client", ErrorCode::Unauthorized))
            }
        }
    }

    with_engine_for_auth(TestHooks, |engine| async move {
        let response = engine
            .post(r###"query { check { authorizedEdgeWithNode(ids: ["1"]) { withId { id } } } }"###)
            .by_client("hi", "")
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "check": {
              "authorizedEdgeWithNode": {
                "withId": {
                  "id": "1"
                }
              }
            }
          }
        }
        "###);

        let response = engine
            .post(r###"query { check { authorizedEdgeWithNode(ids: ["1"]) { withId { id } } } }"###)
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Missing client",
              "path": [
                "check",
                "authorizedEdgeWithNode",
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
        async fn authorize_edge_node_post_execution(
            &self,
            _context: &DynHookContext,
            _definition: EdgeDefinition<'_>,
            _nodes: Vec<serde_json::Value>,
            _metadata: Option<serde_json::Value>,
        ) -> Result<Vec<Result<(), GraphqlError>>, GraphqlError> {
            Err(GraphqlError::new("Broken", ErrorCode::HookError))
        }
    }

    with_engine_for_auth(TestHooks, |engine| async move {
        let response = engine
            .post(
                r#"
                query {
                    check {
                        authorizedEdgeWithNode(ids: ["1"]) { withId { id } }
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
                "authorizedEdgeWithNode",
                "withId"
              ],
              "extensions": {
                "code": "HOOK_ERROR"
              }
            }
          ]
        }
        "###);

        let response = engine
            .post(
                r#"
                query {
                    check {
                        authorizedEdgeWithNode(ids: ["1"]) { nullableWithId { id } }
                    }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "check": {
              "authorizedEdgeWithNode": {
                "nullableWithId": null
              }
            }
          },
          "errors": [
            {
              "message": "Broken",
              "path": [
                "check",
                "authorizedEdgeWithNode",
                "nullableWithId"
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

#[test]
fn lists() {
    struct TestHooks;

    #[async_trait::async_trait]
    impl DynHooks for TestHooks {
        async fn authorize_edge_node_post_execution(
            &self,
            _context: &DynHookContext,
            _definition: EdgeDefinition<'_>,
            nodes: Vec<serde_json::Value>,
            _metadata: Option<serde_json::Value>,
        ) -> Result<Vec<Result<(), GraphqlError>>, GraphqlError> {
            Ok(nodes
                .into_iter()
                .map(|value| {
                    if value["id"].as_str().unwrap().len() <= 1 {
                        Ok(())
                    } else {
                        Err(GraphqlError::new("Id too long!", ErrorCode::Unauthorized))
                    }
                })
                .collect())
        }
    }

    with_engine_for_auth(TestHooks, |engine| async move {
        let response = engine
            .post(
                r#"
                query {
                    goodIds: check {
                       authorizedEdgeWithNode(ids: ["1", "7"]) {
                           listWithId { id }
                           listNullableWithId { id }
                           listListWithId { id }
                           listNullableListWithId { id }
                           listListNullableWithId { id }
                       }
                    }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "goodIds": {
              "authorizedEdgeWithNode": {
                "listWithId": [
                  {
                    "id": "1"
                  },
                  {
                    "id": "7"
                  }
                ],
                "listNullableWithId": [
                  {
                    "id": "1"
                  },
                  {
                    "id": "7"
                  }
                ],
                "listListWithId": [
                  [
                    {
                      "id": "1"
                    }
                  ],
                  [
                    {
                      "id": "7"
                    }
                  ]
                ],
                "listNullableListWithId": [
                  [
                    {
                      "id": "1"
                    }
                  ],
                  [
                    {
                      "id": "7"
                    }
                  ]
                ],
                "listListNullableWithId": [
                  [
                    {
                      "id": "1"
                    }
                  ],
                  [
                    {
                      "id": "7"
                    }
                  ]
                ]
              }
            }
          }
        }
        "###);

        let response = engine
            .post(
                r#"
                query {
                    check {
                       authorizedEdgeWithNode(ids: ["1", "10", "7"]) {
                           listWithId { id }
                       }
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
              "message": "Id too long!",
              "path": [
                "check",
                "authorizedEdgeWithNode",
                "listWithId",
                1
              ],
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "###);

        let response = engine
            .post(
                r#"
                query {
                    check {
                       authorizedEdgeWithNode(ids: ["1", "10", "7"]) {
                           listListWithId { id }
                       }
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
              "message": "Id too long!",
              "path": [
                "check",
                "authorizedEdgeWithNode",
                "listListWithId",
                1,
                0
              ],
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "###);

        let response = engine
            .post(
                r#"
                query {
                    check {
                       authorizedEdgeWithNode(ids: ["1", "10", "7"]) {
                           listNullableWithId { id }
                       }
                    }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "check": {
              "authorizedEdgeWithNode": {
                "listNullableWithId": [
                  {
                    "id": "1"
                  },
                  null,
                  {
                    "id": "7"
                  }
                ]
              }
            }
          },
          "errors": [
            {
              "message": "Id too long!",
              "path": [
                "check",
                "authorizedEdgeWithNode",
                "listNullableWithId",
                1
              ],
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "###);

        let response = engine
            .post(
                r#"
                query {
                    check {
                       authorizedEdgeWithNode(ids: ["1", "10", "7"]) {
                           listNullableListWithId { id }
                       }
                    }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "check": {
              "authorizedEdgeWithNode": {
                "listNullableListWithId": [
                  [
                    {
                      "id": "1"
                    }
                  ],
                  null,
                  [
                    {
                      "id": "7"
                    }
                  ]
                ]
              }
            }
          },
          "errors": [
            {
              "message": "Id too long!",
              "path": [
                "check",
                "authorizedEdgeWithNode",
                "listNullableListWithId",
                1,
                0
              ],
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "###);
        let response = engine
            .post(
                r#"
                query {
                    check {
                       authorizedEdgeWithNode(ids: ["1", "10", "7"]) {
                           listListNullableWithId { id }
                       }
                    }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "check": {
              "authorizedEdgeWithNode": {
                "listListNullableWithId": [
                  [
                    {
                      "id": "1"
                    }
                  ],
                  [
                    null
                  ],
                  [
                    {
                      "id": "7"
                    }
                  ]
                ]
              }
            }
          },
          "errors": [
            {
              "message": "Id too long!",
              "path": [
                "check",
                "authorizedEdgeWithNode",
                "listListNullableWithId",
                1,
                0
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
