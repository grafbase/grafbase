use std::{
    collections::BTreeMap,
    sync::Arc,
    time::{Duration, UNIX_EPOCH},
};

use futures_util::Future;
use indoc::formatdoc;
use serde::{Deserialize, Serialize};

use crate::{clickhouse_client, load_schema, with_static_server, Client};

mod operation;
mod request;

#[serde_with::serde_as]
#[derive(Debug, clickhouse::Row, Deserialize, Serialize, PartialEq)]
struct SumMetricCountRow {
    #[serde(rename = "Value")]
    value: f64,
    #[serde(rename = "Attributes")]
    #[serde_as(deserialize_as = "Vec<(_, _)>")]
    attributes: BTreeMap<String, String>,
}

#[serde_with::serde_as]
#[derive(Debug, clickhouse::Row, Deserialize, Serialize, PartialEq)]
struct ExponentialHistogramRow {
    #[serde(rename = "Count")]
    count: u64,
    #[serde(rename = "Attributes")]
    #[serde_as(deserialize_as = "Vec<(_, _)>")]
    attributes: BTreeMap<String, String>,
}

fn with_gateway<T, F>(test: T)
where
    T: FnOnce(String, u64, Arc<Client>, &'static clickhouse::Client) -> F,
    F: Future<Output = ()>,
{
    let service_name = format!("service_{}", ulid::Ulid::new());
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
    let clickhouse = clickhouse_client();

    println!("service_name: {}", service_name);
    with_static_server(config, &schema, None, None, |client| async move {
        const WAIT_SECONDS: u64 = 2;
        let start = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + WAIT_SECONDS;

        // wait for initial polling to be pushed to OTEL tables so we can ignore it with the
        // appropriate start time filter.
        tokio::time::sleep(Duration::from_secs(WAIT_SECONDS)).await;

        test(service_name, start, client, clickhouse).await
    })
}
