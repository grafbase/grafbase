use std::{
    collections::BTreeMap,
    sync::Arc,
    time::{Duration, UNIX_EPOCH},
};

use futures_util::Future;
use indoc::formatdoc;
use serde::Deserialize;

use crate::{load_schema, with_static_server, Client};

mod operation;
mod request;

#[serde_with::serde_as]
#[derive(clickhouse::Row, Deserialize)]
struct SumMetricCountRow {
    #[serde(rename = "Value")]
    value: f64,
    #[serde(rename = "Attributes")]
    #[serde_as(as = "Vec<(_, _)>")]
    attributes: BTreeMap<String, String>,
}

#[serde_with::serde_as]
#[derive(clickhouse::Row, Deserialize)]
struct ExponentialHistogramRow {
    #[serde(rename = "Count")]
    count: u64,
    #[serde(rename = "Attributes")]
    #[serde_as(as = "Vec<(_, _)>")]
    attributes: BTreeMap<String, String>,
}

fn with_gateway<T, F>(test: T)
where
    T: FnOnce(String, u64, Arc<Client>, clickhouse::Client) -> F,
    F: Future<Output = ()>,
{
    let service_name = format!("service_{}", ulid::Ulid::new());
    let config = &formatdoc! {r#"
        [telemetry]
        service_name = "{service_name}"

        [telemetry.tracing]
        enabled = true
        sampling = 1

        [telemetry.tracing.exporters.otlp]
        enabled = true
        endpoint = "http://localhost:4318"
        protocol = "grpc"

        [telemetry.tracing.exporters.otlp.batch_export]
        scheduled_delay = 1
        max_export_batch_size = 1
    "#};

    let schema = load_schema("big");
    let clickhouse = clickhouse::Client::default()
        .with_url("http://localhost:8124")
        .with_user("default")
        .with_database("otel");

    println!("service_name: {}", service_name);
    with_static_server(config, &schema, None, None, |client| async move {
        let start = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // wait for initial polling to be pushed
        tokio::time::sleep(Duration::from_secs(2)).await;

        test(service_name, start, client, clickhouse).await
    })
}
