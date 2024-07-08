use http::HeaderMap;
use runtime::{
    error::{PartialErrorCode, PartialGraphqlError},
    hooks::{DynHookContext, DynHooks, EdgeDefinition},
};

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
        ) -> Result<(), PartialGraphqlError> {
            #[derive(serde::Deserialize)]
            struct Arguments {
                id: i64,
            }
            let Arguments { id } = serde_json::from_value(arguments).unwrap();
            if id < 100 {
                Err(PartialGraphqlError::new(
                    format!("Unauthorized ID: {id}"),
                    PartialErrorCode::Unauthorized,
                ))
            } else {
                Ok(())
            }
        }
    }

    with_engine_for_auth(TestHooks, |engine| async move {
        let response = engine
            .execute("query { check { authorizedWithId(id: 791) } }")
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

        let response = engine.execute("query { check { authorizedWithId(id: 0) } }").await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Unauthorized ID: 0",
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
        async fn authorize_edge_pre_execution(
            &self,
            _context: &DynHookContext,
            _definition: EdgeDefinition<'_>,
            _arguments: serde_json::Value,
            metadata: Option<serde_json::Value>,
        ) -> Result<(), PartialGraphqlError> {
            if extract_role(metadata.as_ref()) == Some("admin") {
                Ok(())
            } else {
                Err(PartialGraphqlError::new(
                    "Unauthorized role",
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
                        authorizedWithMetadata
                    }
                    noMetadata: nullableCheck {
                        authorized
                    }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r###"
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
        "###);
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
        ) -> Result<(), PartialGraphqlError> {
            if definition.parent_type_name == "Check" && definition.field_name == "authorized" {
                Ok(())
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
        insta::assert_json_snapshot!(response, @r###"
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

        async fn authorize_edge_pre_execution(
            &self,
            context: &DynHookContext,
            _definition: EdgeDefinition<'_>,
            _arguments: serde_json::Value,
            _metadata: Option<serde_json::Value>,
        ) -> Result<(), PartialGraphqlError> {
            if context.get("client").is_some() {
                Ok(())
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
            .execute("query { check { authorized } }")
            .by_client("hi", "")
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "check": {
              "authorized": "You have access"
            }
          }
        }
        "###);

        let response = engine.execute("query { check { authorized } }").await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Missing client",
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
        "###);
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
        ) -> Result<(), PartialGraphqlError> {
            Err(PartialGraphqlError::new("Broken", PartialErrorCode::HookError))
        }
    }

    with_engine_for_auth(TestHooks, |engine| async move {
        let response = engine
            .execute(
                r#"
                query {
                    check {
                        authorized
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
                "authorized"
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
