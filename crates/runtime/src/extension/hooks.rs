use std::future::Future;

use engine_schema::{GraphqlSubgraph, VirtualSubgraph};
use error::{ErrorResponse, GraphqlError};
use event_queue::EventQueue;
use http::{request, response};
use url::Url;

use crate::extension::{AuthorizedContext, OnRequestContext};

pub struct OnRequest {
    pub event_queue: EventQueue,
    pub parts: request::Parts,
    pub contract_key: Option<String>,
    pub context: Vec<u8>,
}

pub trait GatewayHooksExtension: Clone + Send + Sync + 'static {
    fn on_request(&self, parts: request::Parts) -> impl Future<Output = Result<OnRequest, ErrorResponse>> + Send;

    fn on_response<Context>(
        &self,
        context: Context,
        parts: response::Parts,
    ) -> impl Future<Output = Result<response::Parts, String>> + Send
    where
        Context: OnRequestContext;
}

pub struct ReqwestParts {
    pub url: Url,
    pub method: http::Method,
    pub headers: http::HeaderMap,
}

pub trait EngineHooksExtension: Send + Sync + 'static {
    fn on_graphql_subgraph_request<Context>(
        &self,
        context: Context,
        subgraph: GraphqlSubgraph<'_>,
        parts: ReqwestParts,
    ) -> impl Future<Output = Result<ReqwestParts, GraphqlError>> + Send
    where
        Context: AuthorizedContext;

    fn on_virtual_subgraph_request<Context>(
        &self,
        context: Context,
        subgraph: VirtualSubgraph<'_>,
        headers: http::HeaderMap,
    ) -> impl Future<Output = Result<http::HeaderMap, GraphqlError>> + Send
    where
        Context: AuthorizedContext;
}
