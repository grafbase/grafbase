mod consts;
mod cursor;
mod input;
mod normalize;
mod pagination;
mod projection;
mod request;
mod value;

use std::pin::Pin;

use async_runtime::make_send_on_wasm;
use futures_util::Future;
pub use registry_v2::resolvers::atlas_data_api::OperationType;

use super::{ResolvedValue, ResolverContext};
use crate::{ContextExt, ContextField, Error};

type JsonMap = serde_json::Map<String, serde_json::Value>;

pub use registry_v2::resolvers::atlas_data_api::AtlasDataApiResolver;

pub fn resolve<'a>(
    resolver: &'a AtlasDataApiResolver,
    ctx: &'a ContextField<'_>,
    resolver_ctx: &'a ResolverContext<'_>,
) -> Pin<Box<dyn Future<Output = Result<ResolvedValue, Error>> + Send + 'a>> {
    let config = ctx
        .get_mongodb_config(&resolver.directive_name)
        .expect("directive must exist");

    Box::pin(make_send_on_wasm(async move {
        request::execute(ctx, resolver_ctx, config, &resolver.collection, resolver.operation_type).await
    }))
}
