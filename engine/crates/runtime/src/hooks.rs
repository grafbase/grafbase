#[cfg(feature = "test-utils")]
mod test_utils;

use grafbase_telemetry::gql_response_status::{GraphqlResponseStatus, SubgraphResponseStatus};
#[cfg(feature = "test-utils")]
pub use test_utils::*;
use url::Url;

use std::future::Future;
use web_time::Instant;

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

pub trait Hooks: Send + Sync + 'static {
    type Context: Clone + Send + Sync + 'static;

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
pub struct ResponseInfo {
    pub connection_time_ms: u64,
    pub response_time_ms: u64,
    pub status_code: u16,
}

#[derive(Debug, Clone, Copy)]
pub enum SubgraphRequestExecutionKind {
    InternalServerError,
    HookError,
    RequestError,
    RateLimited,
    Responsed(ResponseInfo),
}

impl ResponseInfo {
    pub fn builder() -> ResponseInfoBuilder {
        ResponseInfoBuilder {
            start: Instant::now(),
            connection_time_ms: None,
            response_time_ms: None,
        }
    }
}

pub struct ResponseInfoBuilder {
    start: Instant,
    connection_time_ms: Option<u64>,
    response_time_ms: Option<u64>,
}

pub struct OnSubgraphResponseOutput(pub Vec<u8>);

impl ResponseInfoBuilder {
    /// Stops the clock for connection time. This is typically the time the request gets
    /// sent, but no data is fetched back.
    pub fn track_connection(&mut self) {
        self.connection_time_ms = Some(self.start.elapsed().as_millis() as u64);
    }

    /// Stops the clock for response time. This time is the time it takes to initialize
    /// a connection and waiting to get all the data back.
    pub fn track_response(&mut self) {
        self.response_time_ms = Some(self.start.elapsed().as_millis() as u64);
    }

    pub fn finalize(self, status_code: u16) -> ResponseInfo {
        ResponseInfo {
            connection_time_ms: self.connection_time_ms.unwrap_or_default(),
            response_time_ms: self.response_time_ms.unwrap_or_default(),
            status_code,
        }
    }
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
    pub executions: Vec<SubgraphRequestExecutionKind>,
    pub cache_status: CacheStatus,
    pub total_duration_ms: u64,
    pub has_errors: bool,
}

#[derive(Debug, Clone)]
pub struct ExecutedSubgraphRequestBuilder<'a> {
    subgraph_name: &'a str,
    method: &'a str,
    url: &'a str,
    executions: Vec<SubgraphRequestExecutionKind>,
    cache_status: CacheStatus,
    start_time: Instant,
    status: SubgraphResponseStatus,
}

impl<'a> ExecutedSubgraphRequestBuilder<'a> {
    pub fn push_execution(&mut self, kind: SubgraphRequestExecutionKind) {
        self.executions.push(kind);
    }

    pub fn set_cache_status(&mut self, status: CacheStatus) {
        self.cache_status = status;
    }

    pub fn set_graphql_status(&mut self, status: SubgraphResponseStatus) {
        self.status = status;
    }

    pub fn build(self) -> ExecutedSubgraphRequest<'a> {
        let is_success = matches!(
            self.status,
            SubgraphResponseStatus::GraphqlResponse(GraphqlResponseStatus::Success)
        );

        ExecutedSubgraphRequest {
            subgraph_name: self.subgraph_name,
            method: self.method,
            url: self.url,
            executions: self.executions,
            cache_status: self.cache_status,
            total_duration_ms: self.start_time.elapsed().as_millis() as u64,
            has_errors: !is_success,
        }
    }
}

impl<'a> ExecutedSubgraphRequest<'a> {
    pub fn builder(subgraph_name: &'a str, method: &'a str, url: &'a str) -> ExecutedSubgraphRequestBuilder<'a> {
        ExecutedSubgraphRequestBuilder {
            subgraph_name,
            method,
            url,
            executions: Vec::new(),
            cache_status: CacheStatus::Miss,
            start_time: Instant::now(),
            status: SubgraphResponseStatus::GraphqlResponse(GraphqlResponseStatus::Success),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecutedOperation<'a> {
    pub name: Option<String>,
    pub document: &'a str,
    pub prepare_duration_ms: u64,
    pub cached: bool,
    pub duration_ms: u64,
    pub status: GraphqlResponseStatus,
    pub on_subgraph_response_outputs: Vec<Vec<u8>>,
}

impl<'a> ExecutedOperation<'a> {
    pub fn builder() -> ExecutedOperationBuilder {
        ExecutedOperationBuilder {
            start_time: Instant::now(),
            on_subgraph_response_outputs: Vec::new(),
            name: None,
            prepare_duration_ms: None,
            cached: false,
        }
    }
}

#[derive(Debug)]
pub struct ExecutedOperationBuilder {
    name: Option<String>,
    prepare_duration_ms: Option<u64>,
    cached: bool,
    start_time: Instant,
    on_subgraph_response_outputs: Vec<Vec<u8>>,
}

impl ExecutedOperationBuilder {
    pub fn set_on_subgraph_response_outputs(&mut self, outputs: Vec<Vec<u8>>) {
        self.on_subgraph_response_outputs = outputs;
    }

    pub fn set_name(&mut self, name: Option<impl Into<String>>) {
        self.name = name.map(Into::into)
    }

    pub fn track_prepare(&mut self) {
        self.prepare_duration_ms = Some(self.start_time.elapsed().as_millis() as u64);
    }

    pub fn set_cached(&mut self) {
        self.cached = true;
    }

    pub fn finalize(self, document: &str, status: GraphqlResponseStatus) -> ExecutedOperation<'_> {
        ExecutedOperation {
            duration_ms: self.start_time.elapsed().as_millis() as u64,
            status,
            on_subgraph_response_outputs: self.on_subgraph_response_outputs,
            name: self.name,
            document,
            prepare_duration_ms: self.prepare_duration_ms.unwrap_or_default(),
            cached: self.cached,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecutedHttpRequest {
    pub method: http::Method,
    pub url: http::Uri,
    pub status_code: http::StatusCode,
    pub on_operation_response_outputs: Vec<Vec<u8>>,
}

pub trait ResponseHooks<Context>: Send + Sync + 'static {
    fn on_subgraph_response(
        &self,
        context: &Context,
        request: ExecutedSubgraphRequest<'_>,
    ) -> impl Future<Output = Result<Vec<u8>, PartialGraphqlError>> + Send;

    fn on_operation_response(
        &self,
        context: &Context,
        operation: ExecutedOperation<'_>,
    ) -> impl Future<Output = Result<Vec<u8>, PartialGraphqlError>> + Send;

    fn on_http_response(
        &self,
        context: &Context,
        request: ExecutedHttpRequest,
    ) -> impl Future<Output = Result<(), PartialGraphqlError>> + Send;
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
    async fn on_subgraph_response(
        &self,
        _: &(),
        _: ExecutedSubgraphRequest<'_>,
    ) -> Result<Vec<u8>, PartialGraphqlError> {
        Ok(Vec::new())
    }

    async fn on_operation_response(&self, _: &(), _: ExecutedOperation<'_>) -> Result<Vec<u8>, PartialGraphqlError> {
        Ok(Vec::new())
    }

    async fn on_http_response(&self, _: &(), _: ExecutedHttpRequest) -> Result<(), PartialGraphqlError> {
        Ok(())
    }
}
