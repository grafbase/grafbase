mod builder;

pub use builder::*;
pub use grafbase_telemetry::graphql::OperationType;

use grafbase_telemetry::graphql::GraphqlResponseStatus;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

/// Represents the different types of events that can be collected by the event queue.
#[derive(Debug)]
pub enum Event {
    /// A GraphQL operation execution event
    Operation(ExecutedOperation),
    /// A federated subgraph request event
    Subgraph(ExecutedSubgraphRequest),
    /// A generic HTTP request event
    Http(ExecutedHttpRequest),
    /// A custom extension-generated event
    Extension(ExtensionEvent),
}

/// Represents a completed GraphQL operation execution.
#[derive(Debug)]
pub struct ExecutedOperation {
    pub name: Option<String>,
    pub document: Arc<str>,
    pub prepare_duration: Duration,
    pub duration: Duration,
    pub cached_plan: bool,
    pub status: GraphqlResponseStatus,
    pub operation_type: OperationType,
    pub complexity: Option<u64>,
    pub has_deprecated_fields: bool,
}

impl ExecutedOperation {
    /// Creates a new builder for constructing an `ExecutedOperation`.
    ///
    /// # Arguments
    ///
    /// * `operation_type` - The type of GraphQL operation
    pub fn builder<'a>(operation_type: OperationType) -> ExecutedOperationBuilder<'a> {
        ExecutedOperationBuilder {
            name: None,
            document: None,
            start_time: Instant::now(),
            prepare_duration: None,
            cached_plan: false,
            status: GraphqlResponseStatus::Success,
            operation_type,
            complexity: None,
            has_deprecated_fields: false,
        }
    }

    /// Creates a new builder for constructing an `ExecutedOperation` with a default Query type.
    ///
    /// The operation type can be changed later using the `operation_type()` method on the builder.
    pub fn builder_with_default<'a>() -> ExecutedOperationBuilder<'a> {
        Self::builder(OperationType::Query)
    }
}

/// Represents a completed request to a federated subgraph.
#[derive(Debug)]
pub struct ExecutedSubgraphRequest {
    pub subgraph_name: String,
    pub method: http::Method,
    pub url: String,
    pub executions: Vec<RequestExecution>,
    pub cache_status: CacheStatus,
    pub total_duration: Duration,
    pub has_errors: bool,
}

impl ExecutedSubgraphRequest {
    /// Creates a new builder for constructing an `ExecutedSubgraphRequest`.
    ///
    /// # Arguments
    ///
    /// * `subgraph_name` - The name of the target subgraph
    /// * `method` - The HTTP method to use
    /// * `url` - The URL of the subgraph endpoint
    pub fn builder<'a>(
        subgraph_name: &'a str,
        method: http::Method,
        url: &'a str,
    ) -> ExecutedSubgraphRequestBuilder<'a> {
        ExecutedSubgraphRequestBuilder {
            start_time: Instant::now(),
            subgraph_name,
            method,
            url,
            executions: Vec::new(),
            cache_status: CacheStatus::Miss,
            has_errors: false,
            graphql_response_status: GraphqlResponseStatus::Success,
        }
    }
}

/// Represents the outcome of a single subgraph request attempt.
#[derive(Debug, Clone)]
pub enum RequestExecution {
    /// The subgraph returned a 5xx status code
    InternalServerError,
    /// A network or connection error occurred
    RequestError,
    /// The request was rate limited by the engine
    RateLimited,
    /// A successful response was received
    Response(SubgraphResponse),
}

/// Details about a successful subgraph response.
#[derive(Debug, Clone)]
pub struct SubgraphResponse {
    pub connection_time: Duration,
    pub response_time: Duration,
    pub status: http::StatusCode,
    pub headers: http::HeaderMap,
}

impl SubgraphResponse {
    /// Creates a new builder for constructing a `SubgraphResponse`.
    ///
    /// # Arguments
    ///
    /// * `status` - The HTTP status code of the response
    pub fn builder() -> SubgraphResponseBuilder {
        SubgraphResponseBuilder {
            connection_time: Duration::default(),
            response_time: Duration::default(),
            status: http::StatusCode::OK,
            headers: Default::default(),
            start_time: Instant::now(),
        }
    }
}

/// Indicates whether a subgraph response was served from cache.
#[derive(Debug, Clone, Copy)]
pub enum CacheStatus {
    /// The entire response was served from cache
    Hit,
    /// Part of the response was cached, but some data required fetching
    PartialHit,
    /// No cached data was available; a full fetch was required
    Miss,
}

/// Represents a completed HTTP request of the complete operation.
#[derive(Debug)]
pub struct ExecutedHttpRequest {
    pub method: http::Method,
    pub url: String,
    pub response_status: http::StatusCode,
}

impl ExecutedHttpRequest {
    /// Creates a new builder for constructing an `ExecutedHttpRequest`.
    ///
    /// # Arguments
    ///
    /// * `url` - The target URL for the request
    pub fn builder(url: &str) -> ExecutedHttpRequestBuilder<'_> {
        ExecutedHttpRequestBuilder {
            url,
            method: http::Method::POST,
            response_status: http::StatusCode::OK,
        }
    }
}

/// Represents a custom event emitted by an extension.
#[derive(Debug)]
pub struct ExtensionEvent {
    pub extension_name: String,
    pub event_name: String,
    pub data: Vec<u8>,
}
