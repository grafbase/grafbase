use engine::Engine;
use graphql_mocks::FakeGithubSchema;
use integration_tests::{federation::EngineExt, runtime};

#[test]
fn grafbase_extension_on_successful_request() {
    runtime().block_on(async move {
        let engine = Engine::builder().with_subgraph(FakeGithubSchema).build().await;

        let response = engine
            .post("query { serverVersion }")
            .header("x-grafbase-telemetry", "yes")
            .await;

        insta::assert_json_snapshot!(
            response,
            @r#"
            {
              "data": {
                "serverVersion": "1"
              },
              "extensions": {
                "grafbase": {
                  "traceId": "0"
                }
              }
            }
            "#
        );
    })
}

#[test]
fn grafbase_extension_on_invalid_request() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(FakeGithubSchema)
            .build()
            .await;

        let response = engine
            .post("query x }")
            .header("x-grafbase-telemetry", "yes")
            .await;

        insta::assert_json_snapshot!(
            response,
            @r#"
            {
              "errors": [
                {
                  "message": " --> 1:9\n  |\n1 | query x }\n  |         ^---\n  |\n  = expected variable_definitions, selection_set, or directive",
                  "locations": [
                    {
                      "line": 1,
                      "column": 9
                    }
                  ],
                  "extensions": {
                    "code": "OPERATION_PARSING_ERROR"
                  }
                }
              ],
              "extensions": {
                "grafbase": {
                  "traceId": "0"
                }
              }
            }
            "#
        );
    })
}

#[test]
fn grafbase_extension_secret_value() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(FakeGithubSchema)
            .with_toml_config(
                r#"
            [[telemetry.exporters.response_extension.access_control]]
            rule = "header"
            name = "dummy"
            value = "secret"
            "#,
            )
            .build()
            .await;

        let response = engine
            .post("query { serverVersion }")
            // shouldn't work anymore
            .header("x-grafbase-telemetry", "yes")
            .await;

        insta::assert_json_snapshot!(
            response,
            @r#"
            {
              "data": {
                "serverVersion": "1"
              }
            }
            "#
        );

        let response = engine
            .post("query { serverVersion }")
            // not the right value
            .header("dummy", "yes")
            .await;

        insta::assert_json_snapshot!(
            response,
            @r#"
            {
              "data": {
                "serverVersion": "1"
              }
            }
            "#
        );
        let response = engine.post("query { serverVersion }").header("dummy", "secret").await;

        insta::assert_json_snapshot!(
            response,
            @r#"
            {
              "data": {
                "serverVersion": "1"
              },
              "extensions": {
                "grafbase": {
                  "traceId": "0"
                }
              }
            }
            "#
        );
    })
}

#[test]
fn grafbase_extension_denied() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(FakeGithubSchema)
            .with_toml_config(
                r#"
            [[telemetry.exporters.response_extension.access_control]]
            rule = "deny"
            "#,
            )
            .build()
            .await;

        let response = engine
            .post("query { serverVersion }")
            // shouldn't work anymore
            .header("x-grafbase-telemetry", "yes")
            .await;

        insta::assert_json_snapshot!(
            response,
            @r#"
            {
              "data": {
                "serverVersion": "1"
              }
            }
            "#
        );
    })
}

#[test]
fn grafbase_extension_on_ill_formed_graphql_over_http_request() {
    runtime().block_on(async move {
        let engine = Engine::builder().with_subgraph(FakeGithubSchema).build().await;

        let response = engine
            .raw_execute(
                http::Request::builder()
                    .uri("http://localhost/graphql")
                    .method(http::Method::POST)
                    .header(
                        http::HeaderName::from_static("x-grafbase-telemetry"),
                        http::HeaderValue::from_static(""),
                    )
                    .header(http::header::CONTENT_TYPE, "application/json")
                    .header(http::header::ACCEPT, "application/graphql-response+json")
                    .body(Vec::from(br###"{}"###))
                    .unwrap(),
            )
            .await;
        let status = response.status();
        let body: serde_json::Value = serde_json::from_slice(&response.into_body()).unwrap();
        insta::assert_json_snapshot!(body, @r#"
        {
          "errors": [
            {
              "message": "Missing query",
              "extensions": {
                "code": "BAD_REQUEST"
              }
            }
          ],
          "extensions": {
            "grafbase": {
              "traceId": "0"
            }
          }
        }
        "#);
        assert_eq!(status, 400);
    })
}
