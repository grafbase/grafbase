use http::HeaderMap;
use runtime::{
    error::{ErrorResponse, PartialErrorCode, PartialGraphqlError},
    hooks::{DynHookContext, DynHooks, EdgeDefinition},
};

use super::with_engine_for_auth;

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
        ) -> Result<Vec<Result<(), PartialGraphqlError>>, PartialGraphqlError> {
            Ok(nodes
                .into_iter()
                .map(|value| {
                    if value["id"] == "1" {
                        Ok(())
                    } else {
                        Err(PartialGraphqlError::new(
                            "Unauthorized id",
                            PartialErrorCode::Unauthorized,
                        ))
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
        ) -> Result<Vec<Result<(), PartialGraphqlError>>, PartialGraphqlError> {
            let Some(role) = extract_role(metadata.as_ref()) else {
                return Err(PartialGraphqlError::new(
                    "Unauthorized role",
                    PartialErrorCode::Unauthorized,
                ));
            };
            Ok(nodes
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
        insta::assert_json_snapshot!(response, @r###"
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
        "###);
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
        ) -> Result<Vec<Result<(), PartialGraphqlError>>, PartialGraphqlError> {
            if definition.parent_type_name == "AuthorizedEdgeWithNode" && definition.field_name == "withId" {
                Ok(vec![Ok(()); nodes.len()])
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
        ) -> Result<Vec<Result<(), PartialGraphqlError>>, PartialGraphqlError> {
            if context.get("client").is_some() {
                Ok(vec![Ok(()); nodes.len()])
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
        ) -> Result<Vec<Result<(), PartialGraphqlError>>, PartialGraphqlError> {
            Err(PartialGraphqlError::new("Broken", PartialErrorCode::HookError))
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
        ) -> Result<Vec<Result<(), PartialGraphqlError>>, PartialGraphqlError> {
            Ok(nodes
                .into_iter()
                .map(|value| {
                    if value["id"].as_str().unwrap().len() <= 1 {
                        Ok(())
                    } else {
                        Err(PartialGraphqlError::new("Id too long!", PartialErrorCode::Unauthorized))
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
