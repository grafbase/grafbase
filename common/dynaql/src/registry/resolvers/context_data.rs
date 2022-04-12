use super::ResolverTrait;
use crate::registry::resolvers::ResolverContext;
use crate::registry::transformers::TransformerTrait;
use crate::{Context, Error, Value};
#[cfg(feature = "tracing_worker")]
use logworker::info;
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
        resolver_ctx: &ResolverContext<'_>,
    ) -> Result<Value, Error> {
        match self {
            ContextDataResolver::Key { key } => {
                let ctx_value = ctx
                    .resolvers_data
                    .read()
                    .map_err(|x| {
                        #[cfg(feature = "tracing_worker")]
                        info!("dynamodb-resolver", "Data {:?}", &x);
                        x
                    })
                    .expect("Error")
                    .get(key)
                    .map(std::clone::Clone::clone)
                    .unwrap_or(Value::Null);

                let transformers = resolver_ctx.transforms;

                let mut result = serde_json::to_value(ctx_value)?;
                // Apply transformers
                if let Some(transformers) = transformers {
                    result = transformers
                        .iter()
                        .try_fold(result, |acc, cur| cur.transform(acc))?;
                }
                Value::from_json(result).map_err(|err| Error::new(err.to_string()))
            }
        }
    }
}
