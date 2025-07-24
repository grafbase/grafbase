//! Event queuing functionality for tracking and recording request events.
//!
//! This module provides comprehensive event queuing capabilities for tracking various
//! types of operations and requests within the Grafbase Gateway system. It supports tracking
//! of GraphQL operations, subgraph requests, HTTP requests, and custom extension logs.
//!
//! # Overview
//!
//! The event queue system is designed to capture detailed information about:
//! - GraphQL operation execution (including timing, caching, and status)
//! - Subgraph request details (including retries, caching, and response times)
//! - HTTP request execution
//! - Custom extension logs with serializable data
//!
//! # Example
//!
//! ```no_run
//! use grafbase_sdk::host_io::event_queue;
//! use serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct CustomEvent {
//!     user_id: String,
//!     action: String,
//!     timestamp: u64,
//! }
//!
//! // Send a custom event
//! let log = CustomEvent {
//!     user_id: "user123".to_string(),
//!     action: "query_execution".to_string(),
//!     timestamp: 1234567890,
//! };
//!
//! event_queue::send("custom_event", log).expect("Failed to send event");
//! ```
//!
//! # Log Aggregation
//!
//! By itself, event queue calls do nothing in the Grafbase Gateway. You must implement
//! an [`Hosts`] type of an extension with event filtering, which will be called after
//! a response is sent back to the user.

use std::time::Duration;

use crate::{SdkError, types::Headers, wit};

/// Sends an event queue entry to the system.
///
/// This function serializes the provided log data and sends it to the event queue
/// system. The log data can be any type that implements `serde::Serialize`.
///
/// # Arguments
///
/// * `name` - The name of the event to be logged. Used in event filtering.
/// * `data` - The log data to be sent. Must implement `serde::Serialize`.
///
/// # Returns
///
/// Returns `Ok(())` on success, or an `SdkError` if serialization fails.
///
/// # Example
///
/// ```no_run
/// use serde::Serialize;
/// use grafbase_sdk::host_io::event_queue;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// #[derive(Serialize)]
/// struct UserAction<'a> {
///     action: &'a str,
///     user_id: &'a str,
/// }
///
/// let action = UserAction {
///     action: "login",
///     user_id: "user123"
/// };
///
/// event_queue::send("user_action", action)?;
/// # Ok(())
/// # }
/// ```
pub fn send<T>(name: &str, data: T) -> Result<(), SdkError>
where
    T: serde::Serialize,
{
    if !crate::component::can_skip_sending_events() {
        let data = minicbor_serde::to_vec(data)?;
        crate::component::queue_event(name, &data);
    }

    Ok(())
}

/// A queue of event queue per request from the engine.
///
/// This struct provides access to event queue that have been generated during
/// request processing. Logs can be retrieved and processed using the `pop` method.
pub struct EventQueue(wit::EventQueue);

impl From<wit::EventQueue> for EventQueue {
    fn from(value: wit::EventQueue) -> Self {
        Self(value)
    }
}

impl EventQueue {
    /// Retrieves and removes the next log entry from the queue.
    pub fn pop(&self) -> Option<Event> {
        self.0.pop().map(Into::into)
    }
}

/// Represents different types of event queue entries.
///
/// This enum categorizes the various types of operations and requests that can be
/// logged in the event queue system.
#[non_exhaustive]
pub enum Event {
    /// A GraphQL operation that was executed.
    Operation(ExecutedOperation),
    /// A request made to a subgraph.
    Subgraph(ExecutedSubgraphRequest),
    /// An HTTP request that was executed.
    Http(ExecutedHttpRequest),
    /// A custom extension log entry with serialized data.
    Extension(ExtensionEvent),
}

impl From<wit::Event> for Event {
    fn from(value: wit::Event) -> Self {
        match value {
            wit::Event::Operation(executed_operation) => Self::Operation(executed_operation.into()),
            wit::Event::Subgraph(executed_subgraph_request) => Self::Subgraph(executed_subgraph_request.into()),
            wit::Event::Http(executed_http_request) => Self::Http(executed_http_request.into()),
            wit::Event::Extension(event) => Self::Extension(event.into()),
        }
    }
}

/// Represents an executed GraphQL operation with detailed metrics.
#[non_exhaustive]
pub struct ExecutedOperation {
    /// The name of the GraphQL operation, if available.
    pub name: Option<String>,
    /// The GraphQL document (query/mutation/subscription) that was executed.
    /// The operation is in normalized form, with all possible user data removed.
    pub document: String,
    /// The duration spent preparing the operation for execution.
    /// This includes parsing, validation, and query planning time.
    pub prepare_duration: Duration,
    /// The total duration of the operation execution.
    /// This includes the actual execution time and the preparation.
    pub duration: Duration,
    /// Indicates whether a cached execution plan was used for this operation.
    pub cached_plan: bool,
    /// The status of the GraphQL response.
    pub status: GraphqlResponseStatus,
    /// The type of GraphQL operation that was executed.
    pub operation_type: OperationType,
    /// The complexity represents the computational cost of executing the operation.
    /// Read more: <https://grafbase.com/docs/gateway/configuration/complexity-control>
    pub complexity: Option<u64>,
    /// Indicates whether the operation used any deprecated fields.
    pub has_deprecated_fields: bool,
}

impl From<wit::ExecutedOperation> for ExecutedOperation {
    fn from(value: wit::ExecutedOperation) -> Self {
        ExecutedOperation {
            name: value.name,
            document: value.document,
            prepare_duration: Duration::from_nanos(value.prepare_duration_ns),
            duration: Duration::from_nanos(value.duration_ns),
            cached_plan: value.cached_plan,
            status: value.status.into(),
            operation_type: value.operation_type.into(),
            complexity: value.complexity,
            has_deprecated_fields: value.has_deprecated_fields,
        }
    }
}

/// Represents the type of GraphQL operation.
///
/// This enum categorizes the different types of GraphQL operations
/// that can be executed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationType {
    /// A GraphQL query operation for reading data.
    Query,
    /// A GraphQL mutation operation for modifying data.
    Mutation,
    /// A GraphQL subscription operation for real-time data streaming.
    Subscription,
}

impl From<wit::OperationType> for OperationType {
    fn from(value: wit::OperationType) -> Self {
        match value {
            wit::OperationType::Query => OperationType::Query,
            wit::OperationType::Mutation => OperationType::Mutation,
            wit::OperationType::Subscription => OperationType::Subscription,
        }
    }
}

/// Represents the status of a GraphQL response.
///
/// This enum categorizes the different outcomes of a GraphQL operation execution.
#[derive(serde::Serialize, Debug, Clone)]
pub enum GraphqlResponseStatus {
    /// The operation completed successfully without errors.
    Success,
    /// The operation completed but encountered field-level errors.
    FieldError(FieldError),
    /// The operation failed due to request-level errors.
    RequestError(RequestError),
    /// The request was refused before execution (e.g., due to authentication or rate limiting).
    RefusedRequest,
}

/// Contains information about field-level errors in a GraphQL response.
#[derive(serde::Serialize, Debug, Clone)]
#[non_exhaustive]
pub struct FieldError {
    /// The number of field errors encountered.
    pub count: u64,
    /// Indicates whether the data field in the response is null.
    pub data_is_null: bool,
}

/// Contains information about request-level errors in a GraphQL response.
#[derive(serde::Serialize, Debug, Clone)]
#[non_exhaustive]
pub struct RequestError {
    /// The number of request errors encountered.
    pub count: u64,
}

impl From<wit::GraphqlResponseStatus> for GraphqlResponseStatus {
    fn from(value: wit::GraphqlResponseStatus) -> Self {
        match value {
            wit::GraphqlResponseStatus::Success => GraphqlResponseStatus::Success,
            wit::GraphqlResponseStatus::FieldError(wit::FieldError { count, data_is_null }) => {
                GraphqlResponseStatus::FieldError(FieldError { count, data_is_null })
            }
            wit::GraphqlResponseStatus::RequestError(wit::RequestError { count }) => {
                GraphqlResponseStatus::RequestError(RequestError { count })
            }
            wit::GraphqlResponseStatus::RefusedRequest => GraphqlResponseStatus::RefusedRequest,
        }
    }
}

/// Represents a request made to a subgraph with detailed execution information.
///
/// This struct contains comprehensive information about a subgraph request,
/// including retry attempts, caching status, and timing metrics.
#[non_exhaustive]
pub struct ExecutedSubgraphRequest {
    /// The name of the subgraph that was queried.
    pub subgraph_name: String,
    /// The HTTP method used for the subgraph request (e.g., GET, POST).
    pub method: http::Method,
    /// The URL of the subgraph endpoint that was queried.
    pub url: String,
    /// The cache status of the subgraph request.
    pub cache_status: CacheStatus,
    /// The total duration of all execution attempts for this subgraph request.
    pub total_duration: Duration,
    /// Indicates whether any errors were encountered during the subgraph request.
    pub has_errors: bool,
    executions: Vec<wit::SubgraphRequestExecutionKind>,
}

impl ExecutedSubgraphRequest {
    /// Returns an iterator over all execution attempts for this subgraph request.
    ///
    /// This includes both successful responses and various types of failures
    /// (e.g., rate limiting, server errors).
    pub fn into_executions(self) -> impl Iterator<Item = RequestExecution> {
        self.executions.into_iter().map(RequestExecution::from)
    }
}

impl From<wit::ExecutedSubgraphRequest> for ExecutedSubgraphRequest {
    fn from(value: wit::ExecutedSubgraphRequest) -> Self {
        Self {
            subgraph_name: value.subgraph_name,
            method: value.method.into(),
            url: value.url,
            cache_status: value.cache_status.into(),
            total_duration: Duration::from_nanos(value.total_duration_ns),
            has_errors: value.has_errors,
            executions: value.executions,
        }
    }
}

/// Represents a single execution attempt of a subgraph request.
///
/// This enum captures the different outcomes of attempting to execute a request
/// to a subgraph endpoint.
#[non_exhaustive]
pub enum RequestExecution {
    /// The subgraph returned a 5xx server error.
    InternalServerError,
    /// The request failed due to network or other request-level errors.
    RequestError,
    /// The request was rate limited by the engine rate limiter.
    RateLimited,
    /// The subgraph returned a response (which may still contain GraphQL errors).
    Response(SubgraphResponse),
}

impl From<wit::SubgraphRequestExecutionKind> for RequestExecution {
    fn from(value: wit::SubgraphRequestExecutionKind) -> Self {
        match value {
            wit::SubgraphRequestExecutionKind::InternalServerError => Self::InternalServerError,
            wit::SubgraphRequestExecutionKind::RequestError => Self::RequestError,
            wit::SubgraphRequestExecutionKind::RateLimited => Self::RateLimited,
            wit::SubgraphRequestExecutionKind::Response(subgraph_response) => Self::Response(subgraph_response.into()),
        }
    }
}

/// Contains timing and status information for a successful subgraph response.
#[non_exhaustive]
pub struct SubgraphResponse {
    /// The time taken to establish a connection to the subgraph.
    pub connection_time: Duration,
    /// The time taken to receive the complete response from the subgraph.
    pub response_time: Duration,
    /// The HTTP status code of the subgraph response.
    pub status_code: http::StatusCode,
    /// The HTTP response headers from the subgraph.
    pub response_headers: Headers,
}

impl From<wit::SubgraphResponse> for SubgraphResponse {
    fn from(value: wit::SubgraphResponse) -> Self {
        Self {
            connection_time: Duration::from_nanos(value.connection_time_ns),
            response_time: Duration::from_nanos(value.response_time_ns),
            status_code: http::StatusCode::from_u16(value.status_code).expect("Gateway provides a valid status code"),
            response_headers: value.response_headers.into(),
        }
    }
}

/// Represents the cache status of a subgraph request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CacheStatus {
    /// The entire response was served from cache.
    Hit,
    /// Part of the response was served from cache, but some data required fetching.
    PartialHit,
    /// No cached data was available; the entire response was fetched from the subgraph.
    Miss,
}

impl From<wit::CacheStatus> for CacheStatus {
    fn from(value: wit::CacheStatus) -> Self {
        match value {
            wit::CacheStatus::Hit => Self::Hit,
            wit::CacheStatus::PartialHit => Self::PartialHit,
            wit::CacheStatus::Miss => Self::Miss,
        }
    }
}

impl CacheStatus {
    /// Returns the cache status as a string slice.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Hit => "hit",
            Self::PartialHit => "partial_hit",
            Self::Miss => "miss",
        }
    }
}

impl AsRef<str> for CacheStatus {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// Represents an HTTP request that was executed.
///
/// This struct contains information about non-GraphQL HTTP requests made by the system.
#[non_exhaustive]
pub struct ExecutedHttpRequest {
    /// An `http::StatusCode` representing the response status.
    pub status_code: http::StatusCode,
    /// The HTTP method used for the request.
    pub method: http::Method,
    /// The full URL as a string slice.
    pub url: String,
}

impl From<wit::ExecutedHttpRequest> for ExecutedHttpRequest {
    fn from(value: wit::ExecutedHttpRequest) -> Self {
        Self {
            status_code: http::StatusCode::from_u16(value.status_code).expect("Gateway provides a valid status code"),
            method: value.method.into(),
            url: value.url,
        }
    }
}

/// Represents a custom extension log entry with serialized data.
///
/// Extension logs allow custom data to be included in the event queue stream.
/// The data is serialized using CBOR format and can be deserialized into
/// the appropriate type.
#[non_exhaustive]
pub struct ExtensionEvent {
    /// Event name
    pub event_name: String,
    /// Extension name which produced this event
    pub extension_name: String,
    data: Vec<u8>,
}

impl ExtensionEvent {
    /// Deserializes the extension log data into the specified type.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type to deserialize into. Must implement `serde::Deserialize`.
    ///
    /// # Returns
    ///
    /// Returns `Ok(T)` with the deserialized data on success, or an `SdkError` if
    /// deserialization fails.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use serde::Deserialize;
    /// use grafbase_sdk::host_io::event_queue::Event;
    ///
    /// #[derive(Deserialize)]
    /// struct CustomLog {
    ///     user_id: String,
    ///     action: String,
    /// }
    ///
    /// // Assuming we have an ExtensionLogEntry
    /// let log_entry: ExtensionEvent = // ... obtained from elsewhere
    /// # todo!();
    ///
    /// match log_entry.deserialize::<CustomLog>() {
    ///     Ok(custom_log) => {
    ///         println!("User {} performed: {}", custom_log.user_id, custom_log.action);
    ///     }
    ///     Err(e) => {
    ///         eprintln!("Failed to deserialize log: {:?}", e);
    ///     }
    /// }
    /// ```
    pub fn deserialize<'de, T>(&'de self) -> Result<T, SdkError>
    where
        T: serde::Deserialize<'de>,
    {
        let data = minicbor_serde::from_slice(&self.data)?;

        Ok(data)
    }
}

impl From<wit::ExtensionEvent> for ExtensionEvent {
    fn from(value: wit::ExtensionEvent) -> Self {
        Self {
            event_name: value.event_name,
            extension_name: value.extension_name,
            data: value.data,
        }
    }
}
