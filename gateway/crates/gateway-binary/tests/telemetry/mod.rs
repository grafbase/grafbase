use std::time::Duration;

use indoc::{formatdoc, indoc};
use serde::Deserialize;

use crate::{load_schema, with_hybrid_server, with_static_server};

#[test]
fn with_stdout_telemetry() {
    let config = indoc! {r#"
        [telemetry]
        service_name = "meow"

        [telemetry.tracing.exporters.stdout]
        enabled = true
    "#};

    let schema = load_schema("big");

    let query = indoc! {r#"
        query Me {
          me {
            id
          }
        }
    "#};

    with_static_server(config, &schema, None, None, |client| async move {
        let result: serde_json::Value = client.gql(query).send().await;
        serde_json::to_string_pretty(&result).unwrap();
    })
}

#[test]
fn with_otel() {
    let service_name = format!("service-{}", rand::random::<u128>());
    let config = &formatdoc! {r#"
        [telemetry]
        service_name = "{service_name}"

        [telemetry.tracing]
        enabled = true
        sampling = 1

        [telemetry.tracing.exporters.stdout]
        enabled = true

        [telemetry.tracing.exporters.otlp]
        enabled = true
        endpoint = "http://localhost:4317"
        protocol = "grpc"
    "#};

    let schema = load_schema("big");

    let query = indoc! {r#"
        { __typename }
    "#};

    with_static_server(config, &schema, None, None, |client| async move {
        let result: serde_json::Value = client.gql(query).send().await;
        serde_json::to_string_pretty(&result).unwrap();

        tokio::time::sleep(Duration::from_secs(10)).await;

        let client = clickhouse::Client::default()
            .with_url("http://localhost:8123")
            .with_user("default")
            .with_database("otel");

        #[derive(clickhouse::Row, Deserialize)]
        struct CountRow {
            count: u32,
        }

        let CountRow { count } = client
            .query("SELECT COUNT(1) as count FROM otel_traces WHERE ServiceName = ?")
            .bind(&service_name)
            .fetch_one::<CountRow>()
            .await
            .unwrap();

        assert!(count > 0);

        // takes a bit more time to push metrics, currently only every 10s
        tokio::time::sleep(Duration::from_secs(5)).await;

        let CountRow { count } = client
            .query("SELECT COUNT(1) as count FROM otel_metrics_sum WHERE ResourceAttributes['service.name'] = ?")
            .bind(&service_name)
            .fetch_one::<CountRow>()
            .await
            .unwrap();

        assert!(count > 0);
    });
}

#[test]
fn with_otel_reload_tracing() {
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

    let query = indoc! {r#"
        { __typename }
    "#};

    with_hybrid_server(config, "test_graph", &schema, |client, uplink_mock| async move {
        let result: serde_json::Value = client.gql(query).send().await;
        serde_json::to_string_pretty(&result).unwrap();

        let client = clickhouse::Client::default()
            .with_url("http://localhost:8123")
            .with_user("default")
            .with_database("otel");

        #[derive(clickhouse::Row, Deserialize)]
        struct CountRow {
            count: u32,
        }

        // wait at least 2 seconds due to the async batch export config
        tokio::time::sleep(Duration::from_secs(2)).await;

        let CountRow { count } = client
            .query(
                r#"
                    SELECT COUNT(1) as count FROM otel_traces
                    WHERE ResourceAttributes['service.name'] = ?
                    AND ResourceAttributes['grafbase.branch_name'] = ?
                    AND ResourceAttributes['grafbase.branch_id'] = ?
                    AND ResourceAttributes['grafbase.graph_id'] = ?
                "#,
            )
            .bind(&service_name)
            .bind(&uplink_mock.branch)
            .bind(uplink_mock.branch_id.0.to_string())
            .bind(uplink_mock.graph_id.0.to_string())
            .fetch_one::<CountRow>()
            .await
            .unwrap();

        assert!(count > 0);
    });
}
