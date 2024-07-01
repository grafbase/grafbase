use engine_v2::Engine;
use graphql_mocks::{EchoSchema, MockGraphQlServer};
use http::HeaderMap;
use integration_tests::{federation::GatewayV2Ext, runtime};
use runtime::{
    error::GraphqlError,
    hooks::{DynHookContext, DynHooks},
};

#[test]
fn can_modify_headers() {
    struct TestHooks;

    #[async_trait::async_trait]
    impl DynHooks for TestHooks {
        async fn on_gateway_request(
            &self,
            _context: &mut DynHookContext,
            mut headers: HeaderMap,
        ) -> Result<HeaderMap, GraphqlError> {
            headers.insert("b", "22".parse().unwrap());
            headers.remove("c");
            Ok(headers)
        }
    }

    let response = runtime().block_on(async move {
        let echo_mock = MockGraphQlServer::new(EchoSchema).await;

        let engine = Engine::builder()
            .with_hooks(TestHooks)
            .with_schema("echo", &echo_mock)
            .await
            .with_supergraph_config(
                r#"
                extend schema @subgraph(
                    name: "echo",
                    headers: [
                        { name: "a", forward: "a" },
                        { name: "b", forward: "b" },
                        { name: "c", forward: "c" }
                    ]
                )
            "#,
            )
            .finish()
            .await;

        engine
            .execute(
                r###"
            query {
                unknown: header(name: "unknown")
                a: header(name: "a")
                b: header(name: "b")
                c: header(name: "c")
            }
            "###,
            )
            .header("a", "1")
            .header("b", "2")
            .header("c", "3")
            .await
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "unknown": null,
        "a": "1",
        "b": "22",
        "c": null
      }
    }
    "###);
}

#[test]
fn error_is_propagated_back_to_the_user() {
    struct TestHooks;

    #[async_trait::async_trait]
    impl DynHooks for TestHooks {
        async fn on_gateway_request(
            &self,
            _context: &mut DynHookContext,
            _headers: HeaderMap,
        ) -> Result<HeaderMap, GraphqlError> {
            Err(GraphqlError::new("impossible error").with_extension("foo", "bar"))
        }
    }

    let response = runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(EchoSchema).await;

        let engine = Engine::builder()
            .with_hooks(TestHooks)
            .with_schema("schema", &github_mock)
            .await
            .finish()
            .await;

        engine.execute("query { serverVersion }").await
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": null,
      "errors": [
        {
          "message": "impossible error",
          "extensions": {
            "foo": "bar"
          }
        }
      ]
    }
    "###);
}
