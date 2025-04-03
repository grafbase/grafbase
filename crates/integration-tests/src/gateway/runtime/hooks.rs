use crate::gateway::{DynHookContext, ExtContext};
use engine::{ErrorCode, ErrorResponse, GraphqlError};
use futures_util::{FutureExt, future::BoxFuture};

use runtime::hooks::*;

/// Dynamic hooks, for testing purposes to have a default implementation and avoid
/// re-compiling the whole engine with different hooks types.
///
/// Instead of a context, a request id is generated which can be used to keep track of some
/// request-specific data.
#[allow(unused_variables)] // makes it easier to copy-paste relevant functions
#[async_trait::async_trait]
pub trait DynHooks: Send + Sync + 'static {
    async fn on_gateway_request(
        &self,
        context: &mut DynHookContext,
        url: &str,
        headers: HeaderMap,
    ) -> Result<HeaderMap, ErrorResponse> {
        Ok(headers)
    }

    async fn authorize_edge_pre_execution(
        &self,
        context: &DynHookContext,
        definition: EdgeDefinition<'_>,
        arguments: serde_json::Value,
        metadata: Option<serde_json::Value>,
    ) -> AuthorizationVerdict {
        Err(GraphqlError::new(
            "authorize_edge_pre_execution is not implemented",
            ErrorCode::Unauthorized,
        ))
    }

    async fn authorize_node_pre_execution(
        &self,
        context: &DynHookContext,
        definition: NodeDefinition<'_>,
        metadata: Option<serde_json::Value>,
    ) -> AuthorizationVerdict {
        Err(GraphqlError::new(
            "authorize_node_pre_execution is not implemented",
            ErrorCode::Unauthorized,
        ))
    }

    async fn authorize_node_post_execution(
        &self,
        context: &DynHookContext,
        definition: NodeDefinition<'_>,
        nodes: Vec<serde_json::Value>,
        metadata: Option<serde_json::Value>,
    ) -> AuthorizationVerdicts {
        Err(GraphqlError::new(
            "authorize_node_post_execution is not implemented",
            ErrorCode::Unauthorized,
        ))
    }

    async fn authorize_parent_edge_post_execution(
        &self,
        context: &DynHookContext,
        definition: EdgeDefinition<'_>,
        parents: Vec<serde_json::Value>,
        metadata: Option<serde_json::Value>,
    ) -> AuthorizationVerdicts {
        Err(GraphqlError::new(
            "authorize_parent_edge_post_execution is not implemented",
            ErrorCode::Unauthorized,
        ))
    }

    async fn authorize_edge_node_post_execution(
        &self,
        context: &DynHookContext,
        definition: EdgeDefinition<'_>,
        nodes: Vec<serde_json::Value>,
        metadata: Option<serde_json::Value>,
    ) -> AuthorizationVerdicts {
        Err(GraphqlError::new(
            "authorize_edge_node_post_execution is not implemented",
            ErrorCode::Unauthorized,
        ))
    }

    async fn authorize_edge_post_execution(
        &self,
        context: &DynHookContext,
        definition: EdgeDefinition<'_>,
        edges: Vec<(serde_json::Value, Vec<serde_json::Value>)>,
        metadata: Option<serde_json::Value>,
    ) -> AuthorizationVerdicts {
        Err(GraphqlError::new(
            "authorize_edge_post_execution is not implemented",
            ErrorCode::Unauthorized,
        ))
    }

    async fn on_subgraph_request(
        &self,
        context: &DynHookContext,
        subgraph_name: &str,
        request: SubgraphRequest,
    ) -> Result<SubgraphRequest, GraphqlError> {
        Ok(request)
    }

    async fn on_subgraph_response(
        &self,
        context: &DynHookContext,
        request: ExecutedSubgraphRequest<'_>,
    ) -> Result<Vec<u8>, GraphqlError> {
        Ok(Vec::new())
    }

    async fn on_gateway_response(
        &self,
        context: &DynHookContext,
        request: ExecutedOperation<'_, Vec<u8>>,
    ) -> Result<Vec<u8>, GraphqlError> {
        Ok(Vec::new())
    }

    async fn on_http_response(
        &self,
        context: &DynHookContext,
        request: ExecutedHttpRequest<Vec<u8>>,
    ) -> Result<(), GraphqlError> {
        Ok(())
    }
}

impl<T: DynHooks> From<T> for DynamicHooks {
    fn from(hooks: T) -> Self {
        Self::new(hooks)
    }
}

pub struct DynamicHooks(Box<dyn DynHooks>);

impl Default for DynamicHooks {
    fn default() -> Self {
        Self::new(DynWrapper(()))
    }
}

impl DynamicHooks {
    pub fn wrap<H: Hooks>(hooks: H) -> Self {
        Self::new(DynWrapper(hooks))
    }

    pub fn new(hooks: impl DynHooks) -> Self {
        Self(Box::new(hooks))
    }
}

impl Hooks for DynamicHooks {
    type Context = ExtContext;
    type OnSubgraphResponseOutput = Vec<u8>;
    type OnOperationResponseOutput = Vec<u8>;

    fn new_context(&self) -> Self::Context {
        Default::default()
    }

    async fn on_gateway_request(
        &self,
        url: &str,
        headers: HeaderMap,
    ) -> Result<(Self::Context, HeaderMap), (Self::Context, ErrorResponse)> {
        let mut context = ExtContext::default();

        match self.0.on_gateway_request(&mut context.test, url, headers).await {
            Ok(headers) => Ok((context, headers)),
            Err(error) => Err((context, error)),
        }
    }

    async fn on_subgraph_request(
        &self,
        context: &ExtContext,
        subgraph_name: &str,
        request: SubgraphRequest,
    ) -> Result<SubgraphRequest, GraphqlError> {
        self.0.on_subgraph_request(&context.test, subgraph_name, request).await
    }

    async fn on_subgraph_response(
        &self,
        context: &ExtContext,
        request: ExecutedSubgraphRequest<'_>,
    ) -> Result<Vec<u8>, GraphqlError> {
        self.0.on_subgraph_response(&context.test, request).await
    }

    async fn on_operation_response(
        &self,
        context: &ExtContext,
        operation: ExecutedOperation<'_, Self::OnSubgraphResponseOutput>,
    ) -> Result<Vec<u8>, GraphqlError> {
        self.0.on_gateway_response(&context.test, operation).await
    }

    async fn on_http_response(
        &self,
        context: &ExtContext,
        request: ExecutedHttpRequest<Self::OnOperationResponseOutput>,
    ) -> Result<(), GraphqlError> {
        self.0.on_http_response(&context.test, request).await
    }

    fn authorized(&self) -> &impl AuthorizedHooks<Self::Context> {
        self
    }
}

impl AuthorizedHooks<ExtContext> for DynamicHooks {
    async fn authorize_edge_pre_execution<'a>(
        &self,
        context: &ExtContext,
        definition: EdgeDefinition<'a>,
        arguments: impl Anything<'a>,
        metadata: Option<impl Anything<'a>>,
    ) -> AuthorizationVerdict {
        self.0
            .authorize_edge_pre_execution(
                &context.test,
                definition,
                serde_json::to_value(&arguments).unwrap(),
                metadata.map(|m| serde_json::to_value(&m).unwrap()),
            )
            .await
    }

    async fn authorize_node_post_execution<'a>(
        &self,
        context: &ExtContext,
        definition: NodeDefinition<'a>,
        nodes: impl IntoIterator<Item: Anything<'a>> + Send,
        metadata: Option<impl Anything<'a>>,
    ) -> AuthorizationVerdicts {
        self.0
            .authorize_node_post_execution(
                &context.test,
                definition,
                nodes
                    .into_iter()
                    .map(|value| serde_json::to_value(&value).unwrap())
                    .collect(),
                metadata.map(|m| serde_json::to_value(&m).unwrap()),
            )
            .await
    }

    async fn authorize_node_pre_execution<'a>(
        &self,
        context: &ExtContext,
        definition: NodeDefinition<'a>,
        metadata: Option<impl Anything<'a>>,
    ) -> AuthorizationVerdict {
        self.0
            .authorize_node_pre_execution(
                &context.test,
                definition,
                metadata.map(|m| serde_json::to_value(&m).unwrap()),
            )
            .await
    }

    async fn authorize_parent_edge_post_execution<'a>(
        &self,
        context: &ExtContext,
        definition: EdgeDefinition<'a>,
        parents: impl IntoIterator<Item: Anything<'a>> + Send,
        metadata: Option<impl Anything<'a>>,
    ) -> AuthorizationVerdicts {
        self.0
            .authorize_parent_edge_post_execution(
                &context.test,
                definition,
                parents
                    .into_iter()
                    .map(|value| serde_json::to_value(&value).unwrap())
                    .collect(),
                metadata.map(|m| serde_json::to_value(&m).unwrap()),
            )
            .await
    }

    async fn authorize_edge_node_post_execution<'a>(
        &self,
        context: &ExtContext,
        definition: EdgeDefinition<'a>,
        nodes: impl IntoIterator<Item: Anything<'a>> + Send,
        metadata: Option<impl Anything<'a>>,
    ) -> AuthorizationVerdicts {
        self.0
            .authorize_edge_node_post_execution(
                &context.test,
                definition,
                nodes
                    .into_iter()
                    .map(|value| serde_json::to_value(&value).unwrap())
                    .collect(),
                metadata.map(|m| serde_json::to_value(&m).unwrap()),
            )
            .await
    }

    async fn authorize_edge_post_execution<'a, Parent, Nodes>(
        &self,
        context: &ExtContext,
        definition: EdgeDefinition<'a>,
        edges: impl IntoIterator<Item = (Parent, Nodes)> + Send,
        metadata: Option<impl Anything<'a>>,
    ) -> AuthorizationVerdicts
    where
        Parent: Anything<'a>,
        Nodes: IntoIterator<Item: Anything<'a>> + Send,
    {
        self.0
            .authorize_edge_post_execution(
                &context.test,
                definition,
                edges
                    .into_iter()
                    .map(|(parent, nodes)| {
                        (
                            serde_json::to_value(&parent).unwrap(),
                            nodes
                                .into_iter()
                                .map(|node| serde_json::to_value(&node).unwrap())
                                .collect(),
                        )
                    })
                    .collect(),
                metadata.map(|m| serde_json::to_value(&m).unwrap()),
            )
            .await
    }
}

struct DynWrapper<T>(T);

impl<H: Hooks> DynHooks for DynWrapper<H> {
    fn on_gateway_request<'a, 'b, 'c, 'fut>(
        &'a self,
        context: &'b mut DynHookContext,
        url: &'c str,
        headers: HeaderMap,
    ) -> BoxFuture<'fut, Result<HeaderMap, ErrorResponse>>
    where
        'a: 'fut,
        'b: 'fut,
        'c: 'fut,
    {
        async {
            match Hooks::on_gateway_request(&self.0, url, headers).await {
                Ok((ctx, headers)) => {
                    context.typed_insert(ctx);
                    Ok(headers)
                }
                Err((_, error)) => Err(error),
            }
        }
        .boxed()
    }

    // FIXME: Had to write them explicitly because of: https://github.com/rust-lang/rust/issues/100013
    fn authorize_edge_pre_execution<'a, 'b, 'c, 'fut>(
        &'a self,
        context: &'b DynHookContext,
        definition: EdgeDefinition<'c>,
        arguments: serde_json::Value,
        metadata: Option<serde_json::Value>,
    ) -> BoxFuture<'fut, AuthorizationVerdict>
    where
        'a: 'fut,
        'b: 'fut,
        'c: 'fut,
    {
        Hooks::authorized(&self.0)
            .authorize_edge_pre_execution(context.typed_get().unwrap(), definition, arguments, metadata)
            .boxed()
    }

    fn authorize_node_pre_execution<'a, 'b, 'c, 'fut>(
        &'a self,
        context: &'b DynHookContext,
        definition: NodeDefinition<'c>,
        metadata: Option<serde_json::Value>,
    ) -> BoxFuture<'fut, AuthorizationVerdict>
    where
        'a: 'fut,
        'b: 'fut,
        'c: 'fut,
    {
        Hooks::authorized(&self.0)
            .authorize_node_pre_execution(context.typed_get().unwrap(), definition, metadata)
            .boxed()
    }

    fn authorize_node_post_execution<'a, 'b, 'c, 'fut>(
        &'a self,
        context: &'b DynHookContext,
        definition: NodeDefinition<'c>,
        nodes: Vec<serde_json::Value>,
        metadata: Option<serde_json::Value>,
    ) -> BoxFuture<'fut, AuthorizationVerdicts>
    where
        'a: 'fut,
        'b: 'fut,
        'c: 'fut,
    {
        Hooks::authorized(&self.0)
            .authorize_node_post_execution(context.typed_get().unwrap(), definition, nodes, metadata)
            .boxed()
    }

    fn authorize_parent_edge_post_execution<'a, 'b, 'c, 'fut>(
        &'a self,
        context: &'b DynHookContext,
        definition: EdgeDefinition<'c>,
        parents: Vec<serde_json::Value>,
        metadata: Option<serde_json::Value>,
    ) -> BoxFuture<'fut, AuthorizationVerdicts>
    where
        'a: 'fut,
        'b: 'fut,
        'c: 'fut,
    {
        Hooks::authorized(&self.0)
            .authorize_parent_edge_post_execution(context.typed_get().unwrap(), definition, parents, metadata)
            .boxed()
    }

    fn authorize_edge_node_post_execution<'a, 'b, 'c, 'fut>(
        &'a self,
        context: &'b DynHookContext,
        definition: EdgeDefinition<'c>,
        nodes: Vec<serde_json::Value>,
        metadata: Option<serde_json::Value>,
    ) -> BoxFuture<'fut, AuthorizationVerdicts>
    where
        'a: 'fut,
        'b: 'fut,
        'c: 'fut,
    {
        Hooks::authorized(&self.0)
            .authorize_edge_node_post_execution(context.typed_get().unwrap(), definition, nodes, metadata)
            .boxed()
    }

    fn authorize_edge_post_execution<'a, 'b, 'c, 'fut>(
        &'a self,
        context: &'b DynHookContext,
        definition: EdgeDefinition<'c>,
        edges: Vec<(serde_json::Value, Vec<serde_json::Value>)>,
        metadata: Option<serde_json::Value>,
    ) -> BoxFuture<'fut, AuthorizationVerdicts>
    where
        'a: 'fut,
        'b: 'fut,
        'c: 'fut,
    {
        Hooks::authorized(&self.0)
            .authorize_edge_post_execution(context.typed_get().unwrap(), definition, edges, metadata)
            .boxed()
    }

    fn on_subgraph_request<'a, 'b, 'c, 'fut>(
        &'a self,
        context: &'b DynHookContext,
        subgraph_name: &'c str,
        request: SubgraphRequest,
    ) -> BoxFuture<'fut, Result<SubgraphRequest, GraphqlError>>
    where
        'a: 'fut,
        'b: 'fut,
        'c: 'fut,
    {
        Hooks::on_subgraph_request(&self.0, context.typed_get().unwrap(), subgraph_name, request).boxed()
    }
}
