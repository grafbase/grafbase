use crate::telemetry::metrics::{with_custom_gateway, ExponentialHistogramRow, METRICS_DELAY};

#[test]
fn on_gateway_request_success() {
    let config = indoc::indoc! {r#"
        [hooks]
        location = "../../../engine/crates/wasi-component-loader/examples/target/wasm32-wasip1/debug/gateway_request_no_op.wasm"
    "#};

    with_custom_gateway(config, |service_name, _, gateway, clickhouse| async move {
        let resp = gateway
            .gql::<serde_json::Value>("query SimpleQuery { __typename }")
            .send()
            .await;

        insta::assert_json_snapshot!(resp, @r###"
            {
              "data": {
                "__typename": "Query"
              }
            }
            "###);

        tokio::time::sleep(METRICS_DELAY).await;

        let query = indoc::indoc! {r#"
                SELECT Count, Attributes
                FROM otel_metrics_exponential_histogram
                WHERE ServiceName = ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'grafbase.hook.duration'
            "#};

        let row = clickhouse
            .query(query)
            .bind(&service_name)
            .fetch_optional::<ExponentialHistogramRow>()
            .await
            .unwrap();

        insta::assert_json_snapshot!(row, @r###"
            {
              "Count": 1,
              "Attributes": {
                "grafbase.hook.name": "on-gateway-request",
                "grafbase.hook.status": "SUCCESS"
              }
            }
            "###);
    });
}

#[test]
fn on_gateway_request_host_error() {
    let config = indoc::indoc! {r#"
        [hooks]
        location = "../../../engine/crates/wasi-component-loader/examples/target/wasm32-wasip1/debug/simple.wasm"
    "#};

    with_custom_gateway(config, |service_name, _, gateway, clickhouse| async move {
        let resp = gateway
            .gql::<serde_json::Value>("query SimpleQuery { __typename }")
            .send()
            .await;

        insta::assert_json_snapshot!(resp, @r###"
            {
              "errors": [
                {
                  "message": "Internal hook error",
                  "extensions": {
                    "code": "HOOK_ERROR"
                  }
                }
              ]
            }
            "###);

        tokio::time::sleep(METRICS_DELAY).await;

        let query = indoc::indoc! {r#"
                SELECT Count, Attributes
                FROM otel_metrics_exponential_histogram
                WHERE ServiceName = ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'grafbase.hook.duration'
            "#};

        let row = clickhouse
            .query(query)
            .bind(&service_name)
            .fetch_optional::<ExponentialHistogramRow>()
            .await
            .unwrap();

        insta::assert_json_snapshot!(row, @r###"
            {
              "Count": 1,
              "Attributes": {
                "grafbase.hook.name": "on-gateway-request",
                "grafbase.hook.status": "HOST_ERROR"
              }
            }
            "###);
    });
}

#[test]
fn on_gateway_request_guest_error() {
    let config = indoc::indoc! {r#"
        [hooks]
        location = "../../../engine/crates/wasi-component-loader/examples/target/wasm32-wasip1/debug/error.wasm"
    "#};

    with_custom_gateway(config, |service_name, _, gateway, clickhouse| async move {
        let resp = gateway
            .gql::<serde_json::Value>("query SimpleQuery { __typename }")
            .send()
            .await;

        insta::assert_json_snapshot!(resp, @r###"
            {
              "errors": [
                {
                  "message": "not found",
                  "extensions": {
                    "my": "extension",
                    "code": "BAD_REQUEST"
                  }
                }
              ]
            }
            "###);

        tokio::time::sleep(METRICS_DELAY).await;

        let query = indoc::indoc! {r#"
                SELECT Count, Attributes
                FROM otel_metrics_exponential_histogram
                WHERE ServiceName = ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'grafbase.hook.duration'
            "#};

        let row = clickhouse
            .query(query)
            .bind(&service_name)
            .fetch_optional::<ExponentialHistogramRow>()
            .await
            .unwrap();

        insta::assert_json_snapshot!(row, @r###"
            {
              "Count": 1,
              "Attributes": {
                "grafbase.hook.name": "on-gateway-request",
                "grafbase.hook.status": "GUEST_ERROR"
              }
            }
            "###);
    });
}
