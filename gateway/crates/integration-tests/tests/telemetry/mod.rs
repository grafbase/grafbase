use std::{collections::HashMap, time::Duration};

use gateway_integration_tests::clickhouse_client;
use indoc::{formatdoc, indoc};
use serde::Deserialize;

use crate::{load_schema, with_static_server};

mod logs;
mod metrics;
mod tracing;

#[test]
fn with_stdout_telemetry() {
    let config = indoc! {r#"
        [telemetry]
        service_name = "meow"

        [telemetry.exporters.stdout]
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
        let result = client.execute(query).await.into_body();
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
        let result = client.execute(query).await.into_body();
        serde_json::to_string_pretty(&result).unwrap();

        tokio::time::sleep(Duration::from_secs(2)).await;

        let client = clickhouse_client();

        let Row { resource_attributes } = client
            .query("SELECT ResourceAttributes FROM otel_traces WHERE ServiceName = ?")
            .bind(&service_name)
            .fetch_one()
            .await
            .unwrap();

        let expected_resource_attributes = [("service.name", service_name.as_str())]
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect::<HashMap<_, _>>();

        assert_eq!(resource_attributes, expected_resource_attributes);

        let Row { resource_attributes } = client
            .query("SELECT ResourceAttributes FROM otel_metrics_exponential_histogram WHERE ResourceAttributes['service.name'] = ?")
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
    println!("Service name: {service_name}");

    let config = &formatdoc! {r#"
        [telemetry]
        service_name = "{service_name}"

        [telemetry.resource_attributes]
        my-favorite-app = "graphabase"

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
        let response = client.execute(query).await.into_body();
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "__typename": "Query"
          }
        }
        "###);

        tokio::time::sleep(Duration::from_secs(2)).await;

        let client = clickhouse_client();

        let Row { resource_attributes } = client
            .query("SELECT ResourceAttributes FROM otel_traces WHERE ServiceName = ?")
            .bind(&service_name)
            .fetch_one()
            .await
            .unwrap();

        let expected_resource_attributes = [
            ("service.name", service_name.as_str()),
            ("my-favorite-app", "graphabase"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect::<HashMap<_, _>>();

        assert_eq!(resource_attributes, expected_resource_attributes);

        let Row { resource_attributes } = client
            .query("SELECT ResourceAttributes FROM otel_metrics_exponential_histogram WHERE ResourceAttributes['service.name'] = ?")
            .bind(&service_name)
            .fetch_one()
            .await
            .unwrap();

        assert_eq!(resource_attributes, expected_resource_attributes);
    });
}
