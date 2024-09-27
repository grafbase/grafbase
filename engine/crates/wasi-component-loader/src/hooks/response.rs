use wasmtime::component::{ComponentType, Lower};

use super::{component_instance, ComponentInstance};
use crate::{
    context::SharedContext,
    names::{
        ON_HTTP_RESPONSE_FUNCTION, ON_OPERATION_RESPONSE_FUNCTION, ON_SUBGRAPH_RESPONSE_FUNCTION, RESPONSES_INTERFACE,
    },
    ComponentLoader,
};

component_instance!(ResponsesComponentInstance: RESPONSES_INTERFACE);

/// Data from an executed HTTP request.
#[derive(Debug, Clone, Lower, ComponentType)]
#[component(record)]
pub struct ExecutedHttpRequest {
    /// The request method.
    #[component(name = "method")]
    pub method: String,
    /// The request URL.
    #[component(name = "url")]
    pub url: String,
    /// The response status code.
    #[component(name = "status-code")]
    pub status_code: u16,
    /// Results from on-gateway-response hooks.
    #[component(name = "on-operation-response-outputs")]
    pub on_operation_response_outputs: Vec<Vec<u8>>,
}

/// Error from fetching a subgraph field.
#[derive(Debug, Clone, Copy, Lower, ComponentType)]
#[component(record)]
pub struct FieldError {
    /// Number of errors.
    #[component(name = "count")]
    pub count: u64,
    /// The returned data was null.
    #[component(name = "data-is-null")]
    pub data_is_null: bool,
}

/// Error requesting a subgraph.
#[derive(Debug, Clone, Copy, Lower, ComponentType)]
#[component(record)]
pub struct RequestError {
    /// Number of errors.
    #[component(name = "count")]
    pub count: u64,
}

/// A status of a subgraph response.
#[derive(Debug, Clone, Copy, Lower, ComponentType)]
#[component(variant)]
pub enum GraphqlResponseStatus {
    /// No errors.
    #[component(name = "success")]
    Success,
    /// Error in fetching a field.
    #[component(name = "field-error")]
    FieldError(FieldError),
    /// Error executing a request.
    #[component(name = "request-error")]
    RequestError(RequestError),
    /// Refused fetching subgraph.
    #[component(name = "refused-request")]
    RefusedRequest,
}

/// Data from an executed full operation.
#[derive(Debug, Clone, Lower, ComponentType)]
#[component(record)]
pub struct ExecutedOperation {
    /// The name of the operation, if present.
    #[component(name = "name")]
    pub name: Option<String>,
    /// The sanitized query document.
    #[component(name = "document")]
    pub document: String,
    /// The duration taken by operation preparation in milliseconds.
    #[component(name = "prepare-duration-ms")]
    pub prepare_duration_ms: u64,
    /// If the operation plan was taken from cache.
    #[component(name = "cached-plan")]
    pub cached_plan: bool,
    /// The duration it took to execute the operation.
    #[component(name = "duration-ms")]
    pub duration_ms: u64,
    /// The status of the operation.
    #[component(name = "status")]
    pub status: GraphqlResponseStatus,
    /// The outputs of on-subgraph-response hooks.
    #[component(name = "on-subgraph-response-outputs")]
    pub on_subgraph_response_outputs: Vec<Vec<u8>>,
}

/// A response info from an executed subgraph request.
#[derive(Debug, Clone, Copy, Lower, ComponentType)]
#[component(record)]
pub struct SubgraphResponse {
    /// Time it took to connect to the subgraph endpoint, in milliseconds.
    #[component(name = "connection-time-ms")]
    pub connection_time_ms: u64,
    /// Time it took to fetch the response from the subgraph, in milliseconds.
    #[component(name = "response-time-ms")]
    pub response_time_ms: u64,
    /// The response status code from subgraph.
    #[component(name = "status-code")]
    pub status_code: u16,
}

/// The subgraph cache status.
#[derive(Debug, Clone, Lower, ComponentType)]
#[component(enum)]
pub enum CacheStatus {
    /// Everything was taken from cache.
    #[component(name = "hit")]
    Hit,
    /// Parts of the data was taken from cache.
    #[component(name = "partial-hit")]
    PartialHit,
    /// No data was taken from cache.
    #[component(name = "miss")]
    Miss,
}

/// Response data from a subgraph request.
#[derive(Debug, Clone, Copy, Lower, ComponentType)]
#[component(variant)]
pub enum SubgraphRequestExecutionKind {
    /// Internal error in the host.
    #[component(name = "internal-server-error")]
    InternalServerError,
    /// Response prevented by a hook.
    #[component(name = "hook-error")]
    HookError,
    /// Request failed.
    #[component(name = "request-error")]
    RequestError,
    /// Request was rate-limited.
    #[component(name = "rate-limited")]
    RateLimited,
    /// A response was received.
    #[component(name = "response")]
    Response(SubgraphResponse),
}

/// A response info from subgraph fetch.
#[derive(Debug, Clone, Lower, ComponentType)]
#[component(record)]
pub struct ExecutedSubgraphRequest {
    /// The name of the subgraph.
    #[component(name = "subgraph-name")]
    pub subgraph_name: String,
    /// The HTTP method used in the request.
    #[component(name = "method")]
    pub method: String,
    /// The URL of the subgraph.
    #[component(name = "url")]
    pub url: String,
    /// The subgraph executions(s).
    #[component(name = "executions")]
    pub executions: Vec<SubgraphRequestExecutionKind>,
    /// If anything in the request was cached.
    #[component(name = "cache-status")]
    pub cache_status: CacheStatus,
    /// Total time taken to get a response, retries included. In milliseconds.
    #[component(name = "total-duration-ms")]
    pub total_duration_ms: u64,
    /// True, if the response has any GraphQL errors.
    #[component(name = "has-errors")]
    pub has_errors: bool,
}

impl ResponsesComponentInstance {
    /// Allows inspection of the response from a subgraph request.
    ///
    /// # Arguments
    ///
    /// * `context` - A shared context for the operation.
    /// * `request` - The executed subgraph request containing details of the request.
    ///
    /// # Returns
    ///
    /// A `Result` containing a serialized vector of bytes from the user on success,
    /// or an error on failure.
    pub async fn on_subgraph_response(
        &mut self,
        context: SharedContext,
        request: ExecutedSubgraphRequest,
    ) -> crate::Result<Vec<u8>> {
        let span = tracing::info_span!(ON_SUBGRAPH_RESPONSE_FUNCTION);

        self.call1_one_output(span, ON_SUBGRAPH_RESPONSE_FUNCTION, context, request)
            .await?
            .map(|result: Vec<u8>| Ok(result))
            .unwrap_or_else(|| Ok(Vec::new()))
    }

    /// Allows inspection of the response from an executed operation.
    ///
    /// # Arguments
    ///
    /// * `context` - A shared context for the operation.
    /// * `request` - The executed operation containing details of the operation.
    ///
    /// # Returns
    ///
    /// A `Result` containing a serialized vector of bytes from the user on success,
    /// or an error on failure.
    pub async fn on_operation_response(
        &mut self,
        context: SharedContext,
        request: ExecutedOperation,
    ) -> crate::Result<Vec<u8>> {
        let span = tracing::info_span!(ON_OPERATION_RESPONSE_FUNCTION);

        self.call1_one_output(span, ON_OPERATION_RESPONSE_FUNCTION, context, request)
            .await?
            .map(|result: Vec<u8>| Ok(result))
            .unwrap_or_else(|| Ok(Vec::new()))
    }

    /// Allows inspection of the response from an executed HTTP request.
    ///
    /// # Arguments
    ///
    /// * `context` - A shared context for the operation.
    /// * `request` - The executed HTTP request containing details of the request.
    pub async fn on_http_response(
        &mut self,
        context: SharedContext,
        request: ExecutedHttpRequest,
    ) -> crate::Result<()> {
        let span = tracing::info_span!(ON_HTTP_RESPONSE_FUNCTION);

        self.call1_without_output(span, ON_HTTP_RESPONSE_FUNCTION, context, request)
            .await
    }
}
