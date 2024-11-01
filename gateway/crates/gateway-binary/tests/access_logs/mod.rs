use std::{collections::BTreeMap, sync::Arc, time::Duration};

use crate::{load_schema, runtime, with_static_server, Client};
use futures_util::Future;
use handlebars::Handlebars;
use indoc::formatdoc;
use serde_json::Value;
use tempfile::TempDir;
use wiremock::{matchers::method, Mock, ResponseTemplate};

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct ResponseInfo {
    status_code: u16,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
enum ResponseKind {
    SerializationError,
    HookError,
    RequestError,
    RateLimited,
    Responsed(ResponseInfo),
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct Subgraph {
    subgraph_name: String,
    method: String,
    responses: Vec<ResponseKind>,
    has_errors: bool,
    cached: bool,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct Operation {
    name: String,
    document: String,
    cached: bool,
    status: Value,
    subgraphs: Vec<Subgraph>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct Log {
    method: String,
    url: String,
    trace_id: String,
    status_code: u16,
    operations: Vec<Operation>,
}

#[test]
fn with_working_subgraph() {
    let tmpdir = TempDir::new().unwrap();
    let path = tmpdir.path().to_str().unwrap();

    let config = indoc::formatdoc! {r#"
        [gateway.access_logs]
        enabled = true
        path = "{path}"
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
    });

    let result = std::fs::read_to_string(tmpdir.path().join("access.log")).unwrap();
    let result = serde_json::from_str::<Log>(&result).unwrap();

    let result = serde_json::to_string_pretty(&result).unwrap();

    insta::assert_snapshot!(&result, @r###"
    {
      "method": "POST",
      "url": "/graphql",
      "trace_id": "0af7651916cd43dd8448eb211c80319c",
      "status_code": 200,
      "operations": [
        {
          "name": "Simple",
          "document": "query Simple {\n  me {\n    id\n  }\n}\n",
          "cached": false,
          "status": "Success",
          "subgraphs": [
            {
              "subgraph_name": "accounts",
              "method": "POST",
              "responses": [
                {
                  "Responsed": {
                    "status_code": 200
                  }
                }
              ],
              "has_errors": false,
              "cached": false
            }
          ]
        }
      ]
    }
    "###);
}

#[test]
fn with_broken_query() {
    let tmpdir = TempDir::new().unwrap();
    let path = tmpdir.path().to_str().unwrap();

    let config = indoc::formatdoc! {r#"
        [gateway.access_logs]
        enabled = true
        path = "{path}"
    "#};

    with_gateway(&config, None, |gateway| async move {
        let resp = gateway
            .gql::<serde_json::Value>("query Simple { ")
            .header("traceparent", "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01")
            .header("accept", "application/graphql-response+json")
            .send()
            .await;

        insta::assert_json_snapshot!(resp, @r###"
        {
          "errors": [
            {
              "message": " --> 1:16\n  |\n1 | query Simple { \n  |                ^---\n  |\n  = expected selection",
              "locations": [
                {
                  "line": 1,
                  "column": 16
                }
              ],
              "extensions": {
                "code": "OPERATION_PARSING_ERROR"
              }
            }
          ]
        }
        "###);
    });

    let result = std::fs::read_to_string(tmpdir.path().join("access.log")).unwrap();
    let result = serde_json::from_str::<Log>(&result).unwrap();

    let result = serde_json::to_string_pretty(&result).unwrap();

    insta::assert_snapshot!(&result, @r###"
    {
      "method": "POST",
      "url": "/graphql",
      "trace_id": "0af7651916cd43dd8448eb211c80319c",
      "status_code": 400,
      "operations": []
    }
    "###);
}

#[test]
fn with_working_subgraph_rate_limited() {
    let tmpdir = TempDir::new().unwrap();
    let path = tmpdir.path().to_str().unwrap();

    let config = indoc::formatdoc! {r#"
        [gateway.access_logs]
        enabled = true
        path = "{path}"

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

        insta::assert_json_snapshot!(resp, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Too many requests",
              "path": [
                "me"
              ],
              "extensions": {
                "code": "RATE_LIMITED"
              }
            }
          ]
        }
        "###);
    });

    let result: Vec<_> = std::fs::read_to_string(tmpdir.path().join("access.log"))
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<Log>(line).unwrap())
        .map(|line| serde_json::to_string_pretty(&line).unwrap())
        .collect();

    insta::assert_snapshot!(&result[0], @r###"
    {
      "method": "POST",
      "url": "/graphql",
      "trace_id": "0af7651916cd43dd8448eb211c80319c",
      "status_code": 200,
      "operations": [
        {
          "name": "Simple",
          "document": "query Simple {\n  me {\n    id\n  }\n}\n",
          "cached": false,
          "status": "Success",
          "subgraphs": [
            {
              "subgraph_name": "accounts",
              "method": "POST",
              "responses": [
                {
                  "Responsed": {
                    "status_code": 200
                  }
                }
              ],
              "has_errors": false,
              "cached": false
            }
          ]
        }
      ]
    }
    "###);

    insta::assert_snapshot!(&result[1], @r###"
    {
      "method": "POST",
      "url": "/graphql",
      "trace_id": "0af7651916cd43dd8448eb211c80319c",
      "status_code": 200,
      "operations": [
        {
          "name": "Simple",
          "document": "query Simple {\n  me {\n    id\n  }\n}\n",
          "cached": true,
          "status": {
            "FieldError": {
              "count": 1,
              "data_is_null": true
            }
          },
          "subgraphs": [
            {
              "subgraph_name": "accounts",
              "method": "POST",
              "responses": [
                "RateLimited"
              ],
              "has_errors": false,
              "cached": false
            }
          ]
        }
      ]
    }
    "###);
}

#[test]
fn with_broken_subgraph() {
    let tmpdir = TempDir::new().unwrap();
    let path = tmpdir.path().to_str().unwrap();

    let config = indoc::formatdoc! {r#"
        [gateway.access_logs]
        enabled = true
        path = "{path}"
    "#};

    with_gateway(&config, None, |gateway| async move {
        let resp = gateway
            .gql::<serde_json::Value>("query Simple { me { id } }")
            .header("traceparent", "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01")
            .send()
            .await;

        insta::assert_json_snapshot!(resp, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Request to subgraph 'accounts' failed with: error sending request",
              "path": [
                "me"
              ],
              "extensions": {
                "code": "SUBGRAPH_REQUEST_ERROR"
              }
            }
          ]
        }
        "###);
    });

    let result = std::fs::read_to_string(tmpdir.path().join("access.log")).unwrap();
    let result = serde_json::from_str::<Log>(&result).unwrap();

    let result = serde_json::to_string_pretty(&result).unwrap();

    insta::assert_snapshot!(&result, @r###"
    {
      "method": "POST",
      "url": "/graphql",
      "trace_id": "0af7651916cd43dd8448eb211c80319c",
      "status_code": 200,
      "operations": [
        {
          "name": "Simple",
          "document": "query Simple {\n  me {\n    id\n  }\n}\n",
          "cached": false,
          "status": {
            "FieldError": {
              "count": 1,
              "data_is_null": true
            }
          },
          "subgraphs": [
            {
              "subgraph_name": "accounts",
              "method": "POST",
              "responses": [
                {
                  "Responsed": {
                    "status_code": 0
                  }
                }
              ],
              "has_errors": false,
              "cached": false
            }
          ]
        }
      ]
    }
    "###);
}

#[test]
fn with_broken_subgraph_retried() {
    let tmpdir = TempDir::new().unwrap();
    let path = tmpdir.path().to_str().unwrap();

    let config = indoc::formatdoc! {r#"
        [gateway.access_logs]
        enabled = true
        path = "{path}"

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

        insta::assert_json_snapshot!(resp, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Request to subgraph 'accounts' failed with: error sending request",
              "path": [
                "me"
              ],
              "extensions": {
                "code": "SUBGRAPH_REQUEST_ERROR"
              }
            }
          ]
        }
        "###);
    });

    let result = std::fs::read_to_string(tmpdir.path().join("access.log")).unwrap();
    let result = serde_json::from_str::<Log>(&result).unwrap();

    let result = serde_json::to_string_pretty(&result).unwrap();

    insta::assert_snapshot!(&result, @r###"
    {
      "method": "POST",
      "url": "/graphql",
      "trace_id": "0af7651916cd43dd8448eb211c80319c",
      "status_code": 200,
      "operations": [
        {
          "name": "Simple",
          "document": "query Simple {\n  me {\n    id\n  }\n}\n",
          "cached": false,
          "status": {
            "FieldError": {
              "count": 1,
              "data_is_null": true
            }
          },
          "subgraphs": [
            {
              "subgraph_name": "accounts",
              "method": "POST",
              "responses": [
                {
                  "Responsed": {
                    "status_code": 0
                  }
                },
                {
                  "Responsed": {
                    "status_code": 0
                  }
                }
              ],
              "has_errors": false,
              "cached": false
            }
          ]
        }
      ]
    }
    "###);
}

#[test]
fn with_caching() {
    let tmpdir = TempDir::new().unwrap();
    let path = tmpdir.path().to_str().unwrap();

    let config = indoc::formatdoc! {r#"
        [gateway.access_logs]
        enabled = true
        path = "{path}"

        [entity_caching]
        enabled = true
        ttl = "60s"
    "#};

    with_gateway(&config, Some(200), |gateway| async move {
        gateway
            .gql::<serde_json::Value>("query Simple { me { id } }")
            .header("traceparent", "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01")
            .send()
            .await;

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
    });

    let result: Vec<_> = std::fs::read_to_string(tmpdir.path().join("access.log"))
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<Log>(line).unwrap())
        .map(|line| serde_json::to_string_pretty(&line).unwrap())
        .collect();

    insta::assert_snapshot!(&result[0], @r###"
    {
      "method": "POST",
      "url": "/graphql",
      "trace_id": "0af7651916cd43dd8448eb211c80319c",
      "status_code": 200,
      "operations": [
        {
          "name": "Simple",
          "document": "query Simple {\n  me {\n    id\n  }\n}\n",
          "cached": false,
          "status": "Success",
          "subgraphs": [
            {
              "subgraph_name": "accounts",
              "method": "POST",
              "responses": [
                {
                  "Responsed": {
                    "status_code": 200
                  }
                }
              ],
              "has_errors": false,
              "cached": false
            }
          ]
        }
      ]
    }
    "###);

    insta::assert_snapshot!(&result[1], @r###"
    {
      "method": "POST",
      "url": "/graphql",
      "trace_id": "0af7651916cd43dd8448eb211c80319c",
      "status_code": 200,
      "operations": [
        {
          "name": "Simple",
          "document": "query Simple {\n  me {\n    id\n  }\n}\n",
          "cached": true,
          "status": "Success",
          "subgraphs": [
            {
              "subgraph_name": "accounts",
              "method": "POST",
              "responses": [],
              "has_errors": false,
              "cached": true
            }
          ]
        }
      ]
    }
    "###);
}

#[test]
fn with_subgraph_status_500() {
    let tmpdir = TempDir::new().unwrap();
    let path = tmpdir.path().to_str().unwrap();

    let config = indoc::formatdoc! {r#"
        [gateway.access_logs]
        enabled = true
        path = "{path}"
    "#};

    with_gateway(&config, Some(500), |gateway| async move {
        let resp = gateway
            .gql::<serde_json::Value>("query Simple { me { id } }")
            .header("traceparent", "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01")
            .send()
            .await;

        insta::assert_json_snapshot!(resp, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Request to subgraph 'accounts' failed with: Invalid status code: 500",
              "path": [
                "me"
              ],
              "extensions": {
                "code": "SUBGRAPH_REQUEST_ERROR"
              }
            }
          ]
        }
        "###);
    });

    let result = std::fs::read_to_string(tmpdir.path().join("access.log")).unwrap();
    let result = serde_json::from_str::<Log>(&result).unwrap();

    let result = serde_json::to_string_pretty(&result).unwrap();

    insta::assert_snapshot!(&result, @r###"
    {
      "method": "POST",
      "url": "/graphql",
      "trace_id": "0af7651916cd43dd8448eb211c80319c",
      "status_code": 200,
      "operations": [
        {
          "name": "Simple",
          "document": "query Simple {\n  me {\n    id\n  }\n}\n",
          "cached": false,
          "status": {
            "FieldError": {
              "count": 1,
              "data_is_null": true
            }
          },
          "subgraphs": [
            {
              "subgraph_name": "accounts",
              "method": "POST",
              "responses": [
                {
                  "Responsed": {
                    "status_code": 500
                  }
                }
              ],
              "has_errors": false,
              "cached": false
            }
          ]
        }
      ]
    }
    "###);
}

#[test]
fn with_subgraph_status_500_retried() {
    let tmpdir = TempDir::new().unwrap();
    let path = tmpdir.path().to_str().unwrap();

    let config = indoc::formatdoc! {r#"
        [gateway.access_logs]
        enabled = true
        path = "{path}"

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

        insta::assert_json_snapshot!(resp, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Request to subgraph 'accounts' failed with: Invalid status code: 500",
              "path": [
                "me"
              ],
              "extensions": {
                "code": "SUBGRAPH_REQUEST_ERROR"
              }
            }
          ]
        }
        "###);
    });

    let result = std::fs::read_to_string(tmpdir.path().join("access.log")).unwrap();
    let result = serde_json::from_str::<Log>(&result).unwrap();

    let result = serde_json::to_string_pretty(&result).unwrap();

    insta::assert_snapshot!(&result, @r###"
    {
      "method": "POST",
      "url": "/graphql",
      "trace_id": "0af7651916cd43dd8448eb211c80319c",
      "status_code": 200,
      "operations": [
        {
          "name": "Simple",
          "document": "query Simple {\n  me {\n    id\n  }\n}\n",
          "cached": false,
          "status": {
            "FieldError": {
              "count": 1,
              "data_is_null": true
            }
          },
          "subgraphs": [
            {
              "subgraph_name": "accounts",
              "method": "POST",
              "responses": [
                {
                  "Responsed": {
                    "status_code": 500
                  }
                },
                {
                  "Responsed": {
                    "status_code": 500
                  }
                }
              ],
              "has_errors": false,
              "cached": false
            }
          ]
        }
      ]
    }
    "###);
}

#[test]
fn with_failing_on_gateway_request_hook() {
    let tmpdir = TempDir::new().unwrap();
    let path = tmpdir.path().to_str().unwrap();

    let config = indoc::formatdoc! {r#"
        [gateway.access_logs]
        enabled = true
        path = "{path}"
    "#};

    with_gateway(&config, Some(200), |gateway| async move {
        let resp = gateway
            .gql::<serde_json::Value>("query Simple { me { id } }")
            .header("test-value", "some-value")
            .header("traceparent", "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01")
            .send()
            .await;

        insta::assert_json_snapshot!(resp, @r#"
        {
          "errors": [
            {
              "message": "test-value header is not allowed",
              "extensions": {
                "code": "BAD_REQUEST"
              }
            }
          ]
        }
        "#);
    });

    let result = std::fs::read_to_string(tmpdir.path().join("access.log")).unwrap();
    let result = serde_json::from_str::<Log>(&result).unwrap();

    let result = serde_json::to_string_pretty(&result).unwrap();

    insta::assert_snapshot!(&result, @r#"
    {
      "method": "POST",
      "url": "/graphql",
      "trace_id": "0af7651916cd43dd8448eb211c80319c",
      "status_code": 500,
      "operations": []
    }
    "#);
}

fn with_gateway<T, F>(config: &str, subgraph_status: Option<u16>, test: T)
where
    T: FnOnce(Arc<Client>) -> F,
    F: Future<Output = ()>,
{
    let config = &formatdoc! {r#"
        [graph]
        introspection = true

        [hooks]
        location = "../../../engine/crates/wasi-component-loader/examples/target/wasm32-wasip1/debug/response_hooks.wasm"

        [telemetry.tracing.propagation]
        trace_context = true

        [telemetry]
        service_name = "access-log-test"

        [telemetry.tracing]
        sampling = 1

        [telemetry.exporters.otlp]
        enabled = true
        endpoint = "http://localhost:4318"
        protocol = "grpc"

        [telemetry.exporters.otlp.batch_export]
        scheduled_delay = 1
        max_export_batch_size = 1

        {config}
    "#};

    let server = runtime().block_on(async move {
        let server = wiremock::MockServer::start().await;

        match subgraph_status {
            Some(code) if code == 200 => {
                let response = ResponseTemplate::new(code).set_body_json(serde_json::json!({
                    "data": {
                        "me": {
                            "id": "1",
                            "username": "Alice",
                        }
                    }
                }));

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

    println!("{config}");
    println!("{schema}");

    with_static_server(config, &schema, None, None, |client| async move {
        const WAIT_SECONDS: u64 = 2;

        // wait for initial polling to be pushed to OTEL tables so we can ignore it with the
        // appropriate start time filter.
        tokio::time::sleep(Duration::from_secs(WAIT_SECONDS)).await;

        test(client).await
    })
}
