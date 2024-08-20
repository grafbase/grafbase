mod deserialize;
mod federation;
mod record;
mod request;
mod root_fields;
mod subscription;

use std::time::Duration;

pub(crate) use federation::*;
use headers::HeaderMapExt;
pub(crate) use root_fields::*;

use grafbase_telemetry::gql_response_status::GraphqlResponseStatus;
use runtime::bytes::OwnedOrSharedBytes;

fn should_update_cache(status: GraphqlResponseStatus, cache_control: Option<&headers::CacheControl>) -> bool {
    status.is_success()
        && cache_control
            .map(|cache_control| !(cache_control.private() || cache_control.no_store()))
            .unwrap_or(true)
}

fn calculate_cache_ttl(
    http_response: &http::Response<OwnedOrSharedBytes>,
    cache_control: Option<&headers::CacheControl>,
    subgraph_default_ttl: Option<Duration>,
) -> Option<Duration> {
    let Some(subgraph_default_ttl) = subgraph_default_ttl else {
        // The subgraph_default_ttl is set to None if entity caching is disabled for a subgraph, so
        // we always return None here in that case.
        return None;
    };

    let age = http_response
        .headers()
        .typed_get::<headers::Age>()
        .map(|age| age.as_secs());

    let cache_ttl = cache_control
        .and_then(|cache_control| {
            cache_control
                .max_age()
                .map(|max_age| max_age - Duration::from_secs(age.unwrap_or_default()))
        })
        .unwrap_or(subgraph_default_ttl);

    Some(cache_ttl)
}
