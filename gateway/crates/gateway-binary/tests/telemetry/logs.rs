use std::{collections::HashMap, time::Duration};

use indoc::{formatdoc, indoc};
use serde::Deserialize;

use crate::{load_schema, with_hybrid_server, with_static_server};

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
        sampling = 1

        [telemetry.exporters.otlp]
        enabled = true
        endpoint = "http://localhost:4318"
        protocol = "grpc"

        [telemetry.exporters.otlp.batch_export]
        scheduled_delay = 1
        max_export_batch_size = 1
    "#};

    let schema = load_schema("big");

    let query = indoc! {r#"
        { __typename }
    "#};

    with_static_server(config, &schema, None, None, |client| async move {
        let result: serde_json::Value = client.gql(query).send().await;
        serde_json::to_string_pretty(&result).unwrap();

        tokio::time::sleep(Duration::from_secs(2)).await;

        let client = crate::clickhouse_client();

        let Row { resource_attributes } = client
            .query("SELECT ResourceAttributes FROM otel_logs WHERE ServiceName = ?")
            .bind(&service_name)
            .fetch_one()
            .await
            .unwrap();

        let expected_resource_attributes = [("service.name", service_name.as_str())]
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect::<HashMap<_, _>>();

        assert_eq!(resource_attributes, expected_resource_attributes);
    });
}

#[test]
fn with_otel_reload() {
    let service_name = format!("service-{}", rand::random::<u128>());
    let config = &formatdoc! {r#"
        [telemetry]
        service_name = "{service_name}"

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

    let schema = load_schema("big");

    let query = indoc! {r#"
        { __typename }
    "#};

    with_hybrid_server(config, "test_graph", &schema, |client, _| async move {
        let result: serde_json::Value = client.gql(query).send().await;
        serde_json::to_string_pretty(&result).unwrap();

        tokio::time::sleep(Duration::from_secs(2)).await;

        let client = crate::clickhouse_client();

        let Row { resource_attributes } = client
            .query("SELECT ResourceAttributes FROM otel_logs WHERE ServiceName = ?")
            .bind(&service_name)
            .fetch_one()
            .await
            .unwrap();

        let expected_resource_attributes = [("service.name", service_name.as_str())]
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect::<HashMap<_, _>>();

        assert_eq!(resource_attributes, expected_resource_attributes);
    });
}

#[test]
fn with_otel_with_different_endpoint() {
    let service_name = format!("service-{}", rand::random::<u128>());
    let config = &formatdoc! {r#"
        [telemetry]
        service_name = "{service_name}"

        [telemetry.tracing]
        sampling = 1

        [telemetry.exporters.otlp]
        enabled = false
        endpoint = "http://localhost:6666"
        protocol = "grpc"

        [telemetry.exporters.otlp.batch_export]
        scheduled_delay = 1
        max_export_batch_size = 1

        [telemetry.logs.exporters.otlp]
        enabled = true
        endpoint = "http://localhost:4318"
        protocol = "grpc"

        [telemetry.logs.exporters.otlp.batch_export]
        scheduled_delay = 1
        max_export_batch_size = 1
    "#};

    let schema = load_schema("big");

    let query = indoc! {r#"
        { __typename }
    "#};

    with_static_server(config, &schema, None, None, |client| async move {
        let result: serde_json::Value = client.gql(query).send().await;
        serde_json::to_string_pretty(&result).unwrap();

        tokio::time::sleep(Duration::from_secs(2)).await;

        let client = crate::clickhouse_client();

        let Row { resource_attributes } = client
            .query("SELECT ResourceAttributes FROM otel_logs WHERE ServiceName = ?")
            .bind(&service_name)
            .fetch_one()
            .await
            .unwrap();

        let expected_resource_attributes = [("service.name", service_name.as_str())]
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect::<HashMap<_, _>>();

        assert_eq!(resource_attributes, expected_resource_attributes);
    });
}
