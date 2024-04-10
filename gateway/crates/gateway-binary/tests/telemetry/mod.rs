use std::{collections::HashMap, time::Duration};

use crate::{load_schema, with_static_server};
use indoc::{formatdoc, indoc};
use serde::Deserialize;

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

#[serde_with::serde_as]
#[derive(clickhouse::Row, Deserialize)]
struct Row {
    #[serde(rename = "ResourceAttributes")]
    #[serde_as(as = "Vec<(_, _)>")]
    resource_attributes: HashMap<String, String>,
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

        let Row { resource_attributes } = client
            .query("SELECT ResourceAttributes FROM otel_traces WHERE ServiceName = ?")
            .bind(&service_name)
            .fetch_one()
            .await
            .unwrap();

        let expected_resource_attributes = [
            ("service.name", service_name.as_str()),
            ("grafbase.graph_id", "0"),
            ("grafbase.branch_id", "0"),
            ("grafbase.branch_name", ""),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect::<HashMap<_, _>>();

        assert_eq!(resource_attributes, expected_resource_attributes);

        // takes a bit more time to push metrics, currently only every 10s
        tokio::time::sleep(Duration::from_secs(5)).await;

        let Row { resource_attributes } = client
            .query("SELECT ResourceAttributes FROM otel_metrics_sum WHERE ResourceAttributes['service.name'] = ?")
            .bind(&service_name)
            .fetch_one()
            .await
            .unwrap();

        assert_eq!(resource_attributes, expected_resource_attributes);
    });
}
#[test]
fn extra_resource_attributes() {
    let service_name = format!("service-{}", rand::random::<u128>());
    let config = &formatdoc! {r#"
        [telemetry]
        service_name = "{service_name}"

        [telemetry.resource_attributes]
        my-favorite-app = "graphabase"

        [telemetry.tracing]
        enabled = true
        sampling = 1

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

        let Row { resource_attributes } = client
            .query("SELECT ResourceAttributes FROM otel_traces WHERE ServiceName = ?")
            .bind(&service_name)
            .fetch_one()
            .await
            .unwrap();

        let expected_resource_attributes = [
            ("service.name", service_name.as_str()),
            ("my-favorite-app", "graphabase"),
            ("grafbase.graph_id", "0"),
            ("grafbase.branch_id", "0"),
            ("grafbase.branch_name", ""),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect::<HashMap<_, _>>();

        assert_eq!(resource_attributes, expected_resource_attributes);

        // takes a bit more time to push metrics, currently only every 10s
        tokio::time::sleep(Duration::from_secs(5)).await;

        let Row { resource_attributes } = client
            .query("SELECT ResourceAttributes FROM otel_metrics_sum WHERE ResourceAttributes['service.name'] = ?")
            .bind(&service_name)
            .fetch_one()
            .await
            .unwrap();

        assert_eq!(resource_attributes, expected_resource_attributes);
    });
}
