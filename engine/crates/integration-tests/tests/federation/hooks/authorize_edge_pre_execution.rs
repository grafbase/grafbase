use engine_v2::Engine;
use futures::Future;
use graphql_mocks::{MockGraphQlServer, SecureSchema};
use http::HeaderMap;
use integration_tests::{
    federation::{GatewayV2Ext, TestFederationEngine},
    runtime,
};
use runtime::{
    error::GraphqlError,
    hooks::{DynHookContext, DynHooks, DynamicHooks, EdgeDefinition},
};

fn with_prepared_engine<F, O>(hooks: impl Into<DynamicHooks>, f: impl FnOnce(TestFederationEngine) -> F) -> O
where
    F: Future<Output = O>,
{
    runtime().block_on(async move {
        let secure_mock = MockGraphQlServer::new(SecureSchema::default()).await;

        let engine = Engine::builder()
            .with_schema("secure", &secure_mock)
            .await
            .with_hooks(hooks)
            .finish()
            .await;

        f(engine).await
    })
}

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
            _metadata: serde_json::Value,
        ) -> Result<(), GraphqlError> {
            #[derive(serde::Deserialize)]
            struct Arguments {
                id: i64,
            }
            let Arguments { id } = serde_json::from_value(arguments).unwrap();
            if id < 100 {
                Err("Unauthorized ID".into())
            } else {
                Ok(())
            }
        }
    }

    with_prepared_engine(TestHooks, |engine| async move {
        let response = engine
            .execute("query { check { sensitiveId(id: 791) } }")
            .by_client("hi", "")
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "check": {
              "sensitiveId": "You have access to the sensistive data"
            }
          }
        }
        "###);

        let response = engine.execute("query { check { sensitiveId(id: 0) } }").await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Unauthorized ID",
              "path": [
                "check",
                "sensitiveId"
              ]
            }
          ]
        }
        "###);
    });
}

#[test]
fn authorized_hook_is_called() {
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

        async fn authorize_edge_pre_execution(
            &self,
            context: &DynHookContext,
            _definition: EdgeDefinition<'_>,
            _arguments: serde_json::Value,
            _metadata: serde_json::Value,
        ) -> Result<(), GraphqlError> {
            if context.get("client").is_some() {
                Ok(())
            } else {
                Err("Missing client".into())
            }
        }
    }

    with_prepared_engine(TestHooks, |engine| async move {
        let response = engine
            .execute("query { check { grafbaseClientIsDefined } }")
            .by_client("hi", "")
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "check": {
              "grafbaseClientIsDefined": "You have properly set the x-grafbase-client-name header"
            }
          }
        }
        "###);

        let response = engine.execute("query { check { grafbaseClientIsDefined } }").await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Missing client",
              "path": [
                "check",
                "grafbaseClientIsDefined"
              ]
            }
          ]
        }
        "###);
    });
}
