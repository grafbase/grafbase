use engine::{ErrorCode, ErrorResponse, GraphqlError};
use graphql_mocks::SecureSchema;
use http::HeaderMap;
use integration_tests::gateway::{DynHookContext, DynHooks};
use runtime::hooks::EdgeDefinition;

use super::with_engine_for_auth;

#[test]
fn arguments_are_provided() {
    struct TestHooks;

    #[async_trait::async_trait]
    impl DynHooks for TestHooks {
        async fn authorize_edge_pre_execution(
            &self,
            _context: &DynHookContext,
            _definition: EdgeDefinition<'_>,
            arguments: serde_json::Value,
            _metadata: Option<serde_json::Value>,
        ) -> Result<(), GraphqlError> {
            #[derive(serde::Deserialize)]
            struct Arguments {
                id: i64,
            }
            let Arguments { id } = serde_json::from_value(arguments).unwrap();
            if id < 100 {
                Err(GraphqlError::new(
                    format!("Unauthorized ID: {id}"),
                    ErrorCode::Unauthorized,
                ))
            } else {
                Ok(())
            }
        }
    }

    with_engine_for_auth(TestHooks, |engine| async move {
        let response = engine
            .post("query { check { authorizedWithId(id: 791) } }")
            .by_client("hi", "")
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "check": {
              "authorizedWithId": "You have access to the sensistive data"
            }
          }
        }
        "###);

        let response = engine.post("query { check { authorizedWithId(id: 0) } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": null,
          "errors": [
            {
              "message": "Unauthorized ID: 0",
              "locations": [
                {
                  "line": 1,
                  "column": 17
                }
              ],
              "path": [
                "check",
                "authorizedWithId"
              ],
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "#);

        // We shouldn't have requested the field.
        let requests = engine.drain_graphql_requests_sent_to::<SecureSchema>();
        insta::assert_json_snapshot!(requests, @r#"
        [
          {
            "query": "query($var0: Int!) { check { authorizedWithId(id: $var0) } }",
            "operationName": null,
            "variables": {
              "var0": 791
            },
            "extensions": {}
          },
          {
            "query": "query { check { __typename @skip(if: true) } }",
            "operationName": null,
            "variables": {},
            "extensions": {}
          }
        ]
        "#);
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
        async fn authorize_edge_pre_execution(
            &self,
            _context: &DynHookContext,
            _definition: EdgeDefinition<'_>,
            _arguments: serde_json::Value,
            metadata: Option<serde_json::Value>,
        ) -> Result<(), GraphqlError> {
            if extract_role(metadata.as_ref()) == Some("admin") {
                Ok(())
            } else {
                Err(GraphqlError::new("Unauthorized role", ErrorCode::Unauthorized))
            }
        }
    }

    with_engine_for_auth(TestHooks, |engine| async move {
        let response = engine
            .post(
                r#"
                query {
                    ok: nullableCheck {
                        authorizedWithMetadata
                    }
                    noMetadata: nullableCheck {
                        authorized
                    }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "ok": {
              "authorizedWithMetadata": "You have access"
            },
            "noMetadata": null
          },
          "errors": [
            {
              "message": "Unauthorized role",
              "locations": [
                {
                  "line": 7,
                  "column": 25
                }
              ],
              "path": [
                "noMetadata",
                "authorized"
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
        async fn authorize_edge_pre_execution(
            &self,
            _context: &DynHookContext,
            definition: EdgeDefinition<'_>,
            _arguments: serde_json::Value,
            _metadata: Option<serde_json::Value>,
        ) -> Result<(), GraphqlError> {
            if definition.parent_type_name == "Check" && definition.field_name == "authorized" {
                Ok(())
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
                        authorized
                    }
                    wrongField: nullableCheck {
                        authorizedWithMetadata
                    }
                    wrongType: nullableOtherCheck {
                        authorized
                    }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "ok": {
              "authorized": "You have access"
            },
            "wrongField": null,
            "wrongType": null
          },
          "errors": [
            {
              "message": "Wrong definition",
              "locations": [
                {
                  "line": 7,
                  "column": 25
                }
              ],
              "path": [
                "wrongField",
                "authorizedWithMetadata"
              ],
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            },
            {
              "message": "Wrong definition",
              "locations": [
                {
                  "line": 10,
                  "column": 25
                }
              ],
              "path": [
                "wrongType",
                "authorized"
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

        async fn authorize_edge_pre_execution(
            &self,
            context: &DynHookContext,
            _definition: EdgeDefinition<'_>,
            _arguments: serde_json::Value,
            _metadata: Option<serde_json::Value>,
        ) -> Result<(), GraphqlError> {
            if context.get("client").is_some() {
                Ok(())
            } else {
                Err(GraphqlError::new("Missing client", ErrorCode::Unauthorized))
            }
        }
    }

    with_engine_for_auth(TestHooks, |engine| async move {
        let response = engine.post("query { check { authorized } }").by_client("hi", "").await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "check": {
              "authorized": "You have access"
            }
          }
        }
        "###);

        let response = engine.post("query { check { authorized } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": null,
          "errors": [
            {
              "message": "Missing client",
              "locations": [
                {
                  "line": 1,
                  "column": 17
                }
              ],
              "path": [
                "check",
                "authorized"
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
fn error_propagation() {
    struct TestHooks;

    #[async_trait::async_trait]
    impl DynHooks for TestHooks {
        async fn authorize_edge_pre_execution(
            &self,
            _context: &DynHookContext,
            _definition: EdgeDefinition<'_>,
            _arguments: serde_json::Value,
            _metadata: Option<serde_json::Value>,
        ) -> Result<(), GraphqlError> {
            Err(GraphqlError::new("Broken", ErrorCode::HookError))
        }
    }

    with_engine_for_auth(TestHooks, |engine| async move {
        let response = engine
            .post(
                r#"
                query {
                    check {
                        authorized
                    }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": null,
          "errors": [
            {
              "message": "Broken",
              "locations": [
                {
                  "line": 4,
                  "column": 25
                }
              ],
              "path": [
                "check",
                "authorized"
              ],
              "extensions": {
                "code": "HOOK_ERROR"
              }
            }
          ]
        }
        "#);
    });
}
