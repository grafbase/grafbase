use std::{collections::BTreeMap, sync::Arc, time::Duration};

use crate::{load_schema, runtime, with_static_server, Client};
use futures_util::Future;
use handlebars::Handlebars;
use indoc::formatdoc;
use tempfile::TempDir;
use wiremock::{matchers::method, Mock, ResponseTemplate};

#[test]
fn smoke() {
    let tmpdir = TempDir::new().unwrap();
    let path = tmpdir.path().to_str().unwrap();

    let config = indoc::formatdoc! {r#"
        [hooks]
        location = "../../../engine/crates/wasi-component-loader/examples/target/wasm32-wasip1/debug/response_hooks.wasm"

        [gateway.access_logs]
        enabled = true
        path = "{path}"

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
    "#};

    with_gateway(&config, |gateway| async move {
        let resp = gateway
            .gql::<serde_json::Value>("query Simple { me { id } }")
            .send()
            .await;

        tokio::time::sleep(Duration::from_secs(2)).await;

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

    #[derive(serde::Deserialize, serde::Serialize)]
    struct Subgraph {
        subgraph_name: String,
        method: String,
        status_codes: Vec<u16>,
        has_errors: bool,
        cached: bool,
    }

    #[derive(serde::Deserialize, serde::Serialize)]
    struct Operation {
        name: String,
        document: String,
        cached: bool,
        status: String,
        subgraphs: Vec<Subgraph>,
    }

    #[derive(serde::Deserialize, serde::Serialize)]
    struct Log {
        method: String,
        url: String,
        trace_id: String,
        operations: Vec<Operation>,
    }

    let result = std::fs::read_to_string(tmpdir.path().join("access.log")).unwrap();
    let mut result = serde_json::from_str::<Log>(&result).unwrap();

    assert_ne!("00000000000000000000000000000000", &result.trace_id);
    result.trace_id = String::from("VARIES");

    let result = serde_json::to_string_pretty(&result).unwrap();

    insta::assert_snapshot!(&result, @r###"
    {
      "method": "POST",
      "url": "/graphql",
      "trace_id": "VARIES",
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
              "status_codes": [
                200
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

fn with_gateway<T, F>(config: &str, test: T)
where
    T: FnOnce(Arc<Client>) -> F,
    F: Future<Output = ()>,
{
    let config = &formatdoc! {r#"
        [graph]
        introspection = true

        {config}
    "#};

    let server = runtime().block_on(async move {
        let server = wiremock::MockServer::start().await;

        let response = ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "me": {
                    "id": "1",
                    "username": "Alice",
                }
            }
        }));

        Mock::given(method("POST")).respond_with(response).mount(&server).await;

        server
    });

    let mut hb = Handlebars::new();
    hb.register_template_string("t1", load_schema("small")).unwrap();

    let mut data = BTreeMap::new();
    data.insert("subgraph_endpoint", format!("http://{}", server.address()));

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
