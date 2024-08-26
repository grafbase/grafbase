use wasmtime::component::{ComponentType, Lower};

use super::{component_instance, ComponentInstance};
use crate::{
    context::SharedContext,
    names::{
        ON_GATEWAY_RESPONSE_FUNCTION, ON_HTTP_RESPONSE_FUNCTION, ON_SUBGRAPH_RESPONSE_FUNCTION, RESPONSES_INTERFACE,
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
    #[component(name = "on-gateway-response-outputs")]
    pub on_gateway_response_outputs: Vec<Vec<u8>>,
}

/// Information on a GraphQL operation.
#[derive(Debug, Clone, Lower, ComponentType)]
#[component(record)]
pub struct Operation {
    /// The name of the operation, if present.
    #[component(name = "name")]
    pub name: Option<String>,
    /// The sanitized query document.
    #[component(name = "document")]
    pub document: String,
    /// The duration taken by operation preparation in milliseconds.
    #[component(name = "prepare-duration")]
    pub prepare_duration: u64,
    /// If the operation plan was taken from cache.
    #[component(name = "cached")]
    pub cached: bool,
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
pub struct ExecutedGatewayRequest {
    /// The duration it took to execute the operation.
    #[component(name = "duration")]
    pub duration: u64,
    /// The status of the operation.
    #[component(name = "status")]
    pub status: GraphqlResponseStatus,
    /// The outputs of on-subgraph-request hooks.
    #[component(name = "on-subgraph-request-outputs")]
    pub on_subgraph_request_outputs: Vec<Vec<u8>>,
}

/// A response info from an executed subgraph request.
#[derive(Debug, Clone, Copy, Lower, ComponentType)]
#[component(record)]
pub struct SubgraphResponseInfo {
    /// Time it took to connect to the subgraph endpoint, in milliseconds.
    #[component(name = "connection-time")]
    pub connection_time: u64,
    /// Time it took to fetch the response from the subgraph, in milliseconds.
    #[component(name = "response-time")]
    pub response_time: u64,
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
    /// The subgraph response(s).
    pub response_infos: Vec<SubgraphResponseInfo>,
    /// If anything in the request was cached.
    pub cache_status: CacheStatus,
    /// Total time taken to get a response, retries included. In milliseconds.
    #[component(name = "total-duration")]
    pub total_duration: u64,
    /// True, if the response has any GraphQL errors.
    #[component(name = "has-errors")]
    pub has_errors: bool,
}

impl ResponsesComponentInstance {
    /// Called right after a subgraph request.
    pub async fn on_subgraph_response(
        &mut self,
        context: SharedContext,
        request: ExecutedSubgraphRequest,
    ) -> crate::Result<Vec<u8>> {
        self.call1(ON_SUBGRAPH_RESPONSE_FUNCTION, context, request)
            .await?
            .map(|result: Vec<u8>| Ok(result))
            .unwrap_or_else(|| Ok(Vec::new()))
    }

    /// Called right after a gateway request.
    pub async fn on_gateway_response(
        &mut self,
        context: SharedContext,
        operation: Operation,
        request: ExecutedGatewayRequest,
    ) -> crate::Result<Vec<u8>> {
        self.call2(ON_GATEWAY_RESPONSE_FUNCTION, context, (operation, request))
            .await?
            .map(|result: Vec<u8>| Ok(result))
            .unwrap_or_else(|| Ok(Vec::new()))
    }

    /// Called right after a HTTP request.
    pub async fn on_http_response(
        &mut self,
        context: SharedContext,
        request: ExecutedHttpRequest,
    ) -> crate::Result<()> {
        self.call1_0(ON_HTTP_RESPONSE_FUNCTION, context, request).await
    }
}
