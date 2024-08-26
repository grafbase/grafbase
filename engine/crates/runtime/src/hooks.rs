#[cfg(feature = "test-utils")]
mod test_utils;

use grafbase_telemetry::gql_response_status::GraphqlResponseStatus;
#[cfg(feature = "test-utils")]
pub use test_utils::*;
use url::Url;

use std::future::Future;

pub use http::HeaderMap;

use crate::error::{ErrorResponse, PartialErrorCode, PartialGraphqlError};

pub struct NodeDefinition<'a> {
    pub type_name: &'a str,
}

impl std::fmt::Display for NodeDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.type_name)
    }
}

pub struct EdgeDefinition<'a> {
    pub parent_type_name: &'a str,
    pub field_name: &'a str,
}

impl std::fmt::Display for EdgeDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.parent_type_name, self.field_name)
    }
}

// Used as a sort of convenient type alias
pub trait Anything<'a>: serde::Serialize + serde::de::Deserializer<'a> + Send {}
impl<'a, T> Anything<'a> for T where T: serde::Serialize + serde::de::Deserializer<'a> + Send {}

pub type AuthorizationVerdict = Result<(), PartialGraphqlError>;
pub type AuthorizationVerdicts = Result<Vec<AuthorizationVerdict>, PartialGraphqlError>;

pub struct ChannelBuilder<'a, Sender: Clone> {
    pub sender: &'a Sender,
    pub lossy_log: bool,
}

pub trait Hooks: Send + Sync + 'static {
    type Context: Send + Sync + 'static;

    fn on_gateway_request(
        &self,
        headers: HeaderMap,
    ) -> impl Future<Output = Result<(Self::Context, HeaderMap), ErrorResponse>> + Send;

    fn authorized(&self) -> &impl AuthorizedHooks<Self::Context>;

    fn subgraph(&self) -> &impl SubgraphHooks<Self::Context>;

    fn responses(&self) -> &impl ResponseHooks<Self::Context>;
}

pub trait AuthorizedHooks<Context>: Send + Sync + 'static {
    fn authorize_edge_pre_execution<'a>(
        &self,
        context: &Context,
        definition: EdgeDefinition<'a>,
        arguments: impl Anything<'a>,
        metadata: Option<impl Anything<'a>>,
    ) -> impl Future<Output = AuthorizationVerdict> + Send;

    fn authorize_node_pre_execution<'a>(
        &self,
        context: &Context,
        definition: NodeDefinition<'a>,
        metadata: Option<impl Anything<'a>>,
    ) -> impl Future<Output = AuthorizationVerdict> + Send;

    fn authorize_node_post_execution<'a>(
        &self,
        context: &Context,
        definition: NodeDefinition<'a>,
        nodes: impl IntoIterator<Item: Anything<'a>> + Send,
        metadata: Option<impl Anything<'a>>,
    ) -> impl Future<Output = AuthorizationVerdicts> + Send;

    fn authorize_parent_edge_post_execution<'a>(
        &self,
        context: &Context,
        definition: EdgeDefinition<'a>,
        parents: impl IntoIterator<Item: Anything<'a>> + Send,
        metadata: Option<impl Anything<'a>>,
    ) -> impl Future<Output = AuthorizationVerdicts> + Send;

    fn authorize_edge_node_post_execution<'a>(
        &self,
        context: &Context,
        definition: EdgeDefinition<'a>,
        nodes: impl IntoIterator<Item: Anything<'a>> + Send,
        metadata: Option<impl Anything<'a>>,
    ) -> impl Future<Output = AuthorizationVerdicts> + Send;

    fn authorize_edge_post_execution<'a, Parent, Nodes>(
        &self,
        context: &Context,
        definition: EdgeDefinition<'a>,
        edges: impl IntoIterator<Item = (Parent, Nodes)> + Send,
        metadata: Option<impl Anything<'a>>,
    ) -> impl Future<Output = AuthorizationVerdicts> + Send
    where
        Parent: Anything<'a>,
        Nodes: IntoIterator<Item: Anything<'a>> + Send;
}

pub trait SubgraphHooks<Context>: Send + Sync + 'static {
    fn on_subgraph_request(
        &self,
        context: &Context,
        subgraph_name: &str,
        method: http::Method,
        url: &Url,
        headers: HeaderMap,
    ) -> impl Future<Output = Result<HeaderMap, PartialGraphqlError>> + Send;
}

#[derive(Debug, Clone, Copy)]
pub struct SubgraphResponseInfo {
    pub connection_time: u64,
    pub response_time: u64,
    pub status_code: u16,
}

#[derive(Debug, Clone, Copy)]
pub enum CacheStatus {
    Hit,
    PartialHit,
    Miss,
}

#[derive(Debug, Clone)]
pub struct ExecutedSubgraphRequest<'a> {
    pub subgraph_name: &'a str,
    pub method: &'a str,
    pub url: &'a str,
    pub response_infos: Vec<SubgraphResponseInfo>,
    pub cache_status: CacheStatus,
    pub total_duration: u64,
    pub has_errors: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct Operation<'a> {
    pub name: Option<&'a str>,
    pub document: &'a str,
    pub prepare_duration: u64,
    pub cached: bool,
}

#[derive(Debug, Clone)]
pub struct ExecutedGatewayRequest {
    pub duration: u64,
    pub status: GraphqlResponseStatus,
    pub on_subgraph_request_outputs: Vec<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct ExecutedHttpRequest<'a> {
    pub method: &'a str,
    pub url: &'a str,
    pub status_code: http::StatusCode,
    pub on_gateway_response_outputs: Vec<Vec<u8>>,
}

pub trait ResponseHooks<Context>: Send + Sync + 'static {
    fn on_subgraph_response(
        &self,
        context: &Context,
        request: ExecutedSubgraphRequest<'_>,
    ) -> impl Future<Output = Result<Vec<u8>, ErrorResponse>> + Send;

    fn on_gateway_response(
        &self,
        context: &Context,
        operation: Operation<'_>,
        request: ExecutedGatewayRequest,
    ) -> impl Future<Output = Result<Vec<u8>, ErrorResponse>> + Send;

    fn on_http_response(
        &self,
        context: &Context,
        request: ExecutedHttpRequest<'_>,
    ) -> impl Future<Output = Result<(), ErrorResponse>> + Send;
}

// ---------------------------//
// -- No-op implementation -- //
// ---------------------------//
impl Hooks for () {
    type Context = ();

    async fn on_gateway_request(&self, headers: HeaderMap) -> Result<(Self::Context, HeaderMap), ErrorResponse> {
        Ok(((), headers))
    }

    fn authorized(&self) -> &impl AuthorizedHooks<()> {
        self
    }

    fn subgraph(&self) -> &impl SubgraphHooks<()> {
        self
    }

    fn responses(&self) -> &impl ResponseHooks<Self::Context> {
        self
    }
}

impl AuthorizedHooks<()> for () {
    async fn authorize_edge_pre_execution<'a>(
        &self,
        _: &(),
        _: EdgeDefinition<'a>,
        _: impl Anything<'a>,
        _: Option<impl Anything<'a>>,
    ) -> AuthorizationVerdict {
        Err(PartialGraphqlError::new(
            "@authorized directive cannot be used, so access was denied",
            PartialErrorCode::Unauthorized,
        ))
    }

    async fn authorize_node_pre_execution<'a>(
        &self,
        _: &(),
        _: NodeDefinition<'a>,
        _: Option<impl Anything<'a>>,
    ) -> AuthorizationVerdict {
        Err(PartialGraphqlError::new(
            "@authorized directive cannot be used, so access was denied",
            PartialErrorCode::Unauthorized,
        ))
    }

    async fn authorize_node_post_execution<'a>(
        &self,
        _: &(),
        _: NodeDefinition<'a>,
        _: impl IntoIterator<Item: Anything<'a>> + Send,
        _: Option<impl Anything<'a>>,
    ) -> AuthorizationVerdicts {
        Err(PartialGraphqlError::new(
            "@authorized directive cannot be used, so access was denied",
            PartialErrorCode::Unauthorized,
        ))
    }

    async fn authorize_parent_edge_post_execution<'a>(
        &self,
        _: &(),
        _: EdgeDefinition<'a>,
        _: impl IntoIterator<Item: Anything<'a>> + Send,
        _: Option<impl Anything<'a>>,
    ) -> AuthorizationVerdicts {
        Err(PartialGraphqlError::new(
            "@authorized directive cannot be used, so access was denied",
            PartialErrorCode::Unauthorized,
        ))
    }

    async fn authorize_edge_node_post_execution<'a>(
        &self,
        _: &(),
        _: EdgeDefinition<'a>,
        _: impl IntoIterator<Item: Anything<'a>> + Send,
        _: Option<impl Anything<'a>>,
    ) -> AuthorizationVerdicts {
        Err(PartialGraphqlError::new(
            "@authorized directive cannot be used, so access was denied",
            PartialErrorCode::Unauthorized,
        ))
    }

    async fn authorize_edge_post_execution<'a, Parent, Nodes>(
        &self,
        _: &(),
        _: EdgeDefinition<'a>,
        _: impl IntoIterator<Item = (Parent, Nodes)> + Send,
        _: Option<impl Anything<'a>>,
    ) -> AuthorizationVerdicts
    where
        Parent: Anything<'a>,
        Nodes: IntoIterator<Item: Anything<'a>> + Send,
    {
        Err(PartialGraphqlError::new(
            "@authorized directive cannot be used, so access was denied",
            PartialErrorCode::Unauthorized,
        ))
    }
}

impl SubgraphHooks<()> for () {
    async fn on_subgraph_request(
        &self,
        _: &(),
        _: &str,
        _: http::Method,
        _: &Url,
        headers: HeaderMap,
    ) -> Result<HeaderMap, PartialGraphqlError> {
        Ok(headers)
    }
}

impl ResponseHooks<()> for () {
    async fn on_subgraph_response(&self, _: &(), _: ExecutedSubgraphRequest<'_>) -> Result<Vec<u8>, ErrorResponse> {
        Ok(Vec::new())
    }

    async fn on_gateway_response(
        &self,
        _: &(),
        _: Operation<'_>,
        _: ExecutedGatewayRequest,
    ) -> Result<Vec<u8>, ErrorResponse> {
        Ok(Vec::new())
    }

    async fn on_http_response(&self, _: &(), _: ExecutedHttpRequest<'_>) -> Result<(), ErrorResponse> {
        Ok(())
    }
}
