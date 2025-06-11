use std::{sync::Arc, time::Duration};

use grafbase_telemetry::graphql::GraphqlResponseStatus;

use super::{
    CacheStatus, ExecutedHttpRequest, ExecutedOperation, ExecutedSubgraphRequest, ExtensionEvent, RequestExecution,
    SubgraphResponse,
};

/// Builder for constructing [`ExecutedOperation`] instances.
pub struct ExecutedOperationBuilder<'a> {
    pub(super) name: Option<&'a str>,
    pub(super) document: Arc<str>,
    pub(super) prepare_duration: Duration,
    pub(super) duration: Duration,
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
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = Some(name);
        self
    }

    /// Sets the time spent preparing the operation.
    ///
    /// This includes parsing, validation, and query planning time.
    ///
    /// # Arguments
    ///
    /// * `duration` - The preparation duration
    pub fn prepare_duration(mut self, duration: Duration) -> Self {
        self.prepare_duration = duration;
        self
    }

    /// Sets the total execution duration for the operation.
    ///
    /// This is the end-to-end time from receiving the request to sending the response.
    ///
    /// # Arguments
    ///
    /// * `duration` - The total execution duration
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Sets whether a cached query plan was used.
    ///
    /// Query plan caching can significantly improve performance for repeated queries.
    ///
    /// # Arguments
    ///
    /// * `cached` - `true` if a cached plan was used, `false` otherwise
    pub fn cached_plan(mut self, cached: bool) -> Self {
        self.cached_plan = cached;
        self
    }

    /// Sets the response status for the operation.
    ///
    /// This indicates whether the operation completed successfully, had field errors,
    /// or encountered request-level errors.
    ///
    /// # Arguments
    ///
    /// * `status` - The GraphQL response status
    pub fn status(mut self, status: GraphqlResponseStatus) -> Self {
        self.status = status;
        self
    }

    /// Consumes the builder and creates an [`ExecutedOperation`].
    pub fn build(self) -> ExecutedOperation {
        ExecutedOperation {
            name: self.name.map(|s| s.to_string()),
            document: self.document,
            prepare_duration: self.prepare_duration,
            duration: self.duration,
            cached_plan: self.cached_plan,
            status: self.status,
        }
    }
}

/// Builder for constructing [`ExecutedSubgraphRequest`] instances.
pub struct ExecutedSubgraphRequestBuilder<'a> {
    pub(super) subgraph_name: &'a str,
    pub(super) method: http::Method,
    pub(super) url: &'a str,
    pub(super) executions: Vec<RequestExecution>,
    pub(super) cache_status: CacheStatus,
    pub(super) total_duration: Duration,
    pub(super) has_errors: bool,
}

impl<'a> ExecutedSubgraphRequestBuilder<'a> {
    /// Sets the list of execution attempts for this subgraph request.
    ///
    /// This includes all retry attempts and their outcomes.
    ///
    /// # Arguments
    ///
    /// * `executions` - Slice of execution attempts
    pub fn push_execution(mut self, execution: RequestExecution) -> Self {
        self.executions.push(execution);
        self
    }

    /// Sets the cache status for this request.
    ///
    /// Indicates whether the response was served from cache.
    ///
    /// # Arguments
    ///
    /// * `status` - The cache status (hit, partial hit, or miss)
    pub fn cache_status(mut self, status: CacheStatus) -> Self {
        self.cache_status = status;
        self
    }

    /// Sets the total duration for all execution attempts.
    ///
    /// This includes time spent on retries if any occurred.
    ///
    /// # Arguments
    ///
    /// * `duration` - The total duration across all attempts
    pub fn total_duration(mut self, duration: Duration) -> Self {
        self.total_duration = duration;
        self
    }

    /// Sets whether any errors occurred during execution.
    ///
    /// This includes both network errors and GraphQL errors in the response.
    ///
    /// # Arguments
    ///
    /// * `has_errors` - `true` if any errors occurred
    pub fn has_errors(mut self, has_errors: bool) -> Self {
        self.has_errors = has_errors;
        self
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
}

/// Builder for constructing [`SubgraphResponse`] instances.
pub struct SubgraphResponseBuilder {
    pub(super) connection_time: Duration,
    pub(super) response_time: Duration,
    pub(super) status: http::StatusCode,
    pub(super) headers: http::HeaderMap,
}

impl SubgraphResponseBuilder {
    /// Sets the time taken to establish the connection.
    ///
    /// This helps identify network latency issues.
    ///
    /// # Arguments
    ///
    /// * `duration` - The connection establishment time
    pub fn connection_time(mut self, duration: Duration) -> Self {
        self.connection_time = duration;
        self
    }

    /// Sets the time from request sent to response received.
    ///
    /// This measures the subgraph's processing time.
    ///
    /// # Arguments
    ///
    /// * `duration` - The response generation time
    pub fn response_time(mut self, duration: Duration) -> Self {
        self.response_time = duration;
        self
    }

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

/// Builder for constructing [`ExtensionEvent`] instances.
pub struct ExtensionEventBuilder<'a> {
    pub(super) extension_name: &'a str,
    pub(super) event_name: &'a str,
    pub(super) data: Vec<u8>,
}

impl<'a> ExtensionEventBuilder<'a> {
    /// Sets the binary data payload for the event.
    ///
    /// Extensions can use this to attach arbitrary data to their events.
    ///
    /// # Arguments
    ///
    /// * `data` - The binary data to attach to the event
    pub fn data(mut self, data: Vec<u8>) -> Self {
        self.data = data;
        self
    }

    /// Consumes the builder and creates an [`ExtensionEvent`].
    ///
    /// # Returns
    ///
    /// A new `ExtensionEvent` instance with the configured values.
    pub fn build(self) -> ExtensionEvent {
        ExtensionEvent {
            extension_name: self.extension_name.to_string(),
            event_name: self.event_name.to_string(),
            data: self.data,
        }
    }
}
