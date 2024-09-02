use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

use futures_util::{future::BoxFuture, FutureExt};

use super::*;

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
        Err(PartialGraphqlError::new(
            "authorize_edge_pre_execution is not implemented",
            PartialErrorCode::Unauthorized,
        ))
    }

    async fn authorize_node_pre_execution(
        &self,
        context: &DynHookContext,
        definition: NodeDefinition<'_>,
        metadata: Option<serde_json::Value>,
    ) -> AuthorizationVerdict {
        Err(PartialGraphqlError::new(
            "authorize_node_pre_execution is not implemented",
            PartialErrorCode::Unauthorized,
        ))
    }

    async fn authorize_node_post_execution(
        &self,
        context: &DynHookContext,
        definition: NodeDefinition<'_>,
        nodes: Vec<serde_json::Value>,
        metadata: Option<serde_json::Value>,
    ) -> AuthorizationVerdicts {
        Err(PartialGraphqlError::new(
            "authorize_node_post_execution is not implemented",
            PartialErrorCode::Unauthorized,
        ))
    }

    async fn authorize_parent_edge_post_execution(
        &self,
        context: &DynHookContext,
        definition: EdgeDefinition<'_>,
        parents: Vec<serde_json::Value>,
        metadata: Option<serde_json::Value>,
    ) -> AuthorizationVerdicts {
        Err(PartialGraphqlError::new(
            "authorize_parent_edge_post_execution is not implemented",
            PartialErrorCode::Unauthorized,
        ))
    }

    async fn authorize_edge_node_post_execution(
        &self,
        context: &DynHookContext,
        definition: EdgeDefinition<'_>,
        nodes: Vec<serde_json::Value>,
        metadata: Option<serde_json::Value>,
    ) -> AuthorizationVerdicts {
        Err(PartialGraphqlError::new(
            "authorize_edge_node_post_execution is not implemented",
            PartialErrorCode::Unauthorized,
        ))
    }

    async fn authorize_edge_post_execution(
        &self,
        context: &DynHookContext,
        definition: EdgeDefinition<'_>,
        edges: Vec<(serde_json::Value, Vec<serde_json::Value>)>,
        metadata: Option<serde_json::Value>,
    ) -> AuthorizationVerdicts {
        Err(PartialGraphqlError::new(
            "authorize_edge_post_execution is not implemented",
            PartialErrorCode::Unauthorized,
        ))
    }

    async fn on_subgraph_request(
        &self,
        context: &DynHookContext,
        subgraph_name: &str,
        method: http::Method,
        url: &Url,
        headers: HeaderMap,
    ) -> Result<HeaderMap, PartialGraphqlError> {
        Ok(headers)
    }

    async fn on_subgraph_response(
        &self,
        context: &DynHookContext,
        request: ExecutedSubgraphRequest<'_>,
    ) -> Result<Vec<u8>, PartialGraphqlError> {
        Ok(Vec::new())
    }

    async fn on_gateway_response(
        &self,
        context: &DynHookContext,
        operation: Operation<'_>,
        request: ExecutedOperationRequest,
    ) -> Result<Vec<u8>, PartialGraphqlError> {
        Ok(Vec::new())
    }

    async fn on_http_response(
        &self,
        context: &DynHookContext,
        request: ExecutedHttpRequest,
    ) -> Result<(), PartialGraphqlError> {
        Ok(())
    }
}

#[derive(Default, Clone)]
pub struct DynHookContext {
    by_type: HashMap<TypeId, Arc<dyn Any + Sync + Send>>,
    by_name: HashMap<String, String>,
}

impl DynHookContext {
    pub fn typed_get<T>(&self) -> Option<&T>
    where
        T: 'static + Send + Sync,
    {
        self.by_type
            .get(&TypeId::of::<T>())
            .and_then(|value| value.downcast_ref::<T>())
    }

    pub fn typed_insert<T>(&mut self, value: T)
    where
        T: 'static + Send + Sync,
    {
        self.by_type.insert(TypeId::of::<T>(), Arc::new(value));
    }

    pub fn get(&self, name: &str) -> Option<&String> {
        self.by_name.get(name)
    }

    pub fn insert(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.by_name.insert(name.into(), value.into());
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
    type Context = DynHookContext;

    async fn on_gateway_request(&self, headers: HeaderMap) -> Result<(Self::Context, HeaderMap), ErrorResponse> {
        let mut context = DynHookContext::default();
        let headers = self.0.on_gateway_request(&mut context, headers).await?;
        Ok((context, headers))
    }

    fn authorized(&self) -> &impl AuthorizedHooks<Self::Context> {
        self
    }

    fn subgraph(&self) -> &impl SubgraphHooks<Self::Context> {
        self
    }

    fn responses(&self) -> &impl ResponseHooks<Self::Context> {
        self
    }
}

impl AuthorizedHooks<DynHookContext> for DynamicHooks {
    async fn authorize_edge_pre_execution<'a>(
        &self,
        context: &DynHookContext,
        definition: EdgeDefinition<'a>,
        arguments: impl Anything<'a>,
        metadata: Option<impl Anything<'a>>,
    ) -> AuthorizationVerdict {
        self.0
            .authorize_edge_pre_execution(
                context,
                definition,
                serde_json::to_value(&arguments).unwrap(),
                metadata.map(|m| serde_json::to_value(&m).unwrap()),
            )
            .await
    }

    async fn authorize_node_post_execution<'a>(
        &self,
        context: &DynHookContext,
        definition: NodeDefinition<'a>,
        nodes: impl IntoIterator<Item: Anything<'a>> + Send,
        metadata: Option<impl Anything<'a>>,
    ) -> AuthorizationVerdicts {
        self.0
            .authorize_node_post_execution(
                context,
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
        context: &DynHookContext,
        definition: NodeDefinition<'a>,
        metadata: Option<impl Anything<'a>>,
    ) -> AuthorizationVerdict {
        self.0
            .authorize_node_pre_execution(context, definition, metadata.map(|m| serde_json::to_value(&m).unwrap()))
            .await
    }

    async fn authorize_parent_edge_post_execution<'a>(
        &self,
        context: &DynHookContext,
        definition: EdgeDefinition<'a>,
        parents: impl IntoIterator<Item: Anything<'a>> + Send,
        metadata: Option<impl Anything<'a>>,
    ) -> AuthorizationVerdicts {
        self.0
            .authorize_parent_edge_post_execution(
                context,
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
        context: &DynHookContext,
        definition: EdgeDefinition<'a>,
        nodes: impl IntoIterator<Item: Anything<'a>> + Send,
        metadata: Option<impl Anything<'a>>,
    ) -> AuthorizationVerdicts {
        self.0
            .authorize_edge_node_post_execution(
                context,
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
        context: &DynHookContext,
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
                context,
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

impl SubgraphHooks<DynHookContext> for DynamicHooks {
    async fn on_subgraph_request(
        &self,
        context: &DynHookContext,
        subgraph_name: &str,
        method: http::Method,
        url: &Url,
        headers: HeaderMap,
    ) -> Result<HeaderMap, PartialGraphqlError> {
        self.0
            .on_subgraph_request(context, subgraph_name, method, url, headers)
            .await
    }
}

impl ResponseHooks<DynHookContext> for DynamicHooks {
    async fn on_subgraph_response(
        &self,
        context: &DynHookContext,
        request: ExecutedSubgraphRequest<'_>,
    ) -> Result<Vec<u8>, PartialGraphqlError> {
        self.0.on_subgraph_response(context, request).await
    }

    async fn on_operation_response(
        &self,
        context: &DynHookContext,
        operation: Operation<'_>,
        request: ExecutedOperationRequest,
    ) -> Result<Vec<u8>, PartialGraphqlError> {
        self.0.on_gateway_response(context, operation, request).await
    }

    async fn on_http_response(
        &self,
        context: &DynHookContext,
        request: ExecutedHttpRequest,
    ) -> Result<(), PartialGraphqlError> {
        self.0.on_http_response(context, request).await
    }
}

struct DynWrapper<T>(T);

impl<H: Hooks> DynHooks for DynWrapper<H> {
    fn on_gateway_request<'a, 'b, 'fut>(
        &'a self,
        context: &'b mut DynHookContext,
        headers: HeaderMap,
    ) -> BoxFuture<'fut, Result<HeaderMap, ErrorResponse>>
    where
        'a: 'fut,
        'b: 'fut,
    {
        async {
            let (ctx, headers) = Hooks::on_gateway_request(&self.0, headers).await?;
            context.typed_insert(ctx);
            Ok(headers)
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

    fn on_subgraph_request<'a, 'b, 'c, 'd, 'fut>(
        &'a self,
        context: &'b DynHookContext,
        subgraph_name: &'c str,
        method: http::Method,
        url: &'d Url,
        headers: HeaderMap,
    ) -> BoxFuture<'fut, Result<HeaderMap, PartialGraphqlError>>
    where
        'a: 'fut,
        'b: 'fut,
        'c: 'fut,
        'd: 'fut,
    {
        Hooks::subgraph(&self.0)
            .on_subgraph_request(context.typed_get().unwrap(), subgraph_name, method, url, headers)
            .boxed()
    }
}
