use std::net::SocketAddr;

use opentelemetry::{
    metrics::{Histogram, Meter, UpDownCounter},
    KeyValue,
};

use crate::{grafbase_client::Client, graphql::GraphqlResponseStatus};

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
    pub url_scheme: Option<String>,
    pub route: Option<String>,
    pub listen_address: Option<SocketAddr>,
    pub version: Option<http::Version>,
    pub method: Option<http::Method>,
}

impl RequestMetrics {
    pub fn build(meter: &Meter) -> Self {
        Self {
            latency: meter.u64_histogram("http.server.request.duration").init(),
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
            method,
            url_scheme,
            route,
            listen_address,
            version,
        }: RequestMetricsAttributes,
        latency: std::time::Duration,
    ) {
        let mut attributes = vec![KeyValue::new("http.response.status_code", status_code as i64)];

        if let Some(method) = method {
            attributes.push(KeyValue::new("http.request.method", method.to_string()));
        }

        if let Some(route) = route {
            attributes.push(KeyValue::new("http.route", route));
        }

        if let Some(version) = version {
            attributes.push(KeyValue::new("network.protocol.version", format!("{version:?}")));
        }

        if let Some(listen_address) = listen_address {
            attributes.push(KeyValue::new("server.address", listen_address.ip().to_string()));
            attributes.push(KeyValue::new("server.port", listen_address.port() as i64));
        }

        if let Some(scheme) = url_scheme {
            attributes.push(KeyValue::new("url.scheme", scheme.to_string()));
        }

        if let Some(cache_status) = cache_status {
            attributes.push(KeyValue::new("http.response.headers.cache.status", cache_status));
        }

        if let Some(client) = client {
            attributes.push(KeyValue::new("http.headers.x-grafbase-client-name", client.name));

            if let Some(version) = client.version {
                attributes.push(KeyValue::new("http.headers.x-grafbase-client-version", version));
            }
        }

        if let Some(status) = gql_status {
            attributes.push(KeyValue::new("graphql.response.status", status.as_str()));
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
