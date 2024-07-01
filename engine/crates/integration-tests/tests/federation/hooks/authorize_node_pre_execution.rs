use http::HeaderMap;
use runtime::{
    error::GraphqlError,
    hooks::{DynHookContext, DynHooks, NodeDefinition},
};

use super::with_engine_for_auth;

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
        async fn authorize_node_pre_execution(
            &self,
            _context: &DynHookContext,
            _definition: NodeDefinition<'_>,
            metadata: Option<serde_json::Value>,
        ) -> Result<(), GraphqlError> {
            if extract_role(metadata.as_ref()) == Some("admin") {
                Ok(())
            } else {
                Err("Unauthorized role".into())
            }
        }
    }

    with_engine_for_auth(TestHooks, |engine| async move {
        let response = engine
            .execute(
                r#"
                query {
                    node {
                        ok: nullableAuthorizedWithMetadata {
                            id
                        }
                        noMetadata: nullableAuthorized {
                            id
                        }
                    }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "node": {
              "ok": {
                "id": "2b"
              },
              "noMetadata": null
            }
          },
          "errors": [
            {
              "message": "Unauthorized role",
              "path": [
                "node",
                "noMetadata"
              ]
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
        async fn authorize_node_pre_execution(
            &self,
            _context: &DynHookContext,
            definition: NodeDefinition<'_>,
            _metadata: Option<serde_json::Value>,
        ) -> Result<(), GraphqlError> {
            if definition.type_name == "AuthorizedNode" {
                Ok(())
            } else {
                Err("Wrong definition".into())
            }
        }
    }

    with_engine_for_auth(TestHooks, |engine| async move {
        let response = engine
            .execute(
                r#"
                query {
                    node {
                        ok: nullableAuthorized {
                            id
                        }
                        wrongType: nullableAuthorizedWithMetadata {
                            id
                        }
                    }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "node": {
              "ok": {
                "id": "1b"
              },
              "wrongType": null
            }
          },
          "errors": [
            {
              "message": "Wrong definition",
              "path": [
                "node",
                "wrongType"
              ]
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
        ) -> Result<HeaderMap, GraphqlError> {
            if let Some(client) = headers
                .get("x-grafbase-client-name")
                .and_then(|value| value.to_str().ok())
            {
                context.insert("client", client);
            }
            Ok(headers)
        }

        async fn authorize_node_pre_execution(
            &self,
            context: &DynHookContext,
            _definition: NodeDefinition<'_>,
            _metadata: Option<serde_json::Value>,
        ) -> Result<(), GraphqlError> {
            if context.get("client").is_some() {
                Ok(())
            } else {
                Err("Missing client".into())
            }
        }
    }

    with_engine_for_auth(TestHooks, |engine| async move {
        let response = engine
            .execute("query { node { authorized { id } } }")
            .by_client("hi", "")
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "node": {
              "authorized": {
                "id": "1a"
              }
            }
          }
        }
        "###);

        let response = engine.execute("query { node { authorized { id } } }").await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Missing client",
              "path": [
                "node",
                "authorized"
              ]
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
        async fn authorize_node_pre_execution(
            &self,
            _context: &DynHookContext,
            _definition: NodeDefinition<'_>,
            _metadata: Option<serde_json::Value>,
        ) -> Result<(), GraphqlError> {
            Err("Broken".into())
        }
    }

    with_engine_for_auth(TestHooks, |engine| async move {
        let response = engine
            .execute(
                r#"
                query {
                    node {
                        authorized {
                            id
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
              "message": "Broken",
              "path": [
                "node",
                "authorized"
              ]
            }
          ]
        }
        "###);
    });
}
