use super::ResolverTrait;
use crate::registry::resolvers::ResolverContext;
use crate::{Context, Error, Value};
use std::hash::Hash;

#[non_exhaustive]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash)]
pub enum ContextDataResolver {
    /// Key based Resolver for ResolverContext
    Key { key: String },
}

#[async_trait::async_trait]
impl ResolverTrait for ContextDataResolver {
    async fn resolve(
        &self,
        ctx: &Context<'_>,
        _resolver_ctx: &ResolverContext<'_>,
    ) -> Result<serde_json::Value, Error> {
        match self {
            ContextDataResolver::Key { key } => {
                let ctx_value = ctx
                    .resolvers_data
                    .read()
                    .expect("Error")
                    .get(key)
                    .map(std::clone::Clone::clone)
                    .unwrap_or(Value::Null);
                Ok(serde_json::to_value(ctx_value)?)
            }
        }
    }
}
