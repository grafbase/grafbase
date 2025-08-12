use std::{future::Future, sync::Arc};

use engine_schema::{GraphqlSubgraph, VirtualSubgraph};
use error::{ErrorResponse, GraphqlError};
use event_queue::EventQueue;
use http::{request, response};
use url::Url;

pub struct OnRequest {
    pub parts: request::Parts,
    pub contract_key: Option<String>,
    pub event_queue: Arc<EventQueue>,
    // Arc for Wasmtime because we can't return an non 'static value from a function.
    pub hooks_context: Arc<[u8]>,
}

pub trait GatewayHooksExtension: Clone + Send + Sync + 'static {
    fn on_request(&self, parts: request::Parts) -> impl Future<Output = Result<OnRequest, ErrorResponse>> + Send;

    fn on_response(
        &self,
        event_queue: Arc<EventQueue>,
        hooks_context: Arc<[u8]>,
        parts: response::Parts,
    ) -> impl Future<Output = Result<response::Parts, String>> + Send;
}

pub struct ReqwestParts {
    pub url: Url,
    pub method: http::Method,
    pub headers: http::HeaderMap,
}

pub trait EngineHooksExtension<OperationContext>: Send + Sync + 'static {
    fn on_graphql_subgraph_request(
        &self,
        context: OperationContext,
        subgraph: GraphqlSubgraph<'_>,
        parts: ReqwestParts,
    ) -> impl Future<Output = Result<ReqwestParts, GraphqlError>> + Send;

    fn on_virtual_subgraph_request(
        &self,
        context: OperationContext,
        subgraph: VirtualSubgraph<'_>,
        headers: http::HeaderMap,
    ) -> impl Future<Output = Result<http::HeaderMap, GraphqlError>> + Send;
}
