use base64::Engine;
use graphql_mocks::EchoSchema;
use integration_tests::{gateway::Gateway, runtime};

#[test]
fn receive_events() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_toml_config(
                r#"
                [extensions.hooks]
                path = "./data/extensions/crates/hooks/build"
                stdout = true
                stderr = true

                [[headers]]
                rule = "forward"
                name = "x-incoming-header"

                [extensions.hooks.config]
                events_header_name = "x-events"
            "#,
            )
            .with_hook_extension("hooks")
            .await
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
            "[].total_duration_ms" => "[redacted]"
        },
        @r#"
    [
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
      }
    ]
    "#
    );
}
