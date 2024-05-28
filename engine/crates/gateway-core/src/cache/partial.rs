use common_types::auth::ExecutionAuth;
use futures_util::future::join_all;
use partial_caching::CachingPlan;
use runtime::{cache::Cache, context::RequestContext};

pub async fn partial_caching_execution(
    plan: CachingPlan,
    cache: &Cache,
    auth: &ExecutionAuth,
    request: engine::Request,
    ctx: &impl RequestContext,
) {
    let mut fetch_phase = plan.start_fetch_phase(auth, ctx.headers(), &request.variables);
    let cache_keys = fetch_phase.cache_keys();

    let cache_fetches = cache_keys.iter().map(|key| {
        let key = cache.build_key(&key.to_string());
        async move { cache.get_json::<serde_json::Value>(&key).await }
    });

    for (fetch_result, key) in join_all(cache_fetches).await.into_iter().zip(cache_keys) {
        match fetch_result {
            Ok(entry) => fetch_phase.record_cache_entry(&key, entry),
            Err(error) => {
                // We basically just log and then pretend this is a miss
                tracing::warn!("error when fetching from cache: {error}");
            }
        }
    }

    todo!("finish this in a future PR")
}
