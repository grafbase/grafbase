use engine::{ErrorCode, GraphqlError};
use graphql_mocks::{EchoSchema, FakeGithubSchema, Stateful, Subgraph};
use integration_tests::gateway::{DynHookContext, DynHooks};
use integration_tests::{gateway::Gateway, runtime};
use runtime::hooks::SubgraphRequest;
use url::Url;

#[test]
fn can_modify_headers() {
    struct TestHooks;

    #[async_trait::async_trait]
    impl DynHooks for TestHooks {
        async fn on_subgraph_request(
            &self,
            _context: &DynHookContext,
            _subgraph_name: &str,
            mut request: SubgraphRequest,
        ) -> Result<SubgraphRequest, GraphqlError> {
            request.headers.insert("b", "22".parse().unwrap());
            request.headers.remove("c");
            Ok(request)
        }
    }

    let response = runtime().block_on(async move {
        let config = indoc::formatdoc! {r#"
            [[subgraphs.echo.headers]]
            rule = "forward"
            name = "a"

            [[subgraphs.echo.headers]]
            rule = "forward"
            name = "b"

            [[subgraphs.echo.headers]]
            rule = "forward"
            name = "c"
        "#};

        let engine = Gateway::builder()
            .with_mock_hooks(TestHooks)
            .with_subgraph(EchoSchema)
            .with_toml_config(config)
            .build()
            .await;

        engine
            .post(
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
fn can_modify_url() {
    runtime().block_on(async {
        let subgraph = Stateful::default().start().await;

        struct TestHooks {
            url: Url,
        }

        #[async_trait::async_trait]
        impl DynHooks for TestHooks {
            async fn on_subgraph_request(
                &self,
                _context: &DynHookContext,
                _subgraph_name: &str,
                mut request: SubgraphRequest,
            ) -> Result<SubgraphRequest, GraphqlError> {
                if request.headers.contains_key("redirect") {
                    request.url = self.url.clone();
                }
                Ok(request)
            }
        }

        let engine = Gateway::builder()
            .with_mock_hooks(TestHooks { url: subgraph.url() })
            .with_subgraph(Stateful::default())
            .with_toml_config(
                r#"
                [[subgraphs.stateful.headers]]
                rule = "forward"
                name = "redirect"
                "#,
            )
            .build()
            .await;

        let response = engine
            .post(
                r###"
                mutation {
                    add(val: 1)
                }
            "###,
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "add": 1
          }
        }
        "#);

        let response = engine
            .post(
                r###"
                mutation {
                    add(val: 7)
                }
            "###,
            )
            .header("redirect", "1")
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "add": 7
          }
        }
        "#);
    });
}

#[test]
fn error_is_propagated_back_to_the_user() {
    struct TestHooks;

    #[async_trait::async_trait]
    impl DynHooks for TestHooks {
        async fn on_subgraph_request(
            &self,
            _context: &DynHookContext,
            _subgraph_name: &str,
            _request: SubgraphRequest,
        ) -> Result<SubgraphRequest, GraphqlError> {
            Err(GraphqlError::new("impossible error", ErrorCode::HookError).with_extension("foo", "bar"))
        }
    }

    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_mock_hooks(TestHooks)
            .with_subgraph(FakeGithubSchema)
            .build()
            .await;

        engine.post("query { serverVersion }").await
    });

    insta::assert_json_snapshot!(response, @r#"
    {
      "data": null,
      "errors": [
        {
          "message": "impossible error",
          "locations": [
            {
              "line": 1,
              "column": 9
            }
          ],
          "path": [
            "serverVersion"
          ],
          "extensions": {
            "foo": "bar",
            "code": "HOOK_ERROR"
          }
        }
      ]
    }
    "#);
}

#[test]
fn error_code_is_propagated_back_to_the_user() {
    struct TestHooks;

    #[async_trait::async_trait]
    impl DynHooks for TestHooks {
        async fn on_subgraph_request(
            &self,
            _context: &DynHookContext,
            _subgraph_name: &str,
            _request: SubgraphRequest,
        ) -> Result<SubgraphRequest, GraphqlError> {
            Err(GraphqlError::new("impossible error", ErrorCode::HookError).with_extension("code", "IMPOSSIBLE"))
        }
    }

    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_mock_hooks(TestHooks)
            .with_subgraph(FakeGithubSchema)
            .build()
            .await;

        engine.post("query { serverVersion }").await
    });

    insta::assert_json_snapshot!(response, @r#"
    {
      "data": null,
      "errors": [
        {
          "message": "impossible error",
          "locations": [
            {
              "line": 1,
              "column": 9
            }
          ],
          "path": [
            "serverVersion"
          ],
          "extensions": {
            "code": "IMPOSSIBLE"
          }
        }
      ]
    }
    "#);
}
