use super::{ResolverContext, ResolverTrait};
use crate::{Context, Error};

#[non_exhaustive]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum DebugResolver {
    Value { inner: serde_json::Value },
}

#[async_trait::async_trait]
impl ResolverTrait for DebugResolver {
    async fn resolve(
        &self,
        _ctx: &Context<'_>,
        _resolver_ctx: &ResolverContext<'_>,
        _last_resolver_value: Option<&serde_json::Value>,
    ) -> Result<serde_json::Value, Error> {
        #[cfg(feature = "tracing_worker")]
        logworker::info!("", "",);

        match &self {
            Self::Value { inner } => Ok(inner.clone()),
        }
    }
}
