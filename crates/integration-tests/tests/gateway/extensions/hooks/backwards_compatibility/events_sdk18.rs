use base64::Engine;
use graphql_mocks::EchoSchema;
use indoc::formatdoc;
use integration_tests::{gateway::Gateway, runtime};
use tempfile::TempDir;

#[test]
fn receive_events() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_toml_config(
                r#"
                [extensions.hooks-17]
                path = "./data/extensions/crates/hooks/build"
                stdout = true
                stderr = true

                [[headers]]
                rule = "forward"
                name = "x-incoming-header"

                [extensions.hooks-17.config]
                events_header_name = "x-events"
            "#,
            )
            .with_extension("hooks-17")
            .with_subgraph(EchoSchema)
            .build()
            .await;

        engine.post(r#"query { header(name: "dummy") }"#).await
    });

    let bytes = response
        .headers
        .get("x-events")
        .map(|v| v.as_bytes())
        .unwrap_or_default();

    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(bytes)
        .unwrap_or_default();

    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

    insta::assert_json_snapshot!(value,
        {
            "[].duration_ms" => "[redacted]",
            "[].prepare_duration_ms" => "[redacted]",
            "[].executions[].connection_time_ms" => "[redacted]",
            "[].executions[].response_time_ms" => "[redacted]",
            "[].total_duration_ms" => "[redacted]",
            "[].url" => "[redacted]"
        },
        @r#"
    [
      {
        "cache_status": "miss",
        "executions": [
          {
            "connection_time_ms": "[redacted]",
            "response_time_ms": "[redacted]",
            "status_code": 200,
            "type": "response"
          }
        ],
        "has_errors": false,
        "method": "POST",
        "subgraph_name": "echo",
        "total_duration_ms": "[redacted]",
        "type": "subgraph",
        "url": "[redacted]"
      },
      {
        "cached_plan": false,
        "document": "query { header(name: \"\") }",
        "duration_ms": "[redacted]",
        "name": null,
        "prepare_duration_ms": "[redacted]",
        "status": {
          "type": "success"
        },
        "type": "operation"
      },
      {
        "method": "POST",
        "status_code": 200,
        "type": "http",
        "url": "[redacted]"
      }
    ]
    "#
    );
}

#[test]
fn access_logs_with_working_subgraph() {
    let tmpdir = TempDir::new().unwrap();
    let path = tmpdir.path().to_str().unwrap();

    let config = formatdoc! {r#"
        [extensions.access-logs-18]
        path = "./data/extensions/crates/access-logs/build"
        stdout = true
        stderr = true

        [extensions.access-logs-18.config]
        path = "{path}/access.log"

        [telemetry.tracing.propagation]
        trace_context = true

        [telemetry]
        service_name = "access-log-test"
    "#};

    let mut response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_toml_config(config)
            .with_extension("access-logs-18")
            .with_subgraph(EchoSchema)
            .build()
            .await;

        engine
            .post(r#"query Simple { responseHeader(name: "X-Special", value: "kekw") }"#)
            .header("traceparent", "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01")
            .await
    });

    insta::assert_json_snapshot!(response.take(), @r#"
    {
      "data": {
        "responseHeader": null
      }
    }
    "#);

    let result = std::fs::read_to_string(tmpdir.path().join("access.log")).unwrap();

    let line = result
        .split('\n')
        .next()
        .map(|s| serde_json::from_str::<serde_json::Value>(s).unwrap())
        .unwrap();

    let result = serde_json::to_string_pretty(&line).unwrap();

    insta::assert_snapshot!(&result, @r#"
    {
      "operations": [
        {
          "name": "Simple",
          "document": "query Simple { responseHeader(name: \"\", value: \"\") }",
          "cached": false,
          "status": "Success",
          "type": "query",
          "complexity": null
        }
      ],
      "subgraph_requests": [
        {
          "subgraph_name": "echo",
          "method": "POST",
          "url": "http://127.0.0.1:XXXXX/",
          "executions": [
            {
              "Response": {
                "status": 200,
                "special_header_value": "kekw"
              }
            }
          ],
          "cache_status": "miss",
          "has_errors": false
        }
      ],
      "http_requests": [
        {
          "method": "POST",
          "url": "http://127.0.0.1/graphql",
          "status": 200
        }
      ],
      "custom": [
        {
          "on_request": {
            "value": 1
          },
          "extension_name": "access-logs-18",
          "event_name": "on_request"
        }
      ]
    }
    "#);
}

#[test]
fn access_logs_operation_limits() {
    let tmpdir = TempDir::new().unwrap();
    let path = tmpdir.path().to_str().unwrap();

    let config = formatdoc! {r#"
        [extensions.access-logs-18]
        path = "./data/extensions/crates/access-logs/build"
        stdout = true
        stderr = true

        [extensions.access-logs-18.config]
        path = "{path}/access.log"

        [complexity_control]
        mode = "measure"

        [telemetry.tracing.propagation]
        trace_context = true

        [telemetry]
        service_name = "access-log-test"
    "#};

    let mut response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_toml_config(config)
            .with_extension("access-logs-18")
            .with_subgraph(EchoSchema)
            .build()
            .await;

        engine
            .post(r#"query Simple { responseHeader(name: "X-Special", value: "kekw") }"#)
            .header("traceparent", "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01")
            .await
    });

    insta::assert_json_snapshot!(response.take(), @r#"
    {
      "data": {
        "responseHeader": null
      }
    }
    "#);

    let result = std::fs::read_to_string(tmpdir.path().join("access.log")).unwrap();

    let line = result
        .split('\n')
        .next()
        .map(|s| serde_json::from_str::<serde_json::Value>(s).unwrap())
        .unwrap();

    let result = serde_json::to_string_pretty(&line).unwrap();

    insta::assert_snapshot!(&result, @r#"
    {
      "operations": [
        {
          "name": "Simple",
          "document": "query Simple { responseHeader(name: \"\", value: \"\") }",
          "cached": false,
          "status": "Success",
          "type": "query",
          "complexity": 0
        }
      ],
      "subgraph_requests": [
        {
          "subgraph_name": "echo",
          "method": "POST",
          "url": "http://127.0.0.1:XXXXX/",
          "executions": [
            {
              "Response": {
                "status": 200,
                "special_header_value": "kekw"
              }
            }
          ],
          "cache_status": "miss",
          "has_errors": false
        }
      ],
      "http_requests": [
        {
          "method": "POST",
          "url": "http://127.0.0.1/graphql",
          "status": 200
        }
      ],
      "custom": [
        {
          "on_request": {
            "value": 1
          },
          "extension_name": "access-logs-18",
          "event_name": "on_request"
        }
      ]
    }
    "#);
}

#[test]
fn access_logs_with_broken_query() {
    let tmpdir = TempDir::new().unwrap();
    let path = tmpdir.path().to_str().unwrap();

    let config = formatdoc! {r#"
        [extensions.access-logs-18]
        path = "./data/extensions/crates/access-logs/build"
        stdout = true
        stderr = true

        [extensions.access-logs-18.config]
        path = "{path}/access.log"

        [telemetry.tracing.propagation]
        trace_context = true

        [telemetry]
        service_name = "access-log-test"
    "#};

    let mut response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_toml_config(config)
            .with_extension("access-logs-18")
            .with_subgraph(EchoSchema)
            .build()
            .await;

        engine
            .post(r#"query Simple { "#)
            .header("traceparent", "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01")
            .header("accept", "application/graphql-response+json")
            .await
    });

    insta::assert_json_snapshot!(response.take(), @r#"
    {
      "errors": [
        {
          "message": "unexpected end of file (expected one of , \"...\"RawIdent, schema, query, mutation, subscription, ty, input, true, false, null, implements, interface, \"enum\", union, scalar, extend, directive, repeatable, on, fragment)",
          "locations": [
            {
              "line": 1,
              "column": 15
            }
          ],
          "extensions": {
            "code": "OPERATION_PARSING_ERROR"
          }
        }
      ]
    }
    "#);

    let result = std::fs::read_to_string(tmpdir.path().join("access.log")).unwrap();

    let line = result
        .split('\n')
        .next()
        .map(|s| serde_json::from_str::<serde_json::Value>(s).unwrap())
        .unwrap();

    let result = serde_json::to_string_pretty(&line).unwrap();

    insta::assert_snapshot!(&result, @r#"
    {
      "operations": [],
      "subgraph_requests": [],
      "http_requests": [
        {
          "method": "POST",
          "url": "http://127.0.0.1/graphql",
          "status": 400
        }
      ],
      "custom": [
        {
          "on_request": {
            "value": 1
          },
          "extension_name": "access-logs-18",
          "event_name": "on_request"
        }
      ]
    }
    "#);
}

#[test]
fn access_logs_with_caching() {
    let tmpdir = TempDir::new().unwrap();
    let path = tmpdir.path().to_str().unwrap();

    let config = formatdoc! {r#"
        [extensions.access-logs-18]
        path = "./data/extensions/crates/access-logs/build"
        stdout = true
        stderr = true

        [extensions.access-logs-18.config]
        path = "{path}/access.log"

        [entity_caching]
        enabled = true
        ttl = "1h"

        [telemetry.tracing.propagation]
        trace_context = true

        [telemetry]
        service_name = "access-log-test"
    "#};

    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_toml_config(config)
            .with_extension("access-logs-18")
            .with_subgraph(EchoSchema)
            .build()
            .await;

        let mut response = engine
            .post(r#"query Simple { responseHeader(name: "X-Special", value: "kekw") }"#)
            .header("traceparent", "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01")
            .await;

        insta::assert_json_snapshot!(response.take(), @r#"
        {
          "data": {
            "responseHeader": null
          }
        }
        "#);

        let mut response = engine
            .post(r#"query Simple { responseHeader(name: "X-Special", value: "kekw") }"#)
            .header("traceparent", "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01")
            .await;

        insta::assert_json_snapshot!(response.take(), @r#"
        {
          "data": {
            "responseHeader": null
          }
        }
        "#);

        let result: Vec<_> = std::fs::read_to_string(tmpdir.path().join("access.log"))
            .unwrap()
            .lines()
            .filter(|s| !s.is_empty())
            .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
            .map(|line| serde_json::to_string_pretty(&line).unwrap())
            .collect();

        insta::assert_snapshot!(&result[0], @r#"
        {
          "operations": [
            {
              "name": "Simple",
              "document": "query Simple { responseHeader(name: \"\", value: \"\") }",
              "cached": false,
              "status": "Success",
              "type": "query",
              "complexity": null
            }
          ],
          "subgraph_requests": [
            {
              "subgraph_name": "echo",
              "method": "POST",
              "url": "http://127.0.0.1:XXXXX/",
              "executions": [
                {
                  "Response": {
                    "status": 200,
                    "special_header_value": "kekw"
                  }
                }
              ],
              "cache_status": "miss",
              "has_errors": false
            }
          ],
          "http_requests": [
            {
              "method": "POST",
              "url": "http://127.0.0.1/graphql",
              "status": 200
            }
          ],
          "custom": [
            {
              "on_request": {
                "value": 1
              },
              "extension_name": "access-logs-18",
              "event_name": "on_request"
            }
          ]
        }
        "#);

        insta::assert_snapshot!(&result[1], @r#"
        {
          "operations": [
            {
              "name": "Simple",
              "document": "query Simple { responseHeader(name: \"\", value: \"\") }",
              "cached": true,
              "status": "Success",
              "type": "query",
              "complexity": null
            }
          ],
          "subgraph_requests": [],
          "http_requests": [
            {
              "method": "POST",
              "url": "http://127.0.0.1/graphql",
              "status": 200
            }
          ],
          "custom": [
            {
              "on_request": {
                "value": 1
              },
              "extension_name": "access-logs-18",
              "event_name": "on_request"
            }
          ]
        }
        "#);
    });
}
