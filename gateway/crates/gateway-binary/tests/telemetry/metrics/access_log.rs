use tempfile::TempDir;

use crate::telemetry::metrics::{SumRow, METRICS_DELAY};

use super::with_custom_gateway;

#[test]
fn measures_pending_logs() {
    let tmpdir = TempDir::new().unwrap();
    let path = tmpdir.path().to_str().unwrap();

    let config = indoc::formatdoc! {r#"
        [gateway.access_logs]
        enabled = true
        path = "{path}"

        [hooks]
        location = "../../../engine/crates/wasi-component-loader/examples/target/wasm32-wasip1/debug/response_hooks.wasm"
    "#};

    with_custom_gateway(&config, |service_name, _, gateway, clickhouse| async move {
        let response = gateway
            .gql::<serde_json::Value>("query Simple { __typename }")
            .send()
            .await;

        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "__typename": "Query"
          }
        }"###);

        tokio::time::sleep(METRICS_DELAY).await;

        let row = clickhouse
            .query(
                r#"
                SELECT Value, Attributes
                FROM otel_metrics_sum
                WHERE ServiceName = ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'grafbase.gateway.access_log.pending'
                "#,
            )
            .bind(&service_name)
            .fetch_one::<SumRow>()
            .await
            .unwrap();

        insta::assert_json_snapshot!(row, @r###"
        {
          "Value": 0.0,
          "Attributes": {}
        }
        "###);
    });
}

#[test]
fn measures_pool_size() {
    let tmpdir = TempDir::new().unwrap();
    let path = tmpdir.path().to_str().unwrap();

    let config = indoc::formatdoc! {r#"
        [gateway.access_logs]
        enabled = true
        path = "{path}"

        [hooks]
        location = "../../../engine/crates/wasi-component-loader/examples/target/wasm32-wasip1/debug/response_hooks.wasm"
    "#};

    with_custom_gateway(&config, |service_name, _, gateway, clickhouse| async move {
        let response = gateway
            .gql::<serde_json::Value>("query Simple { __typename }")
            .send()
            .await;

        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "__typename": "Query"
          }
        }"###);

        tokio::time::sleep(METRICS_DELAY).await;

        let row = clickhouse
            .query(
                r#"
                SELECT Value, Attributes
                FROM otel_metrics_sum
                WHERE ServiceName = ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'grafbase.hook.pool.instances.busy'
                "#,
            )
            .bind(&service_name)
            .fetch_one::<SumRow>()
            .await
            .unwrap();

        insta::assert_json_snapshot!(row, @r#"
        {
          "Value": 0.0,
          "Attributes": {
            "grafbase.hook.interface": "component:grafbase/responses"
          }
        }
        "#);
    });
}
