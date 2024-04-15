use async_graphql::{EmptySubscription, Schema};
use runtime::cache::Cache;
use tracing::Instrument;

use super::{CacheConfig, RequestContext};

mod error;
mod graphql;

struct AdminContext {
    host_name: String,
    cache: Cache,
}

#[tracing::instrument(skip_all)]
pub async fn handle_graphql_request(
    _ctx: &impl RequestContext,
    cache: Cache,
    cache_config: &CacheConfig,
    request: async_graphql::Request,
) -> async_graphql::Response {
    let schema = Schema::build(graphql::Query, graphql::Mutation::default(), EmptySubscription)
        .data(AdminContext {
            cache,
            host_name: cache_config.host_name.clone(),
        })
        .finish();

    schema
        .execute(request)
        .instrument(tracing::info_span!("admin_request"))
        .await
}
