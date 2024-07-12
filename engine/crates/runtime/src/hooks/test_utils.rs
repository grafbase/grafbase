use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

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
    ) -> Result<HeaderMap, PartialGraphqlError> {
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

    async fn authorize_node_post_execution<'a>(
        &self,
        context: &DynHookContext,
        definition: NodeDefinition<'a>,
        nodes: Vec<serde_json::Value>,
        metadata: Option<serde_json::Value>,
    ) -> AuthorizationVerdicts {
        Err(PartialGraphqlError::new(
            "authorize_node_post_execution is not implemented",
            PartialErrorCode::Unauthorized,
        ))
    }

    async fn authorize_parent_edge_post_execution<'a>(
        &self,
        context: &DynHookContext,
        definition: EdgeDefinition<'a>,
        parents: Vec<serde_json::Value>,
        metadata: Option<serde_json::Value>,
    ) -> AuthorizationVerdicts {
        Err(PartialGraphqlError::new(
            "authorize_parent_edge_post_execution is not implemented",
            PartialErrorCode::Unauthorized,
        ))
    }

    async fn authorize_edge_node_post_execution<'a>(
        &self,
        context: &DynHookContext,
        definition: EdgeDefinition<'a>,
        nodes: Vec<serde_json::Value>,
        metadata: Option<serde_json::Value>,
    ) -> AuthorizationVerdicts {
        Err(PartialGraphqlError::new(
            "authorize_edge_node_post_execution is not implemented",
            PartialErrorCode::Unauthorized,
        ))
    }

    async fn authorize_edge_post_execution<'a>(
        &self,
        context: &DynHookContext,
        definition: EdgeDefinition<'a>,
        edges: Vec<(serde_json::Value, Vec<serde_json::Value>)>,
        metadata: Option<serde_json::Value>,
    ) -> AuthorizationVerdicts {
        Err(PartialGraphqlError::new(
            "authorize_edge_post_execution is not implemented",
            PartialErrorCode::Unauthorized,
        ))
    }
}

#[derive(Default)]
pub struct DynHookContext {
    by_type: HashMap<TypeId, Box<dyn Any + Sync + Send>>,
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
        self.by_type.insert(TypeId::of::<T>(), Box::new(value));
    }

    pub fn get(&self, name: &str) -> Option<&String> {
        self.by_name.get(name)
    }

    pub fn insert(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.by_name.insert(name.into(), value.into());
    }
}

impl DynHooks for () {}

impl<T: DynHooks> From<T> for DynamicHooks {
    fn from(hooks: T) -> Self {
        Self::new(hooks)
    }
}

pub struct DynamicHooks(Box<dyn DynHooks>);

impl Default for DynamicHooks {
    fn default() -> Self {
        Self::new(())
    }
}

impl DynamicHooks {
    pub fn new(hooks: impl DynHooks) -> Self {
        Self(Box::new(hooks))
    }
}

impl Hooks for DynamicHooks {
    type Context = DynHookContext;

    async fn on_gateway_request(&self, headers: HeaderMap) -> Result<(Self::Context, HeaderMap), PartialGraphqlError> {
        let mut context = DynHookContext::default();
        let headers = self.0.on_gateway_request(&mut context, headers).await?;
        Ok((context, headers))
    }

    fn authorized(&self) -> &impl AuthorizedHooks<Self::Context> {
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
