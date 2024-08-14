use opentelemetry::{
    metrics::{Counter, Histogram, Meter, UpDownCounter},
    KeyValue,
};

use crate::{
    gql_response_status::{GraphqlResponseStatus, SubgraphResponseStatus},
    grafbase_client::Client,
};

#[derive(Clone)]
pub struct GraphqlOperationMetrics {
    operation_latency: Histogram<u64>,
    subgraph_latency: Histogram<u64>,
    subgraph_retries: Counter<u64>,
    subgraph_request_body_size: Histogram<u64>,
    subgraph_response_body_size: Histogram<u64>,
    subgraph_requests_inflight: UpDownCounter<i64>,
    subgraph_cache_hits: Counter<u64>,
    subgraph_cache_misses: Counter<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum OperationType {
    Query,
    Mutation,
    Subscription,
}

impl OperationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Query => "query",
            Self::Mutation => "mutation",
            Self::Subscription => "subscription",
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OperationMetricsAttributes {
    pub ty: OperationType,
    pub name: Option<String>,
    pub sanitized_query: String,
    pub sanitized_query_hash: [u8; 32],
    /// For a schema:
    /// ```ignore
    /// type Query {
    ///    user(id: ID!): User
    /// }
    ///
    /// type User {
    ///   id: ID!
    ///   name: String
    /// }
    /// ```
    /// and query:
    /// ```ignore
    /// query {
    ///   user(id: "0x1") {
    ///     id
    ///     name
    ///   }
    /// }
    /// ```
    /// We expected the following string
    /// ```
    /// "Query.user,User.id+name"
    /// ```
    pub used_fields: String,
}

#[derive(Debug)]
pub struct GraphqlRequestMetricsAttributes {
    pub operation: OperationMetricsAttributes,
    pub status: GraphqlResponseStatus,
    pub cache_status: Option<String>,
    pub client: Option<Client>,
}

#[derive(Debug)]
pub struct SubgraphRequestDurationAttributes {
    pub name: String,
    pub status: SubgraphResponseStatus,
}

#[derive(Debug)]
pub struct SubgraphRequestRetryAttributes {
    pub name: String,
    pub aborted: bool,
}

#[derive(Debug)]
pub struct SubgraphRequestBodySizeAttributes {
    pub name: String,
}

#[derive(Debug)]
pub struct SubgraphResponseBodySizeAttributes {
    pub name: String,
}

#[derive(Debug)]
pub struct SubgraphInFlightRequestAttributes {
    pub name: String,
}

#[derive(Debug)]
pub struct SubgraphCacheHitAttributes {
    pub name: String,
}

#[derive(Debug)]
pub struct SubgraphCacheMissAttributes {
    pub name: String,
}

impl GraphqlOperationMetrics {
    pub fn build(meter: &Meter) -> Self {
        Self {
            operation_latency: meter.u64_histogram("gql_operation_latency").init(),
            subgraph_latency: meter.u64_histogram("graphql.subgraph.request.duration").init(),
            subgraph_retries: meter.u64_counter("graphql.subgraph.request.retries").init(),
            subgraph_request_body_size: meter.u64_histogram("graphql.subgraph.request.body.size").init(),
            subgraph_response_body_size: meter.u64_histogram("graphql.subgraph.response.body.size").init(),
            subgraph_requests_inflight: meter.i64_up_down_counter("graphql.subgraph.request.inflight").init(),
            subgraph_cache_hits: meter.u64_counter("graphql.subgraph.request.cache.hit").init(),
            subgraph_cache_misses: meter.u64_counter("graphql.subgraph.request.cache.miss").init(),
        }
    }

    pub fn record_operation(
        &self,
        GraphqlRequestMetricsAttributes {
            operation:
                OperationMetricsAttributes {
                    name,
                    ty,
                    sanitized_query,
                    sanitized_query_hash,
                    used_fields,
                },
            status,
            cache_status,
            client,
        }: GraphqlRequestMetricsAttributes,
        latency: std::time::Duration,
    ) {
        use base64::{engine::general_purpose::STANDARD, Engine as _};
        let sanitized_query_hash = STANDARD.encode(sanitized_query_hash);

        let mut attributes = vec![
            KeyValue::new("gql.operation.query_hash", sanitized_query_hash),
            KeyValue::new("gql.operation.query", sanitized_query),
            KeyValue::new("gql.operation.type", ty.as_str()),
            KeyValue::new("gql.operation.used_fields", used_fields),
        ];

        if let Some(name) = name {
            attributes.push(KeyValue::new("gql.operation.name", name));
        }

        if let Some(cache_status) = cache_status {
            attributes.push(KeyValue::new("gql.response.cache_status", cache_status));
        }

        attributes.push(KeyValue::new("gql.response.status", status.as_str()));

        if let Some(client) = client {
            attributes.push(KeyValue::new("http.headers.x-grafbase-client-name", client.name));
            if let Some(version) = client.version {
                attributes.push(KeyValue::new("http.headers.x-grafbase-client-version", version));
            }
        }

        self.operation_latency.record(latency.as_millis() as u64, &attributes);
    }

    pub fn record_subgraph_duration(
        &self,
        SubgraphRequestDurationAttributes { name, status }: SubgraphRequestDurationAttributes,
        latency: std::time::Duration,
    ) {
        let attributes = vec![
            KeyValue::new("graphql.subgraph.name", name),
            KeyValue::new("graphql.subgraph.response.status", status.as_str()),
        ];

        self.subgraph_latency.record(latency.as_millis() as u64, &attributes);
    }

    pub fn record_subgraph_retry(
        &self,
        SubgraphRequestRetryAttributes { name, aborted }: SubgraphRequestRetryAttributes,
    ) {
        let attributes = vec![
            KeyValue::new("graphql.subgraph.name", name),
            KeyValue::new("graphql.subgraph.aborted", aborted),
        ];

        self.subgraph_retries.add(1, &attributes);
    }

    pub fn record_subgraph_request_size(
        &self,
        SubgraphRequestBodySizeAttributes { name }: SubgraphRequestBodySizeAttributes,
        size: usize,
    ) {
        let attributes = vec![KeyValue::new("graphql.subgraph.name", name)];
        self.subgraph_request_body_size.record(size as u64, &attributes);
    }

    pub fn record_subgraph_response_size(
        &self,
        SubgraphResponseBodySizeAttributes { name }: SubgraphResponseBodySizeAttributes,
        size: usize,
    ) {
        let attributes = vec![KeyValue::new("graphql.subgraph.name", name)];
        self.subgraph_response_body_size.record(size as u64, &attributes);
    }

    pub fn increment_subgraph_inflight_requests(
        &self,
        SubgraphInFlightRequestAttributes { name }: SubgraphInFlightRequestAttributes,
    ) {
        let attributes = vec![KeyValue::new("graphql.subgraph.name", name)];
        self.subgraph_requests_inflight.add(1, &attributes);
    }

    pub fn decrement_subgraph_inflight_requests(
        &self,
        SubgraphInFlightRequestAttributes { name }: SubgraphInFlightRequestAttributes,
    ) {
        let attributes = vec![KeyValue::new("graphql.subgraph.name", name)];
        self.subgraph_requests_inflight.add(-1, &attributes);
    }

    pub fn record_subgraph_cache_hit(&self, SubgraphCacheHitAttributes { name }: SubgraphCacheHitAttributes) {
        let attributes = vec![KeyValue::new("graphql.subgraph.name", name)];
        self.subgraph_cache_hits.add(1, &attributes);
    }

    pub fn record_subgraph_cache_miss(&self, SubgraphCacheMissAttributes { name }: SubgraphCacheMissAttributes) {
        let attributes = vec![KeyValue::new("graphql.subgraph.name", name)];
        self.subgraph_cache_misses.add(1, &attributes);
    }
}
