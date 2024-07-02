use futures::StreamExt;
use http::HeaderMap;
use integration_tests::federation::DeterministicEngine;
use runtime::{
    error::GraphqlError,
    hooks::{DynHookContext, DynHooks, NodeDefinition},
};
use serde_json::json;

use super::with_engine_for_auth;

#[test]
fn query_root_type() {
    struct TestHooks;

    #[async_trait::async_trait]
    impl DynHooks for TestHooks {
        async fn authorize_node_pre_execution(
            &self,
            _context: &DynHookContext,
            definition: NodeDefinition<'_>,
            _metadata: Option<serde_json::Value>,
        ) -> Result<(), GraphqlError> {
            if definition.type_name == "Query" {
                Err("Query is not allowed!".into())
            } else {
                Ok(())
            }
        }
    }

    const SCHEMA: &str = r#"
        enum join__Graph {
          ACCOUNTS @join__graph(name: "accounts", url: "http://127.0.0.1:46697")
        }

        type Query @authorized {
            me: String @join__field(graph: ACCOUNTS)
        }

        type Mutation {
            doStuff: String @join__field(graph: ACCOUNTS)
        }
        "#;

    let response = integration_tests::runtime().block_on(
        DeterministicEngine::builder(
            SCHEMA,
            r#"
        query {
            me
        }
        "#,
        )
        .with_hooks(TestHooks)
        .with_subgraph_response(json!({"data": {"me": "Rusty"} }))
        .build()
        .execute(),
    );
    insta::assert_json_snapshot!(response, @r###"
    {
      "data": null,
      "errors": [
        {
          "message": "Query is not allowed!"
        }
      ]
    }
    "###);

    let response = integration_tests::runtime().block_on(
        DeterministicEngine::builder(
            SCHEMA,
            r#"
        mutation {
            doStuff
        }
        "#,
        )
        .with_hooks(TestHooks)
        .with_subgraph_response(json!({"data": {"doStuff": "done"} }))
        .build()
        .execute(),
    );
    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "doStuff": "done"
      }
    }
    "###);
}

#[test]
fn mutation_root_type() {
    struct TestHooks;

    #[async_trait::async_trait]
    impl DynHooks for TestHooks {
        async fn authorize_node_pre_execution(
            &self,
            _context: &DynHookContext,
            definition: NodeDefinition<'_>,
            _metadata: Option<serde_json::Value>,
        ) -> Result<(), GraphqlError> {
            if definition.type_name == "Mutation" {
                Err("Mutation is not allowed!".into())
            } else {
                Ok(())
            }
        }
    }

    const SCHEMA: &str = r#"
        enum join__Graph {
          ACCOUNTS @join__graph(name: "accounts", url: "http://127.0.0.1:46697")
        }

        type Query {
            me: String @join__field(graph: ACCOUNTS)
        }

        type Mutation @authorized {
            doStuff: String @join__field(graph: ACCOUNTS)
        }
        "#;

    let response = integration_tests::runtime().block_on(
        DeterministicEngine::builder(
            SCHEMA,
            r#"
        query {
            me
        }
        "#,
        )
        .with_hooks(TestHooks)
        .with_subgraph_response(json!({"data": {"me": "Rusty"} }))
        .build()
        .execute(),
    );
    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "me": "Rusty"
      }
    }
    "###);

    let response = integration_tests::runtime().block_on(
        DeterministicEngine::builder(
            SCHEMA,
            r#"
        mutation {
            doStuff
        }
        "#,
        )
        .with_hooks(TestHooks)
        .with_subgraph_response(json!({"data": {"doStuff": "done"} }))
        .build()
        .execute(),
    );
    insta::assert_json_snapshot!(response, @r###"
    {
      "data": null,
      "errors": [
        {
          "message": "Mutation is not allowed!"
        }
      ]
    }
    "###);
}

#[test]
fn subscription_root_type() {
    struct TestHooks;

    #[async_trait::async_trait]
    impl DynHooks for TestHooks {
        async fn authorize_node_pre_execution(
            &self,
            _context: &DynHookContext,
            definition: NodeDefinition<'_>,
            _metadata: Option<serde_json::Value>,
        ) -> Result<(), GraphqlError> {
            if definition.type_name == "Subscription" {
                Err("Subscription is not allowed!".into())
            } else {
                Ok(())
            }
        }
    }

    const SCHEMA: &str = r#"
        enum join__Graph {
          ACCOUNTS @join__graph(name: "accounts", url: "http://127.0.0.1:46697")
        }

        type Query {
            me: String @join__field(graph: ACCOUNTS)
        }

        type Subscription @authorized {
            doStuff: String @join__field(graph: ACCOUNTS)
        }
        "#;

    let response = integration_tests::runtime().block_on(
        DeterministicEngine::builder(
            SCHEMA,
            r#"
        query {
            me
        }
        "#,
        )
        .with_hooks(TestHooks)
        .with_subgraph_response(json!({"data": {"me": "Rusty"} }))
        .build()
        .execute(),
    );
    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "me": "Rusty"
      }
    }
    "###);

    let response = integration_tests::runtime().block_on(async {
        DeterministicEngine::builder(
            SCHEMA,
            r#"
        subscription {
            doStuff
        }
        "#,
        )
        .with_hooks(TestHooks)
        .with_subgraph_response(json!({"data": {"doStuff": "done"} }))
        .build()
        .execute_stream()
        .await
        .stream
        .collect::<Vec<_>>()
        .await
    });
    insta::assert_json_snapshot!(response, @r###"
    [
      {
        "data": null,
        "errors": [
          {
            "message": "Subscription is not allowed!"
          }
        ]
      }
    ]
    "###);
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
