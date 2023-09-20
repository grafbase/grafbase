use std::sync::Arc;

use super::{CacheConfig, RequestContext};
use async_graphql::{EmptySubscription, Schema};
use runtime::cache::Cache;
use tracing::Instrument;

mod error;
mod graphql;

struct AdminContext {
    ray_id: String,
    host_name: String,
    cache: Arc<dyn Cache<Value = engine::Response> + 'static>,
}

#[tracing::instrument(skip_all)]
pub async fn handle_graphql_request(
    ctx: &impl RequestContext,
    cache: &Arc<impl Cache<Value = engine::Response> + 'static>,
    cache_config: &CacheConfig<'_>,
    request: async_graphql::Request,
) -> async_graphql::Response {
    let schema = Schema::build(graphql::Query, graphql::Mutation::default(), EmptySubscription)
        .data(AdminContext {
            cache: Arc::clone(cache) as Arc<dyn Cache<Value = engine::Response> + 'static>,
            ray_id: ctx.ray_id().to_string(),
            host_name: cache_config.host_name.clone(),
        })
        .finish();

    schema
        .execute(request)
        .instrument(tracing::info_span!("admin_request"))
        .await
}
