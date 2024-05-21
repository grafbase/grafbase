use opentelemetry::{
    metrics::{Counter, Histogram, Meter},
    KeyValue,
};

use crate::grafbase_client::Client;

#[derive(Clone)]
pub struct GraphqlOperationMetrics {
    count: Counter<u64>,
    latency: Histogram<u64>,
}

pub struct GraphqlOperationMetricsAttributes {
    pub ty: &'static str,
    pub name: Option<String>,
    pub normalized_query: String,
    pub normalized_query_hash: [u8; 32],
    pub has_errors: bool,
    pub cache_status: Option<String>,
    pub client: Option<Client>,
}

impl GraphqlOperationMetrics {
    pub fn build(meter: &Meter) -> Self {
        Self {
            count: meter.u64_counter("gql_operation_count").init(),
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
            has_errors,
            cache_status,
            client,
        }: GraphqlOperationMetricsAttributes,
        latency: std::time::Duration,
    ) {
        use base64::{engine::general_purpose::STANDARD, Engine as _};
        let normalized_query_hash = STANDARD.encode(normalized_query_hash);
        let name = name.unwrap_or_default();
        let mut attributes = vec![
            KeyValue::new("gql.operation.normalized_query_hash", normalized_query_hash.clone()),
            KeyValue::new("gql.operation.normalized_query", normalized_query),
            KeyValue::new("gql.operation.type", ty),
            KeyValue::new("gql.operation.name", name.clone()),
        ];
        if let Some(cache_status) = cache_status {
            attributes.push(KeyValue::new("gql.response.cache_status", cache_status));
        }
        if has_errors {
            attributes.push(KeyValue::new("gql.response.has_errors", "true"));
        }
        if let Some(client) = client {
            attributes.push(KeyValue::new("http.headers.x-grafbase-client-name", client.name));
            if let Some(version) = client.version {
                attributes.push(KeyValue::new("http.headers.x-grafbase-client-version", version));
            }
        }
        self.count.add(1, &attributes);
        self.latency.record(latency.as_millis() as u64, &attributes);
    }
}
