#![allow(clippy::derived_hash_with_manual_eq)]
use super::{ResolvedValue, ResolverContext, ResolverTrait};
use crate::{Context, Error};
use std::sync::Arc;

#[non_exhaustive]
#[derive(
    Clone, Debug, serde::Deserialize, serde::Serialize, derivative::Derivative, PartialEq, Eq,
)]
#[derivative(Hash)]
pub enum DebugResolver {
    Value {
        #[derivative(Hash = "ignore")]
        inner: serde_json::Value,
    },
}

#[async_trait::async_trait]
impl ResolverTrait for DebugResolver {
    async fn resolve(
        &self,
        _ctx: &Context<'_>,
        _resolver_ctx: &ResolverContext<'_>,
        _last_resolver_value: Option<&ResolvedValue>,
    ) -> Result<ResolvedValue, Error> {
        #[cfg(feature = "tracing_worker")]
        logworker::info!("", "",);

        match &self {
            Self::Value { inner } => Ok(ResolvedValue::new(Arc::new(inner.clone()))),
        }
    }
}
