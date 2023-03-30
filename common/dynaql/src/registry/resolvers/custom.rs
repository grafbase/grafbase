use super::{ResolvedValue, ResolverContext, ResolverTrait};

use crate::{Context, Error};
use grafbase_runtime::custom_resolvers::{
    CustomResolverRequest, CustomResolverRequestPayload, CustomResolversEngine,
};

use send_wrapper::SendWrapper;

use std::hash::Hash;
use std::sync::Arc;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub struct CustomResolver {
    pub resolver_name: String,
}

#[async_trait::async_trait]
impl ResolverTrait for CustomResolver {
    async fn resolve(
        &self,
        ctx: &Context<'_>,
        _resolver_ctx: &ResolverContext<'_>,
        _last_resolver_value: Option<&ResolvedValue>,
    ) -> Result<ResolvedValue, Error> {
        let custom_resolvers_engine = ctx.data::<CustomResolversEngine>()?;
        let arguments = ctx
            .field()
            .arguments()?
            .into_iter()
            .map(|(name, value)| value.into_json().map(|value| (name.to_string(), value)))
            .collect::<serde_json::Result<_>>()?;
        let future = SendWrapper::new(custom_resolvers_engine.invoke(
            ctx.data()?,
            CustomResolverRequest {
                resolver_name: self.resolver_name.clone(),
                payload: CustomResolverRequestPayload {
                    arguments,
                    parent: None,
                },
            },
        ));
        let value = Box::pin(future).await?;
        Ok(ResolvedValue::new(Arc::new(value.value)))
    }
}
