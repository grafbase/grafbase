use super::ResolverTrait;
use crate::registry::transformers::TransformerTrait;
use crate::registry::{resolvers::ResolverContext, variables::VariableResolveDefinition};
use crate::{Context, Error, Value};
use dynamodb::DynamoDBBatchersData;
use std::hash::Hash;

#[non_exhaustive]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash)]
pub enum DynamoResolver {
    /// A Query based on the PK and the SK
    QueryPKSK {
        pk: VariableResolveDefinition,
        sk: VariableResolveDefinition,
    },
}

#[async_trait::async_trait]
impl ResolverTrait for DynamoResolver {
    async fn resolve(
        &self,
        ctx: &Context<'_>,
        resolver_ctx: &ResolverContext<'_>,
    ) -> Result<Value, Error> {
        let batchers = &ctx.data::<DynamoDBBatchersData>()?.loader;
        match self {
            DynamoResolver::QueryPKSK { pk, sk } => {
                let pk = match pk.param(ctx).expect("can't fail") {
                    Value::String(inner) => inner,
                    _ => {
                        return Err(Error::new("Internal Error: failed to infer key"));
                    }
                };

                let sk = match sk.param(ctx).expect("can't fail") {
                    Value::String(inner) => inner,
                    _ => {
                        return Err(Error::new("Internal Error: failed to infer key"));
                    }
                };

                let dyna = batchers
                    .load_one((pk.clone(), sk))
                    .await?
                    .ok_or_else(|| Error::new("Internal Error: Failed to fetch the node"))?;

                let transformers = resolver_ctx.transforms;

                let mut result = serde_json::to_value(dyna)?;
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
