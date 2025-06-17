use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use grafbase_telemetry::graphql::GraphqlResponseStatus;

use super::{
    CacheStatus, ExecutedHttpRequest, ExecutedOperation, ExecutedSubgraphRequest, RequestExecution, SubgraphResponse,
};

/// Builder for constructing [`ExecutedOperation`] instances.
#[derive(Debug, Clone)]
pub struct ExecutedOperationBuilder<'a> {
    pub(super) name: Option<&'a str>,
    pub(super) document: Option<&'a Arc<str>>,
    pub(super) start_time: Instant,
    pub(super) prepare_duration: Option<Duration>,
    pub(super) cached_plan: bool,
    pub(super) status: GraphqlResponseStatus,
}

impl<'a> ExecutedOperationBuilder<'a> {
    /// Sets the operation name.
    ///
    /// This should match the operation name from the GraphQL document, if present.
    ///
    /// # Arguments
    ///
    /// * `name` - The operation name
    pub fn name(&mut self, name: &'a str) -> &mut Self {
        self.name = Some(name);
        self
    }

    /// Sets the GraphQL document for the operation.
    ///
    /// This should contain the complete GraphQL query, mutation, or subscription document.
    ///
    /// # Arguments
    ///
    /// * `document` - The GraphQL document as a string
    pub fn document(&mut self, document: &'a Arc<str>) -> &mut Self {
        self.document = Some(document);
        self
    }

    /// Records the duration of the preparation phase.
    ///
    /// This should be called when the operation preparation (parsing, validation,
    /// planning) is complete. It captures the elapsed time since the operation started.
    pub fn track_prepare(&mut self) -> Duration {
        let elapsed = self.start_time.elapsed();
        self.prepare_duration = Some(elapsed);

        elapsed
    }

    /// Sets whether a cached query plan was used.
    ///
    /// Query plan caching can significantly improve performance for repeated queries.
    ///
    /// # Arguments
    ///
    /// * `cached` - `true` if a cached plan was used, `false` otherwise
    pub fn cached_plan(&mut self, cached: bool) {
        self.cached_plan = cached;
    }

    /// Sets the response status for the operation.
    ///
    /// This indicates whether the operation completed successfully, had field errors,
    /// or encountered request-level errors.
    ///
    /// # Arguments
    ///
    /// * `status` - The GraphQL response status
    pub fn status(&mut self, status: GraphqlResponseStatus) -> &mut Self {
        self.status = status;
        self
    }

    /// Consumes the builder and creates an [`ExecutedOperation`].
    pub fn build(self) -> ExecutedOperation {
        ExecutedOperation {
            name: self.name.map(|s| s.to_string()),
            document: self.document.map(Clone::clone).unwrap_or_default(),
            prepare_duration: self.prepare_duration.unwrap_or_default(),
            duration: self.start_time.elapsed(),
            cached_plan: self.cached_plan,
            status: self.status,
        }
    }
}

/// Builder for constructing [`ExecutedSubgraphRequest`] instances.
#[derive(Debug, Clone)]
pub struct ExecutedSubgraphRequestBuilder<'a> {
    pub(super) subgraph_name: &'a str,
    pub(super) method: http::Method,
    pub(super) url: &'a str,
    pub(super) executions: Vec<RequestExecution>,
    pub(super) cache_status: CacheStatus,
    pub(super) total_duration: Duration,
    pub(super) has_errors: bool,
    pub(super) graphql_response_status: GraphqlResponseStatus,
}

impl<'a> ExecutedSubgraphRequestBuilder<'a> {
    /// Sets the list of execution attempts for this subgraph request.
    ///
    /// This includes all retry attempts and their outcomes.
    ///
    /// # Arguments
    ///
    /// * `executions` - Slice of execution attempts
    pub fn push_execution(&mut self, execution: RequestExecution) {
        self.executions.push(execution);
    }

    /// Sets the cache status for this request.
    ///
    /// Indicates whether the response was served from cache.
    ///
    /// # Arguments
    ///
    /// * `status` - The cache status (hit, partial hit, or miss)
    pub fn cache_status(&mut self, status: CacheStatus) {
        self.cache_status = status;
    }

    /// Sets the total duration for all execution attempts.
    ///
    /// This includes time spent on retries if any occurred.
    ///
    /// # Arguments
    ///
    /// * `duration` - The total duration across all attempts
    pub fn total_duration(&mut self, duration: Duration) {
        self.total_duration = duration;
    }

    /// Sets whether any errors occurred during execution.
    ///
    /// This includes both network errors and GraphQL errors in the response.
    ///
    /// # Arguments
    ///
    /// * `has_errors` - `true` if any errors occurred
    pub fn has_errors(&mut self, has_errors: bool) {
        self.has_errors = has_errors;
    }

    /// Consumes the builder and creates an [`ExecutedSubgraphRequest`].
    ///
    /// # Returns
    ///
    /// A new `ExecutedSubgraphRequest` instance with the configured values.
    pub fn build(self) -> ExecutedSubgraphRequest {
        ExecutedSubgraphRequest {
            subgraph_name: self.subgraph_name.to_string(),
            method: self.method,
            url: self.url.to_string(),
            executions: self.executions.to_vec(),
            cache_status: self.cache_status,
            total_duration: self.total_duration,
            has_errors: self.has_errors,
        }
    }

    /// Sets the GraphQL response status for this subgraph request.
    ///
    /// This indicates whether the subgraph request completed successfully, had field errors,
    /// or encountered request-level errors.
    ///
    /// # Arguments
    ///
    /// * `status` - The GraphQL response status
    pub fn graphql_response_status(&mut self, status: GraphqlResponseStatus) {
        self.graphql_response_status = status;
    }
}

/// Builder for constructing [`SubgraphResponse`] instances.
pub struct SubgraphResponseBuilder {
    pub(super) start_time: Instant,
    pub(super) connection_time: Duration,
    pub(super) response_time: Duration,
    pub(super) status: http::StatusCode,
    pub(super) headers: http::HeaderMap,
}

impl SubgraphResponseBuilder {
    /// Consumes the builder and creates a [`SubgraphResponse`].
    ///
    /// # Returns
    ///
    /// A new `SubgraphResponse` instance with the configured values.
    pub fn build(self) -> SubgraphResponse {
        SubgraphResponse {
            connection_time: self.connection_time,
            response_time: self.response_time,
            status: self.status,
            headers: self.headers,
        }
    }

    /// Sets the HTTP headers for the response.
    ///
    /// This captures the headers returned by the subgraph in the HTTP response.
    ///
    /// # Arguments
    ///
    /// * `headers` - The HTTP headers from the response
    pub fn headers(&mut self, headers: http::HeaderMap) {
        self.headers = headers;
    }

    /// Records the connection establishment time.
    ///
    /// This should be called when the connection to the subgraph is established.
    /// It captures the elapsed time since the request started.
    pub fn track_connection(&mut self) {
        self.connection_time = self.start_time.elapsed();
    }

    /// Records the response completion time.
    ///
    /// This should be called when the complete response is received from the subgraph.
    /// It captures the elapsed time since the request started.
    pub fn track_response(&mut self) {
        self.response_time = self.start_time.elapsed();
    }
}

/// Builder for constructing [`ExecutedHttpRequest`] instances.
pub struct ExecutedHttpRequestBuilder<'a> {
    pub(super) url: &'a str,
    pub(super) method: http::Method,
    pub(super) response_status: http::StatusCode,
}

impl<'a> ExecutedHttpRequestBuilder<'a> {
    /// Sets the HTTP method for the request.
    ///
    /// Defaults to POST if not specified.
    ///
    /// # Arguments
    ///
    /// * `method` - The HTTP method (GET, POST, etc.)
    pub fn method(mut self, method: http::Method) -> Self {
        self.method = method;
        self
    }

    /// Sets the HTTP response status code.
    ///
    /// Defaults to 200 OK if not specified.
    ///
    /// # Arguments
    ///
    /// * `status` - The HTTP status code received
    pub fn response_status(mut self, status: http::StatusCode) -> Self {
        self.response_status = status;
        self
    }

    /// Consumes the builder and creates an [`ExecutedHttpRequest`].
    ///
    /// # Returns
    ///
    /// A new `ExecutedHttpRequest` instance with the configured values.
    pub fn build(self) -> ExecutedHttpRequest {
        ExecutedHttpRequest {
            url: self.url.to_string(),
            method: self.method,
            response_status: self.response_status,
        }
    }
}
