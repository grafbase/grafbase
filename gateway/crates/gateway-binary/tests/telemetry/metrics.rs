use std::{collections::HashMap, time::Duration};

use indoc::formatdoc;
use serde::Deserialize;

use crate::{load_schema, with_static_server};

#[serde_with::serde_as]
#[derive(clickhouse::Row, Deserialize)]
struct SumMetricCountRow {
    #[serde(rename = "Value")]
    value: f64,
    #[serde(rename = "Attributes")]
    #[serde_as(as = "Vec<(_, _)>")]
    attributes: HashMap<String, String>,
}

#[serde_with::serde_as]
#[derive(clickhouse::Row, Deserialize)]
struct ExponentialHistogramRow {
    #[serde(rename = "Count")]
    count: u64,
    #[serde(rename = "Attributes")]
    #[serde_as(as = "Vec<(_, _)>")]
    attributes: HashMap<String, String>,
}

#[test]
fn request_metrics() {
    let service_name = format!("service-{}", rand::random::<u128>());
    let config = &formatdoc! {r#"
        [telemetry]
        service_name = "{service_name}"

        [telemetry.tracing]
        enabled = true
        sampling = 1

        [telemetry.tracing.exporters.otlp]
        enabled = true
        endpoint = "http://localhost:4317"
        protocol = "grpc"

        [telemetry.tracing.exporters.otlp.batch_export]
        scheduled_delay = 1
        max_export_batch_size = 1
    "#};

    let schema = load_schema("big");
    with_static_server(config, &schema, None, None, |client| async move {
        client.gql::<serde_json::Value>("{ __typename }").send().await;

        tokio::time::sleep(Duration::from_secs(2)).await;

        let client = clickhouse::Client::default()
            .with_url("http://localhost:8123")
            .with_user("default")
            .with_database("otel");

        let SumMetricCountRow { value, attributes } = client
            .query(
                r#"
                SELECT Value, Attributes
                FROM otel_metrics_sum
                WHERE ResourceAttributes['service.name'] = ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'request_count'
                "#,
            )
            .bind(&service_name)
            .fetch_one()
            .await
            .unwrap();
        assert!(value >= 1.0); // initial polling also counts
        assert_eq!(
            attributes,
            HashMap::from([("http.response.status_code".to_string(), "200".to_string())])
        );

        let ExponentialHistogramRow { count, attributes } = client
            .query(
                r#"
                SELECT Count, Attributes
                FROM otel_metrics_exponential_histogram
                WHERE ResourceAttributes['service.name'] = ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'request_latency'
                "#,
            )
            .bind(&service_name)
            .fetch_one()
            .await
            .unwrap();
        assert!(count >= 1); // Initial polling also counts
        assert_eq!(attributes, HashMap::new())
    });
}

#[test]
fn operation_metrics() {
    let service_name = format!("service-{}", rand::random::<u128>());
    let config = &formatdoc! {r#"
        [telemetry]
        service_name = "{service_name}"

        [telemetry.tracing]
        enabled = true
        sampling = 1

        [telemetry.tracing.exporters.otlp]
        enabled = true
        endpoint = "http://localhost:4317"
        protocol = "grpc"

        [telemetry.tracing.exporters.otlp.batch_export]
        scheduled_delay = 1
        max_export_batch_size = 1
    "#};

    let schema = load_schema("big");
    with_static_server(config, &schema, None, None, |client| async move {
        client
            .gql::<serde_json::Value>("query Simple { __typename }")
            .send()
            .await;

        tokio::time::sleep(Duration::from_secs(2)).await;

        let client = clickhouse::Client::default()
            .with_url("http://localhost:8123")
            .with_user("default")
            .with_database("otel");

        let SumMetricCountRow { value, attributes } = client
            .query(
                r#"
                SELECT Value, Attributes
                FROM otel_metrics_sum
                WHERE ResourceAttributes['service.name'] = ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'gql_operation_count'
                "#,
            )
            .bind(&service_name)
            .fetch_one()
            .await
            .unwrap();
        assert_eq!(value, 1.0);
        assert_eq!(
            attributes,
            HashMap::from([
                ("gql.operation.name".to_string(), "Simple".to_string()),
                ("gql.operation.id".to_string(), "".to_string())
            ])
        );

        let ExponentialHistogramRow { count, attributes } = client
            .query(
                r#"
                SELECT Count, Attributes
                FROM otel_metrics_exponential_histogram
                WHERE ResourceAttributes['service.name'] = ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'gql_operation_latency'
                "#,
            )
            .bind(&service_name)
            .fetch_one()
            .await
            .unwrap();
        assert_eq!(count, 1);
        assert_eq!(
            attributes,
            HashMap::from([
                ("gql.operation.name".to_string(), "Simple".to_string()),
                ("gql.operation.id".to_string(), "".to_string())
            ])
        );
    });
}
