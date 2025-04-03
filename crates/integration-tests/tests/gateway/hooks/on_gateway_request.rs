use engine::{ErrorCode, ErrorResponse, GraphqlError};
use graphql_mocks::{EchoSchema, FakeGithubSchema};
use http::HeaderMap;
use integration_tests::gateway::{DynHookContext, DynHooks};
use integration_tests::{gateway::Gateway, runtime};

#[test]
fn can_modify_headers() {
    struct TestHooks;

    #[async_trait::async_trait]
    impl DynHooks for TestHooks {
        async fn on_gateway_request(
            &self,
            _context: &mut DynHookContext,
            url: &str,
            mut headers: HeaderMap,
        ) -> Result<HeaderMap, ErrorResponse> {
            headers.insert("b", "22".parse().unwrap());
            headers.remove("c");
            headers.insert("url", url.parse().unwrap());
            Ok(headers)
        }
    }

    let response = runtime().block_on(async move {
        let config = indoc::formatdoc! {r#"
            [[subgraphs.echo.headers]]
            rule = "forward"
            pattern = ".*"
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
                url: header(name: "url")
            }
            "###,
            )
            .header("a", "1")
            .header("b", "2")
            .header("c", "3")
            .await
    });

    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "unknown": null,
        "a": "1",
        "b": "22",
        "c": null,
        "url": "http://127.0.0.1/graphql"
      }
    }
    "#);
}

#[test]
fn error_is_propagated_back_to_the_user() {
    struct TestHooks;

    #[async_trait::async_trait]
    impl DynHooks for TestHooks {
        async fn on_gateway_request(
            &self,
            _context: &mut DynHookContext,
            _url: &str,
            _headers: HeaderMap,
        ) -> Result<HeaderMap, ErrorResponse> {
            let error = GraphqlError::new("impossible error", ErrorCode::BadRequest).with_extension("foo", "bar");

            Err(ErrorResponse {
                status: http::StatusCode::BAD_REQUEST,
                errors: vec![error],
            })
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

    insta::assert_json_snapshot!(response, @r###"
    {
      "errors": [
        {
          "message": "impossible error",
          "extensions": {
            "foo": "bar",
            "code": "BAD_REQUEST"
          }
        }
      ]
    }
    "###);
}

#[test]
fn error_code_is_propagated_back_to_the_user() {
    struct TestHooks;

    #[async_trait::async_trait]
    impl DynHooks for TestHooks {
        async fn on_gateway_request(
            &self,
            _context: &mut DynHookContext,
            _url: &str,
            _headers: HeaderMap,
        ) -> Result<HeaderMap, ErrorResponse> {
            let error =
                GraphqlError::new("impossible error", ErrorCode::BadRequest).with_extension("code", "IMPOSSIBLE");

            Err(ErrorResponse {
                status: http::StatusCode::BAD_REQUEST,
                errors: vec![error],
            })
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

    insta::assert_json_snapshot!(response, @r###"
    {
      "errors": [
        {
          "message": "impossible error",
          "extensions": {
            "code": "IMPOSSIBLE"
          }
        }
      ]
    }
    "###);
}
