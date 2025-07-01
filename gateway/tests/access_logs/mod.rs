use std::{collections::BTreeMap, sync::Arc, time::Duration};

use crate::{Client, CommandHandles, cargo_bin, listen_address, load_schema, runtime, with_static_server};
use duct::cmd;
use futures_util::Future;
use handlebars::Handlebars;
use indoc::formatdoc;
use tempfile::TempDir;
use wiremock::{Mock, ResponseTemplate, matchers::method};

#[test]
fn with_working_subgraph_rate_limited() {
    let tmpdir = TempDir::new().unwrap();
    let path = tmpdir.path().to_str().unwrap();

    let config = indoc::formatdoc! {r#"
        [extensions.access-logs.config]
        path = "{path}/access.log"

        [gateway.rate_limit]
        storage = "redis"

        [subgraphs.accounts.rate_limit]
        limit = 1
        duration = "1m"
    "#};

    with_gateway(&config, Some(200), |gateway| async move {
        let resp = gateway
            .gql::<serde_json::Value>("query Simple { me { id } }")
            .header("traceparent", "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01")
            .send()
            .await;

        insta::assert_json_snapshot!(resp, @r###"
        {
          "data": {
            "me": {
              "id": "1"
            }
          }
        }
        "###);

        let resp = gateway
            .gql::<serde_json::Value>("query Simple { me { id } }")
            .header("traceparent", "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01")
            .send()
            .await;

        insta::assert_json_snapshot!(resp, @r#"
        {
          "data": null,
          "errors": [
            {
              "message": "Too many requests",
              "locations": [
                {
                  "line": 1,
                  "column": 16
                }
              ],
              "path": [
                "me"
              ],
              "extensions": {
                "code": "RATE_LIMITED"
              }
            }
          ]
        }
        "#);
    });

    let result: Vec<_> = std::fs::read_to_string(tmpdir.path().join("access.log"))
        .unwrap()
        .lines()
        .filter(|s| !s.is_empty())
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .map(|line| serde_json::to_string_pretty(&line).unwrap())
        .collect();

    insta::assert_snapshot!(&result[1], @r#"
    {
      "operations": [
        {
          "name": "Simple",
          "document": "query Simple { me { id } }",
          "cached": false,
          "status": "Success",
          "type": "query",
          "complexity": null
        }
      ],
      "subgraph_requests": [
        {
          "subgraph_name": "accounts",
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
          "url": "/graphql",
          "status": 200
        }
      ],
      "custom": [
        {
          "value": 1
        }
      ]
    }
    "#);

    insta::assert_snapshot!(&result[2], @r#"
    {
      "operations": [
        {
          "name": "Simple",
          "document": "query Simple { me { id } }",
          "cached": true,
          "status": {
            "FieldError": {
              "count": 1,
              "data_is_null": true
            }
          },
          "type": "query",
          "complexity": null
        }
      ],
      "subgraph_requests": [
        {
          "subgraph_name": "accounts",
          "method": "POST",
          "url": "http://127.0.0.1:XXXXX/",
          "executions": [
            "RateLimitExceeded"
          ],
          "cache_status": "miss",
          "has_errors": false
        }
      ],
      "http_requests": [
        {
          "method": "POST",
          "url": "/graphql",
          "status": 200
        }
      ],
      "custom": [
        {
          "value": 1
        }
      ]
    }
    "#);
}

#[test]
fn with_broken_subgraph() {
    let tmpdir = TempDir::new().unwrap();
    let path = tmpdir.path().to_str().unwrap();

    let config = indoc::formatdoc! {r#"
        [extensions.access-logs.config]
        path = "{path}/access.log"
    "#};

    with_gateway(&config, None, |gateway| async move {
        let resp = gateway
            .gql::<serde_json::Value>("query Simple { me { id } }")
            .header("traceparent", "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01")
            .send()
            .await;

        insta::assert_json_snapshot!(resp, @r#"
        {
          "data": null,
          "errors": [
            {
              "message": "Request to subgraph 'accounts' failed.",
              "locations": [
                {
                  "line": 1,
                  "column": 16
                }
              ],
              "path": [
                "me"
              ],
              "extensions": {
                "code": "SUBGRAPH_REQUEST_ERROR"
              }
            }
          ]
        }
        "#);
    });

    let result = std::fs::read_to_string(tmpdir.path().join("access.log")).unwrap();

    let line = result
        .split('\n')
        .nth(1)
        .map(|s| serde_json::from_str::<serde_json::Value>(s).unwrap())
        .unwrap();

    let result = serde_json::to_string_pretty(&line).unwrap();

    insta::assert_snapshot!(&result, @r#"
    {
      "operations": [
        {
          "name": "Simple",
          "document": "query Simple { me { id } }",
          "cached": false,
          "status": {
            "FieldError": {
              "count": 1,
              "data_is_null": true
            }
          },
          "type": "query",
          "complexity": null
        }
      ],
      "subgraph_requests": [
        {
          "subgraph_name": "accounts",
          "method": "POST",
          "url": "http://localhost:1234/1:XXXXX/",
          "executions": [
            "RequestError"
          ],
          "cache_status": "miss",
          "has_errors": false
        }
      ],
      "http_requests": [
        {
          "method": "POST",
          "url": "/graphql",
          "status": 200
        }
      ],
      "custom": [
        {
          "value": 1
        }
      ]
    }
    "#);
}

#[test]
fn with_broken_subgraph_retried() {
    let tmpdir = TempDir::new().unwrap();
    let path = tmpdir.path().to_str().unwrap();

    let config = indoc::formatdoc! {r#"
        [extensions.access-logs.config]
        path = "{path}/access.log"

        [gateway.retry]
        enabled = true
        min_per_second = 1
        ttl = "1s"
        retry_percent = 0.1
        retry_mutations = false
    "#};

    with_gateway(&config, None, |gateway| async move {
        let resp = gateway
            .gql::<serde_json::Value>("query Simple { me { id } }")
            .header("traceparent", "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01")
            .send()
            .await;

        insta::assert_json_snapshot!(resp, @r#"
        {
          "data": null,
          "errors": [
            {
              "message": "Request to subgraph 'accounts' failed.",
              "locations": [
                {
                  "line": 1,
                  "column": 16
                }
              ],
              "path": [
                "me"
              ],
              "extensions": {
                "code": "SUBGRAPH_REQUEST_ERROR"
              }
            }
          ]
        }
        "#);
    });

    let result = std::fs::read_to_string(tmpdir.path().join("access.log")).unwrap();

    let line = result
        .split('\n')
        .nth(1)
        .map(|s| serde_json::from_str::<serde_json::Value>(s).unwrap())
        .unwrap();

    let result = serde_json::to_string_pretty(&line).unwrap();

    insta::assert_snapshot!(&result, @r#"
    {
      "operations": [
        {
          "name": "Simple",
          "document": "query Simple { me { id } }",
          "cached": false,
          "status": {
            "FieldError": {
              "count": 1,
              "data_is_null": true
            }
          },
          "type": "query",
          "complexity": null
        }
      ],
      "subgraph_requests": [
        {
          "subgraph_name": "accounts",
          "method": "POST",
          "url": "http://localhost:1234/1:XXXXX/",
          "executions": [
            "RequestError",
            "RequestError"
          ],
          "cache_status": "miss",
          "has_errors": false
        }
      ],
      "http_requests": [
        {
          "method": "POST",
          "url": "/graphql",
          "status": 200
        }
      ],
      "custom": [
        {
          "value": 1
        }
      ]
    }
    "#);
}

#[test]
fn with_subgraph_status_500() {
    let tmpdir = TempDir::new().unwrap();
    let path = tmpdir.path().to_str().unwrap();

    let config = indoc::formatdoc! {r#"
        [extensions.access-logs.config]
        path = "{path}/access.log"
    "#};

    with_gateway(&config, Some(500), |gateway| async move {
        let resp = gateway
            .gql::<serde_json::Value>("query Simple { me { id } }")
            .header("traceparent", "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01")
            .send()
            .await;

        insta::assert_json_snapshot!(resp, @r#"
        {
          "data": null,
          "errors": [
            {
              "message": "Request to subgraph 'accounts' failed.",
              "locations": [
                {
                  "line": 1,
                  "column": 16
                }
              ],
              "path": [
                "me"
              ],
              "extensions": {
                "code": "SUBGRAPH_REQUEST_ERROR"
              }
            }
          ]
        }
        "#);
    });

    let result = std::fs::read_to_string(tmpdir.path().join("access.log")).unwrap();

    let line = result
        .split('\n')
        .nth(1)
        .map(|s| serde_json::from_str::<serde_json::Value>(s).unwrap())
        .unwrap();

    let result = serde_json::to_string_pretty(&line).unwrap();

    insta::assert_snapshot!(&result, @r#"
    {
      "operations": [
        {
          "name": "Simple",
          "document": "query Simple { me { id } }",
          "cached": false,
          "status": {
            "FieldError": {
              "count": 1,
              "data_is_null": true
            }
          },
          "type": "query",
          "complexity": null
        }
      ],
      "subgraph_requests": [
        {
          "subgraph_name": "accounts",
          "method": "POST",
          "url": "http://127.0.0.1:XXXXX/",
          "executions": [
            {
              "Response": {
                "status": 500,
                "special_header_value": null
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
          "url": "/graphql",
          "status": 200
        }
      ],
      "custom": [
        {
          "value": 1
        }
      ]
    }
    "#);
}

#[test]
fn with_subgraph_status_500_retried() {
    let tmpdir = TempDir::new().unwrap();
    let path = tmpdir.path().to_str().unwrap();

    let config = indoc::formatdoc! {r#"
        [extensions.access-logs.config]
        path = "{path}/access.log"

        [gateway.retry]
        enabled = true
        min_per_second = 1
        ttl = "1s"
        retry_percent = 0.1
        retry_mutations = false
    "#};

    with_gateway(&config, Some(500), |gateway| async move {
        let resp = gateway
            .gql::<serde_json::Value>("query Simple { me { id } }")
            .header("traceparent", "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01")
            .send()
            .await;

        insta::assert_json_snapshot!(resp, @r#"
        {
          "data": null,
          "errors": [
            {
              "message": "Request to subgraph 'accounts' failed.",
              "locations": [
                {
                  "line": 1,
                  "column": 16
                }
              ],
              "path": [
                "me"
              ],
              "extensions": {
                "code": "SUBGRAPH_REQUEST_ERROR"
              }
            }
          ]
        }
        "#);
    });

    let result = std::fs::read_to_string(tmpdir.path().join("access.log")).unwrap();

    let line = result
        .split('\n')
        .nth(1)
        .map(|s| serde_json::from_str::<serde_json::Value>(s).unwrap())
        .unwrap();

    let result = serde_json::to_string_pretty(&line).unwrap();

    insta::assert_snapshot!(&result, @r#"
    {
      "operations": [
        {
          "name": "Simple",
          "document": "query Simple { me { id } }",
          "cached": false,
          "status": {
            "FieldError": {
              "count": 1,
              "data_is_null": true
            }
          },
          "type": "query",
          "complexity": null
        }
      ],
      "subgraph_requests": [
        {
          "subgraph_name": "accounts",
          "method": "POST",
          "url": "http://127.0.0.1:XXXXX/",
          "executions": [
            {
              "Response": {
                "status": 500,
                "special_header_value": null
              }
            },
            {
              "Response": {
                "status": 500,
                "special_header_value": null
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
          "url": "/graphql",
          "status": 200
        }
      ],
      "custom": [
        {
          "value": 1
        }
      ]
    }
    "#);
}

#[test]
fn with_stdout_capture() {
    use std::fs;

    let tmpdir = TempDir::new().unwrap();
    let path = tmpdir.path().to_str().unwrap();

    let wasi_module_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../crates/integration-tests/data/extensions/crates/access-logs/build"
    );

    let config = formatdoc! {r#"
        [graph]
        introspection = true

        [extensions.access-logs]
        path = "{wasi_module_path}"
        stdout = true
        stderr = true

        [extensions.access-logs.config]
        path = "{path}/access.log"
    "#};

    runtime().block_on(async move {
        const WAIT_SECONDS: u64 = 2;

        let server = wiremock::MockServer::start().await;

        let response = ResponseTemplate::new(200)
            .set_body_json(serde_json::json!({
                "data": {
                    "me": {
                        "id": "1",
                        "username": "Alice",
                    }
                }
            }))
            .insert_header("X-Special", "test-header");

        Mock::given(method("POST")).respond_with(response).mount(&server).await;

        let mut hb = Handlebars::new();
        hb.register_template_string("t1", load_schema("small")).unwrap();

        let mut data = BTreeMap::new();
        data.insert("subgraph_endpoint", format!("http://{}", server.address()));

        let schema = hb.render("t1", &data).unwrap();

        let temp_dir = tempfile::tempdir().unwrap();
        let schema_path = temp_dir.path().join("schema.graphql");
        fs::write(&schema_path, &schema).unwrap();

        let config_path = temp_dir.path().join("grafbase.toml");
        fs::write(&config_path, &config).unwrap();

        let addr = listen_address();
        let args = vec![
            "--listen-address".to_string(),
            addr.to_string(),
            "--schema".to_string(),
            schema_path.to_str().unwrap().to_string(),
            "--config".to_string(),
            config_path.to_str().unwrap().to_string(),
            "--log-style".to_string(),
            "json".to_string(),
        ];

        let command = cmd(cargo_bin("grafbase-gateway"), &args)
            .unchecked()
            .stdout_capture()
            .stderr_capture()
            .start()
            .unwrap();

        // Wait for gateway to start
        tokio::time::sleep(Duration::from_secs(WAIT_SECONDS)).await;

        let client = Arc::new(Client::new(
            format!("http://{addr}/graphql"),
            CommandHandles::new(),
            Some(schema_path),
        ));

        client.poll_endpoint(30, 300).await;

        // Make a request
        let resp = client
            .gql::<serde_json::Value>("query Simple { me { id } }")
            .send()
            .await;

        assert_eq!(
            resp,
            serde_json::json!({
                "data": {
                    "me": {
                        "id": "1"
                    }
                }
            })
        );

        // Give time for logs to flush
        tokio::time::sleep(Duration::from_secs(1)).await;
        command.kill().unwrap();

        let output = command.into_output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);

        #[derive(serde::Deserialize, serde::Serialize)]
        struct LogEntry {
            fields: serde_json::Value
        }

        // Filter out non-deterministic parts like timestamps and ports
        let stdout_filtered: Vec<String> = stdout
            .lines()
            .filter(|line| line.contains("on-response-hook"))
            .map(|line| serde_json::from_str::<LogEntry>(line).unwrap())
            .map(|line| serde_json::to_string(&line).unwrap())
            .collect();

        insta::assert_snapshot!(stdout_filtered.join("\n"), @r#"
        {"fields":{"extension":"access-logs","message":"on-response-hook","guest_fields":{"operations":"0","subgraph_requests":"0","http_requests":"1","custom_events":"1","optional_field":"foo","random_string":"random_string_value"}}}
        {"fields":{"extension":"access-logs","message":"on-response-hook","guest_fields":{"operations":"1","subgraph_requests":"1","http_requests":"1","custom_events":"1","optional_field":"foo","random_string":"random_string_value"}}}
        "#);
    });
}

fn with_gateway<T, F>(config: &str, subgraph_status: Option<u16>, test: T)
where
    T: FnOnce(Arc<Client>) -> F,
    F: Future<Output = ()>,
{
    let wasi_module_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../crates/integration-tests/data/extensions/crates/access-logs/build"
    );

    let config = &formatdoc! {r#"
        [graph]
        introspection = true

        [extensions.access-logs]
        path = "{wasi_module_path}"

        [telemetry.tracing.propagation]
        trace_context = true

        [telemetry]
        service_name = "access-log-test"

        [telemetry.tracing]
        sampling = 1

        [telemetry.exporters.otlp]
        enabled = true
        endpoint = "http://localhost:4327"
        protocol = "grpc"

        [telemetry.exporters.otlp.batch_export]
        scheduled_delay = "1s"
        max_export_batch_size = 1

        {config}
    "#};

    let server = runtime().block_on(async move {
        let server = wiremock::MockServer::start().await;

        match subgraph_status {
            Some(code) if code == 200 => {
                let response = ResponseTemplate::new(code)
                    .set_body_json(serde_json::json!({
                        "data": {
                            "me": {
                                "id": "1",
                                "username": "Alice",
                            }
                        }
                    }))
                    .insert_header("X-Special", "kekw");

                Mock::given(method("POST")).respond_with(response).mount(&server).await;
            }
            Some(code) => {
                let response = ResponseTemplate::new(code).set_body_json(serde_json::json!({
                    "errors": [
                      {
                        "message": "FAILED",
                      }
                    ]
                }));

                Mock::given(method("POST")).respond_with(response).mount(&server).await;
            }
            None => (),
        }

        server
    });

    let mut hb = Handlebars::new();
    hb.register_template_string("t1", load_schema("small")).unwrap();

    let mut data = BTreeMap::new();

    if subgraph_status.is_some() {
        data.insert("subgraph_endpoint", format!("http://{}", server.address()));
    } else {
        data.insert("subgraph_endpoint", "http://localhost:1234".to_string());
    }

    let schema = hb.render("t1", &data).unwrap();

    with_static_server(config, &schema, None, None, |client| async move {
        const WAIT_SECONDS: u64 = 2;

        // wait for initial polling to be pushed to OTEL tables so we can ignore it with the
        // appropriate start time filter.
        tokio::time::sleep(Duration::from_secs(WAIT_SECONDS)).await;

        test(client).await
    })
}
