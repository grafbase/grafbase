use engine_v2::Engine;
use graphql_mocks::{EchoSchema, FakeGithubSchema};
use http::HeaderMap;
use integration_tests::{federation::EngineV2Ext, runtime};
use runtime::{
    error::{PartialErrorCode, PartialGraphqlError},
    hooks::{DynHookContext, DynHooks},
};
use runtime_local::HooksWasiConfig;
use url::Url;

#[test]
fn wasi() {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};

    let mut response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_wasi_hooks(HooksWasiConfig::default().with_location("subgraph_request.wasm"))
            .with_subgraph(EchoSchema)
            .with_sdl_config(
                r#"
                    extend schema @subgraph(
                        name: "echo",
                        headers: [
                            { name: "hi", forward: "hi" },
                        ]
                    )
                "#,
            )
            .build()
            .await;

        engine
            .execute(
                r###"
            query {
                hi: header(name: "hi")
                everything: header(name: "everything")
            }
            "###,
            )
            .header("hi", "Rusty")
            .await
    });
    let value = response["data"]["everything"].as_str().unwrap();
    let value: serde_json::Value = serde_json::from_slice(&URL_SAFE_NO_PAD.decode(value).unwrap()).unwrap();
    response["data"]["everything"] = value;

    let url_redaction = insta::dynamic_redaction(|value, _path| {
        let url = value.as_str().unwrap();
        assert!(url.starts_with("http://127.0.0.1:") || url.starts_with("http://localhost:"));
        "[url]"
    });
    // the content of everything has no particular order.
    insta::with_settings!({sort_maps => true}, {
        insta::assert_json_snapshot!(
            response,
            {
                ".data.everything.url" => url_redaction,
            },
            @r###"
        {
          "data": {
            "everything": {
              "headers": [
                [
                  "hi",
                  "Rusty"
                ]
              ],
              "method": "POST",
              "subgraph_name": "echo",
              "url": "[url]"
            },
            "hi": "Rusty"
          }
        }
        "###
        );
    });
}

#[test]
fn can_modify_headers() {
    struct TestHooks;

    #[async_trait::async_trait]
    impl DynHooks for TestHooks {
        async fn on_subgraph_request(
            &self,
            _context: &DynHookContext,
            _subgraph_name: &str,
            _method: http::Method,
            _url: &Url,
            mut headers: HeaderMap,
        ) -> Result<HeaderMap, PartialGraphqlError> {
            headers.insert("b", "22".parse().unwrap());
            headers.remove("c");
            Ok(headers)
        }
    }

    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_hooks(TestHooks)
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
        async fn on_subgraph_request(
            &self,
            _context: &DynHookContext,
            _subgraph_name: &str,
            _method: http::Method,
            _url: &Url,
            _headers: HeaderMap,
        ) -> Result<HeaderMap, PartialGraphqlError> {
            Err(PartialGraphqlError::new("impossible error", PartialErrorCode::HookError).with_extension("foo", "bar"))
        }
    }

    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_hooks(TestHooks)
            .with_subgraph(FakeGithubSchema)
            .build()
            .await;

        engine.execute("query { serverVersion }").await
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": null,
      "errors": [
        {
          "message": "impossible error",
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
    "###);
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
            _method: http::Method,
            _url: &Url,
            _headers: HeaderMap,
        ) -> Result<HeaderMap, PartialGraphqlError> {
            Err(
                PartialGraphqlError::new("impossible error", PartialErrorCode::HookError)
                    .with_extension("code", "IMPOSSIBLE"),
            )
        }
    }

    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_hooks(TestHooks)
            .with_subgraph(FakeGithubSchema)
            .build()
            .await;

        engine.execute("query { serverVersion }").await
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": null,
      "errors": [
        {
          "message": "impossible error",
          "path": [
            "serverVersion"
          ],
          "extensions": {
            "code": "IMPOSSIBLE"
          }
        }
      ]
    }
    "###);
}
