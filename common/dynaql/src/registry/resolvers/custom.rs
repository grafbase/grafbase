use super::{ResolvedValue, ResolverContext, ResolverTrait};

use crate::{Context, Error};
use dynamodb::DynamoDBBatchersData;

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
        // FIXME: We're abusing this abstraction, considering its current naming.
        // It seems, however, that we may want to simply rename this datum type.
        // It'll no longer be concerned exclusively with the database reads/writes
        // but with any operations that are implemented differently on Grafbase.com and local dev.
        let batchers = &ctx.data::<Arc<DynamoDBBatchersData>>()?;
        let custom_resolvers = &batchers.custom_resolvers;
        let value = custom_resolvers
            .invoke(&self.resolver_name, serde_json::json!({}))
            .await?;
        Ok(ResolvedValue::new(Arc::new(value)))
    }
}
