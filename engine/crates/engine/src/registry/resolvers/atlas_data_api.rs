mod consts;
mod cursor;
mod input;
mod normalize;
mod operation;
mod pagination;
mod projection;
mod request;
mod value;

use std::pin::Pin;

use async_runtime::make_send_on_wasm;
use futures_util::Future;
pub use operation::OperationType;

use super::{ResolvedValue, ResolverContext};
use crate::{ContextExt, ContextField, Error};

type JsonMap = serde_json::Map<String, serde_json::Value>;

/// Resolver for the MongoDB Atlas Data API, which is a MongoDB endpoint using
/// HTTP protocol for transfer.
///
/// # Internal documentation
/// https://www.notion.so/grafbase/MongoDB-Connector-b4d134d2dd0f41ef88dd25cf19143be8
#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub struct AtlasDataApiResolver {
    /// The type of operation to execute in the target.
    pub operation_type: OperationType,
    pub directive_name: String,
    pub collection: String,
}

impl AtlasDataApiResolver {
    pub fn resolve<'a>(
        &'a self,
        ctx: &'a ContextField<'_>,
        resolver_ctx: &'a ResolverContext<'_>,
    ) -> Pin<Box<dyn Future<Output = Result<ResolvedValue, Error>> + Send + 'a>> {
        let config = ctx
            .get_mongodb_config(&self.directive_name)
            .expect("directive must exist");

        Box::pin(make_send_on_wasm(async move {
            request::execute(ctx, resolver_ctx, config, &self.collection, self.operation_type).await
        }))
    }
}
