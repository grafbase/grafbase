use base64::Engine;
use graphql_mocks::EchoSchema;
use integration_tests::{gateway::Gateway, runtime};

#[test]
fn receive_events() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_toml_config(
                r#"
                [extensions.hooks-19]
                path = "./data/extensions/crates/hooks/build"
                stdout = true
                stderr = true

                [[headers]]
                rule = "forward"
                name = "x-incoming-header"

                [extensions.hooks-19.config]
                events_header_name = "x-events"
            "#,
            )
            .with_extension("hooks-19")
            .with_subgraph(EchoSchema::default())
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
