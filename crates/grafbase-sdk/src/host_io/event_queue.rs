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

use crate::{SdkError, wit};

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
            wit::Event::Extension(items) => Self::Extension(ExtensionEvent(items)),
        }
    }
}

/// Represents an executed GraphQL operation with detailed metrics.
pub struct ExecutedOperation(wit::ExecutedOperation);

impl From<wit::ExecutedOperation> for ExecutedOperation {
    fn from(value: wit::ExecutedOperation) -> Self {
        Self(value)
    }
}

impl ExecutedOperation {
    /// Returns the name of the GraphQL operation, if available.
    pub fn name(&self) -> Option<&str> {
        self.0.name.as_deref()
    }

    /// Returns the GraphQL document (query/mutation/subscription) that was executed.
    /// The operation is in normalized form, with all possible user data removed.
    pub fn document(&self) -> &str {
        &self.0.document
    }

    /// Returns the duration spent preparing the operation for execution.
    ///
    /// This includes parsing, validation, and query planning time.
    pub fn prepare_duration(&self) -> Duration {
        Duration::from_nanos(self.0.prepare_duration_ns)
    }

    /// Returns the total duration of the operation execution.
    ///
    /// This includes the actual execution time after preparation.
    pub fn duration(&self) -> Duration {
        Duration::from_nanos(self.0.duration_ns)
    }

    /// Indicates whether a cached execution plan was used for this operation.
    pub fn cached_plan(&self) -> bool {
        self.0.cached_plan
    }

    /// Returns the status of the GraphQL response.
    pub fn status(&self) -> GraphqlResponseStatus {
        self.0.status.into()
    }
}

/// Represents the status of a GraphQL response.
///
/// This enum categorizes the different outcomes of a GraphQL operation execution.
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
pub struct FieldError {
    /// The number of field errors encountered.
    pub count: u64,
    /// Indicates whether the data field in the response is null.
    pub data_is_null: bool,
}

/// Contains information about request-level errors in a GraphQL response.
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
pub struct ExecutedSubgraphRequest(wit::ExecutedSubgraphRequest);

impl ExecutedSubgraphRequest {
    /// Returns the name of the subgraph that was queried.
    pub fn subgraph_name(&self) -> &str {
        &self.0.subgraph_name
    }

    /// Returns the HTTP method used for the subgraph request.
    pub fn method(&self) -> http::Method {
        self.0.method.into()
    }

    /// Returns the URL of the subgraph endpoint.
    pub fn url(&self) -> &str {
        &self.0.url
    }

    /// Returns an iterator over all execution attempts for this subgraph request.
    ///
    /// This includes both successful responses and various types of failures
    /// (e.g., rate limiting, server errors).
    pub fn executions(&self) -> impl Iterator<Item = RequestExecution> {
        self.0.executions.clone().into_iter().map(RequestExecution::from)
    }

    /// Returns the cache status for this subgraph request.
    pub fn cache_status(&self) -> CacheStatus {
        self.0.cache_status.into()
    }

    /// Returns the total duration of all execution attempts.
    pub fn total_duration(&self) -> Duration {
        Duration::from_nanos(self.0.total_duration_ns)
    }

    /// Indicates whether any errors were encountered during the subgraph request.
    pub fn has_errors(&self) -> bool {
        self.0.has_errors
    }
}

impl From<wit::ExecutedSubgraphRequest> for ExecutedSubgraphRequest {
    fn from(value: wit::ExecutedSubgraphRequest) -> Self {
        Self(value)
    }
}

/// Represents a single execution attempt of a subgraph request.
///
/// This enum captures the different outcomes of attempting to execute a request
/// to a subgraph endpoint.
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
            wit::SubgraphRequestExecutionKind::Response(subgraph_response) => {
                Self::Response(SubgraphResponse(subgraph_response))
            }
        }
    }
}

/// Contains timing and status information for a successful subgraph response.
pub struct SubgraphResponse(wit::SubgraphResponse);

impl SubgraphResponse {
    /// Returns the time taken to establish a connection to the subgraph.
    pub fn connection_time(&self) -> Duration {
        Duration::from_nanos(self.0.connection_time_ns)
    }

    /// Returns the time taken to receive the complete response from the subgraph.
    pub fn response_time(&self) -> Duration {
        Duration::from_nanos(self.0.response_time_ns)
    }

    /// Returns the HTTP status code of the subgraph response.
    pub fn status(&self) -> http::StatusCode {
        http::StatusCode::from_u16(self.0.status_code).expect("this comes from reqwest")
    }
}

/// Represents the cache status of a subgraph request.
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

/// Represents an HTTP request that was executed.
///
/// This struct contains information about non-GraphQL HTTP requests made by the system.
pub struct ExecutedHttpRequest(wit::ExecutedHttpRequest);

impl ExecutedHttpRequest {
    /// Returns the HTTP method used for the request.
    ///
    /// # Returns
    ///
    /// An `http::Method` representing the HTTP method (GET, POST, etc.).
    pub fn method(&self) -> http::Method {
        self.0.method.into()
    }

    /// Returns the URL of the HTTP request.
    ///
    /// # Returns
    ///
    /// The full URL as a string slice.
    pub fn url(&self) -> &str {
        &self.0.url
    }

    /// Returns the HTTP status code of the response.
    ///
    /// # Returns
    ///
    /// An `http::StatusCode` representing the response status.
    pub fn response_status(&self) -> http::StatusCode {
        http::StatusCode::from_u16(self.0.status_code).expect("this comes from engine")
    }
}

impl From<wit::ExecutedHttpRequest> for ExecutedHttpRequest {
    fn from(value: wit::ExecutedHttpRequest) -> Self {
        Self(value)
    }
}

/// Represents a custom extension log entry with serialized data.
///
/// Extension logs allow custom data to be included in the event queue stream.
/// The data is serialized using CBOR format and can be deserialized into
/// the appropriate type.
pub struct ExtensionEvent(wit::ExtensionEvent);

impl ExtensionEvent {
    /// Event name
    pub fn event_name(&self) -> &str {
        &self.0.event_name
    }

    /// Extension name which produced this event
    pub fn extension_name(&self) -> &str {
        &self.0.extension_name
    }

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
        let data = minicbor_serde::from_slice(&self.0.data)?;

        Ok(data)
    }
}
