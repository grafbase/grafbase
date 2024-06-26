#[cfg(feature = "test-utils")]
mod test_utils;

#[cfg(feature = "test-utils")]
pub use test_utils::*;

use std::future::Future;

pub use http::HeaderMap;

use crate::error::GraphqlError;

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

pub trait Hooks: Send + Sync + 'static {
    type Context: Send + Sync + 'static;

    fn on_gateway_request(
        &self,
        headers: HeaderMap,
    ) -> impl Future<Output = Result<(Self::Context, HeaderMap), GraphqlError>> + Send;

    fn authorize_edge_pre_execution<'a>(
        &self,
        context: &Self::Context,
        definition: EdgeDefinition<'a>,
        arguments: impl serde::Serialize + serde::de::Deserializer<'a> + Send,
        metadata: impl serde::Serialize + serde::de::Deserializer<'a> + Send,
    ) -> impl Future<Output = Result<(), GraphqlError>> + Send;

    fn authorize_node_pre_execution<'a>(
        &self,
        context: &Self::Context,
        definition: NodeDefinition<'a>,
        metadata: impl serde::Serialize + serde::de::Deserializer<'a> + Send,
    ) -> impl Future<Output = Result<(), GraphqlError>> + Send;
}
