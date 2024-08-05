use opentelemetry::{
    metrics::{Histogram, Meter},
    KeyValue,
};

use crate::{gql_response_status::GraphqlResponseStatus, grafbase_client::Client};

#[derive(Clone)]
pub struct GraphqlOperationMetrics {
    latency: Histogram<u64>,
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

impl GraphqlOperationMetrics {
    pub fn build(meter: &Meter) -> Self {
        Self {
            latency: meter.u64_histogram("gql_operation_latency").init(),
        }
    }

    pub fn record(
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
        self.latency.record(latency.as_millis() as u64, &attributes);
    }
}
