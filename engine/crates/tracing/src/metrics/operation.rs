use opentelemetry::{
    metrics::{Histogram, Meter},
    KeyValue,
};

use crate::{gql_response_status::GraphqlResponseStatus, grafbase_client::Client};

#[derive(Clone)]
pub struct GraphqlOperationMetrics {
    latency: Histogram<u64>,
}

#[derive(Debug)]
pub struct GraphqlOperationMetricsAttributes {
    pub ty: &'static str,
    pub name: Option<String>,
    pub normalized_query: String,
    pub normalized_query_hash: [u8; 32],
    pub status: GraphqlResponseStatus,
    pub cache_status: Option<String>,
    pub client: Option<Client>,
}

impl GraphqlOperationMetrics {
    pub fn build(meter: &Meter) -> Self {
        Self {
            latency: meter.u64_histogram("gql_operation_latency").init(),
        }
    }

    pub fn record(
        &self,
        GraphqlOperationMetricsAttributes {
            name,
            ty,
            normalized_query,
            normalized_query_hash,
            status,
            cache_status,
            client,
        }: GraphqlOperationMetricsAttributes,
        latency: std::time::Duration,
    ) {
        use base64::{engine::general_purpose::STANDARD, Engine as _};
        let normalized_query_hash = STANDARD.encode(normalized_query_hash);
        let mut attributes = vec![
            KeyValue::new("gql.operation.normalized_query_hash", normalized_query_hash),
            KeyValue::new("gql.operation.normalized_query", normalized_query),
            KeyValue::new("gql.operation.type", ty),
        ];
        if let Some(name) = name {
            attributes.push(KeyValue::new("gql.operation.name", name));
        }
        if let Some(cache_status) = cache_status {
            attributes.push(KeyValue::new("gql.response.cache_status", cache_status));
        }
        // Not present will simply be assumed to be a success.
        if !status.is_success() {
            attributes.push(KeyValue::new("gql.response.status", status.as_str()));
        }
        if let Some(client) = client {
            attributes.push(KeyValue::new("http.headers.x-grafbase-client-name", client.name));
            if let Some(version) = client.version {
                attributes.push(KeyValue::new("http.headers.x-grafbase-client-version", version));
            }
        }
        self.latency.record(latency.as_millis() as u64, &attributes);
    }
}
