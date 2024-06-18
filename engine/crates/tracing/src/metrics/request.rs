use opentelemetry::{
    metrics::{Histogram, Meter},
    KeyValue,
};

use crate::{gql_response_status::GraphqlResponseStatus, grafbase_client::Client};

#[derive(Clone)]
pub struct RequestMetrics {
    latency: Histogram<u64>,
}

pub struct RequestMetricsAttributes {
    pub status_code: u16,
    pub cache_status: Option<String>,
    pub gql_status: Option<GraphqlResponseStatus>,
    pub client: Option<Client>,
}

impl RequestMetrics {
    pub fn build(meter: &Meter) -> Self {
        Self {
            latency: meter.u64_histogram("request_latency").init(),
        }
    }

    pub fn record(
        &self,
        RequestMetricsAttributes {
            status_code,
            cache_status,
            gql_status,
            client,
        }: RequestMetricsAttributes,
        latency: std::time::Duration,
    ) {
        let mut attributes = Vec::new();
        // No status code is simply assumed to be 200
        if status_code != 200 {
            attributes.push(KeyValue::new("http.response.status_code", status_code as i64));
        }
        if let Some(cache_status) = cache_status {
            attributes.push(KeyValue::new("http.response.headers.cache_status", cache_status));
        }
        if let Some(client) = client {
            attributes.push(KeyValue::new("http.headers.x-grafbase-client-name", client.name));
            if let Some(version) = client.version {
                attributes.push(KeyValue::new("http.headers.x-grafbase-client-version", version));
            }
        }
        // We only really care about keeping track of errors.
        if let Some(status) = gql_status.filter(|s| !s.is_success()) {
            attributes.push(KeyValue::new("gql.response.status", status.as_str()));
        }
        self.latency.record(latency.as_millis() as u64, &attributes);
    }
}
