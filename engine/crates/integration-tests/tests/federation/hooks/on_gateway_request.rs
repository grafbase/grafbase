use engine_v2::Engine;
use graphql_mocks::{EchoSchema, FakeGithubSchema};
use http::HeaderMap;
use integration_tests::{federation::EngineV2Ext, runtime};
use runtime::{
    error::{ErrorResponse, PartialErrorCode, PartialGraphqlError},
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
        ) -> Result<HeaderMap, ErrorResponse> {
            headers.insert("b", "22".parse().unwrap());
            headers.remove("c");
            Ok(headers)
        }
    }

    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_mock_hooks(TestHooks)
            .with_subgraph(EchoSchema)
            .with_sdl_config(
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
fn error_is_propagated_back_to_the_user() {
    struct TestHooks;

    #[async_trait::async_trait]
    impl DynHooks for TestHooks {
        async fn on_gateway_request(
            &self,
            _context: &mut DynHookContext,
            _headers: HeaderMap,
        ) -> Result<HeaderMap, ErrorResponse> {
            let error =
                PartialGraphqlError::new("impossible error", PartialErrorCode::BadRequest).with_extension("foo", "bar");

            Err(ErrorResponse {
                status: http::StatusCode::BAD_REQUEST,
                errors: vec![error],
            })
        }
    }

    let response = runtime().block_on(async move {
        let engine = Engine::builder()
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
            _headers: HeaderMap,
        ) -> Result<HeaderMap, ErrorResponse> {
            let error = PartialGraphqlError::new("impossible error", PartialErrorCode::BadRequest)
                .with_extension("code", "IMPOSSIBLE");

            Err(ErrorResponse {
                status: http::StatusCode::BAD_REQUEST,
                errors: vec![error],
            })
        }
    }

    let response = runtime().block_on(async move {
        let engine = Engine::builder()
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
