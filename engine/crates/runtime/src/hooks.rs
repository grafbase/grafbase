#[cfg(feature = "test-utils")]
mod test_utils;

#[cfg(feature = "test-utils")]
pub use test_utils::*;
mod on_response;
pub use on_response::*;
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

#[allow(unused)]
pub trait Hooks: Send + Sync + 'static {
    type Context: Clone + Send + Sync + 'static;
    type OnSubgraphResponseOutput: Send + Sync + 'static;
    type OnOperationResponseOutput: Send + Sync + 'static;

    fn on_gateway_request(
        &self,
        headers: HeaderMap,
    ) -> impl Future<Output = Result<(Self::Context, HeaderMap), (Self::Context, ErrorResponse)>> + Send;

    fn on_subgraph_request(
        &self,
        context: &Self::Context,
        subgraph_name: &str,
        method: http::Method,
        url: &Url,
        headers: HeaderMap,
    ) -> impl Future<Output = Result<HeaderMap, PartialGraphqlError>> + Send {
        async { Ok(headers) }
    }

    fn on_subgraph_response(
        &self,
        context: &Self::Context,
        request: ExecutedSubgraphRequest<'_>,
    ) -> impl Future<Output = Result<Self::OnSubgraphResponseOutput, PartialGraphqlError>> + Send;

    fn on_operation_response(
        &self,
        context: &Self::Context,
        operation: ExecutedOperation<'_, Self::OnSubgraphResponseOutput>,
    ) -> impl Future<Output = Result<Self::OnOperationResponseOutput, PartialGraphqlError>> + Send;

    fn on_http_response(
        &self,
        context: &Self::Context,
        request: ExecutedHttpRequest<Self::OnOperationResponseOutput>,
    ) -> impl Future<Output = Result<(), PartialGraphqlError>> + Send;

    fn authorized(&self) -> &impl AuthorizedHooks<Self::Context> {
        &()
    }
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

// ---------------------------//
// -- No-op implementation -- //
// ---------------------------//
impl Hooks for () {
    type Context = ();
    type OnSubgraphResponseOutput = ();
    type OnOperationResponseOutput = ();

    async fn on_gateway_request(
        &self,
        headers: HeaderMap,
    ) -> Result<(Self::Context, HeaderMap), (Self::Context, ErrorResponse)> {
        Ok(((), headers))
    }

    fn authorized(&self) -> &impl AuthorizedHooks<()> {
        self
    }

    async fn on_subgraph_response(
        &self,
        _context: &Self::Context,
        _request: ExecutedSubgraphRequest<'_>,
    ) -> Result<Self::OnSubgraphResponseOutput, PartialGraphqlError> {
        Ok(())
    }

    async fn on_operation_response(
        &self,
        _context: &Self::Context,
        _operation: ExecutedOperation<'_, ()>,
    ) -> Result<Self::OnOperationResponseOutput, PartialGraphqlError> {
        Ok(())
    }

    async fn on_http_response(
        &self,
        _context: &Self::Context,
        _request: ExecutedHttpRequest<()>,
    ) -> Result<(), PartialGraphqlError> {
        Ok(())
    }
}

impl<C: Send + Sync> AuthorizedHooks<C> for () {
    async fn authorize_edge_pre_execution<'a>(
        &self,
        _: &C,
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
        _: &C,
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
        _: &C,
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
        _: &C,
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
        _: &C,
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
        _: &C,
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
