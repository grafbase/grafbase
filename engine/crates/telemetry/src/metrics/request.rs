use opentelemetry::{
    metrics::{Histogram, Meter, UpDownCounter},
    KeyValue,
};

use crate::{gql_response_status::GraphqlResponseStatus, grafbase_client::Client};

#[derive(Clone)]
pub struct RequestMetrics {
    latency: Histogram<u64>,
    connected_clients: UpDownCounter<i64>,
    response_body_sizes: Histogram<u64>,
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
            connected_clients: meter.i64_up_down_counter("http.server.connected.clients").init(),
            response_body_sizes: meter.u64_histogram("http.server.response.body.size").init(),
        }
    }

    pub fn record_http_duration(
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
        attributes.push(KeyValue::new("http.response.status_code", status_code as i64));
        if let Some(cache_status) = cache_status {
            attributes.push(KeyValue::new("http.response.headers.cache_status", cache_status));
        }
        if let Some(client) = client {
            attributes.push(KeyValue::new("http.headers.x-grafbase-client-name", client.name));
            if let Some(version) = client.version {
                attributes.push(KeyValue::new("http.headers.x-grafbase-client-version", version));
            }
        }
        if let Some(status) = gql_status {
            attributes.push(KeyValue::new("gql.response.status", status.as_str()));
        }
        self.latency.record(latency.as_millis() as u64, &attributes);
    }

    pub fn increment_connected_clients(&self) {
        self.connected_clients.add(1, &[]);
    }

    pub fn decrement_connected_clients(&self) {
        self.connected_clients.add(-1, &[]);
    }

    pub fn record_response_body_size(&self, size: u64) {
        self.response_body_sizes.record(size, &[]);
    }
}
