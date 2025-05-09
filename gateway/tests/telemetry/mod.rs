use std::{collections::HashMap, time::Duration};

use indoc::{formatdoc, indoc};
use serde::Deserialize;

use crate::{load_schema, with_static_server};

mod logs;
mod metrics;
mod tracing;

enum Protocol {
    Grpc,
    Http,
}

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
        sampling = 1

        [telemetry.exporters.otlp]
        enabled = true
        endpoint = "http://localhost:4327"
        protocol = "grpc"

        [telemetry.exporters.otlp.batch_export]
        scheduled_delay = "1s"
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
            .query("SELECT ResourceAttributes FROM otel_traces WHERE ServiceName = ?")
            .bind(&service_name)
            .fetch_one()
            .await
            .unwrap();

        let attribute = resource_attributes.get("service.name");
        assert_eq!(attribute.unwrap(), service_name.as_str());

        let Row { resource_attributes } = client
            .query("SELECT ResourceAttributes FROM otel_metrics_exponential_histogram WHERE ResourceAttributes['service.name'] = ?")
            .bind(&service_name)
            .fetch_one()
            .await
            .unwrap();

        let attribute = resource_attributes.get("service.name");
        assert_eq!(attribute.unwrap(), service_name.as_str());
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
        endpoint = "http://localhost:4327"
        protocol = "grpc"

        [telemetry.exporters.otlp.batch_export]
        scheduled_delay = "1s"
        max_export_batch_size = 1
    "#};

    let schema = load_schema("big");

    let query = indoc! {r#"
        { __typename }
    "#};

    with_static_server(config, &schema, None, None, |client| async move {
        let response: serde_json::Value = client.gql(query).send().await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "__typename": "Query"
          }
        }
        "###);

        tokio::time::sleep(Duration::from_secs(2)).await;

        let client = crate::clickhouse_client();

        let Row { resource_attributes } = client
            .query("SELECT ResourceAttributes FROM otel_traces WHERE ServiceName = ?")
            .bind(&service_name)
            .fetch_one()
            .await
            .unwrap();

        let attribute = resource_attributes.get("my-favorite-app").unwrap();
        assert_eq!(attribute, "graphabase");

        let Row { resource_attributes } = client
            .query("SELECT ResourceAttributes FROM otel_metrics_exponential_histogram WHERE ResourceAttributes['service.name'] = ?")
            .bind(&service_name)
            .fetch_one()
            .await
            .unwrap();

        let attribute = resource_attributes.get("my-favorite-app").unwrap();
        assert_eq!(attribute, "graphabase");
    });
}
