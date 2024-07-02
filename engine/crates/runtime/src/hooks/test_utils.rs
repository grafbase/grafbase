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
    ) -> Result<HeaderMap, GraphqlError> {
        Ok(headers)
    }

    async fn authorize_edge_pre_execution(
        &self,
        context: &DynHookContext,
        definition: EdgeDefinition<'_>,
        arguments: serde_json::Value,
        metadata: Option<serde_json::Value>,
    ) -> Result<(), GraphqlError> {
        Err("authorize_edge_pre_execution is not implemented".into())
    }

    async fn authorize_node_pre_execution(
        &self,
        context: &DynHookContext,
        definition: NodeDefinition<'_>,
        metadata: Option<serde_json::Value>,
    ) -> Result<(), GraphqlError> {
        Err("authorize_node_pre_execution is not implemented".into())
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

    async fn on_gateway_request(&self, headers: HeaderMap) -> Result<(Self::Context, HeaderMap), GraphqlError> {
        let mut context = DynHookContext::default();
        let headers = self.0.on_gateway_request(&mut context, headers).await?;
        Ok((context, headers))
    }

    async fn authorize_edge_pre_execution<'a>(
        &self,
        context: &Self::Context,
        definition: EdgeDefinition<'a>,
        arguments: impl serde::Serialize + serde::de::Deserializer<'a> + Send,
        metadata: Option<impl serde::Serialize + serde::de::Deserializer<'a> + Send>,
    ) -> Result<(), GraphqlError> {
        self.0
            .authorize_edge_pre_execution(
                context,
                definition,
                serde_json::to_value(&arguments).unwrap(),
                metadata.map(|m| serde_json::to_value(&m).unwrap()),
            )
            .await
    }

    async fn authorize_node_pre_execution<'a>(
        &self,
        context: &Self::Context,
        definition: NodeDefinition<'a>,
        metadata: Option<impl serde::Serialize + serde::de::Deserializer<'a> + Send>,
    ) -> Result<(), GraphqlError> {
        self.0
            .authorize_node_pre_execution(context, definition, metadata.map(|m| serde_json::to_value(&m).unwrap()))
            .await
    }
}
