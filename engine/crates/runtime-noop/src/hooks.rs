use runtime::{
    error::GraphqlError,
    hooks::{HeaderMap, Hooks},
};

#[derive(Clone)]
pub struct HooksNoop;

impl Hooks for HooksNoop {
    type Context = ();

    async fn on_gateway_request(
        &self,
        headers: HeaderMap,
    ) -> Result<(Self::Context, HeaderMap), runtime::error::GraphqlError> {
        Ok(((), headers))
    }

    async fn authorize_edge_pre_execution<'a>(
        &self,
        _: &Self::Context,
        _: runtime::hooks::EdgeDefinition<'a>,
        _: impl serde::Serialize + serde::de::Deserializer<'a> + Send,
        _: impl serde::Serialize + serde::de::Deserializer<'a> + Send,
    ) -> Result<(), runtime::error::GraphqlError> {
        Err(GraphqlError::new(
            "@authorized directive cannot be used, so access was denied",
        ))
    }

    async fn authorize_node_pre_execution<'a>(
        &self,
        _: &Self::Context,
        _: runtime::hooks::NodeDefinition<'a>,
        _: impl serde::Serialize + serde::de::Deserializer<'a> + Send,
    ) -> Result<(), runtime::error::GraphqlError> {
        Err(GraphqlError::new(
            "@authorized directive cannot be used, so access was denied",
        ))
    }
}
