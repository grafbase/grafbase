mod deserialize;
mod federation;
mod record;
mod request;
mod root_fields;
mod subscription;

pub(crate) use federation::*;
pub(crate) use root_fields::*;

use grafbase_telemetry::gql_response_status::GraphqlResponseStatus;
use headers::HeaderMapExt;
use http::HeaderMap;
use std::time::Duration;

fn calculate_cache_ttl(
    status: GraphqlResponseStatus,
    headers: &HeaderMap,
    subgraph_default_ttl: Option<Duration>,
) -> Option<Duration> {
    let Some(subgraph_default_ttl) = subgraph_default_ttl else {
        // The subgraph_default_ttl is set to None if entity caching is disabled for a subgraph, so
        // we always return None here in that case.
        return None;
    };

    if !status.is_success() {
        return None;
    }

    let Some(cache_control) = headers.typed_get::<headers::CacheControl>() else {
        return Some(subgraph_default_ttl);
    };

    if cache_control.private() || cache_control.no_store() {
        return None;
    }

    let age = headers.typed_get::<headers::Age>().map(|age| age.as_secs());

    let cache_ttl = cache_control
        .max_age()
        .map(|max_age| max_age - Duration::from_secs(age.unwrap_or_default()))
        .unwrap_or(subgraph_default_ttl);

    Some(cache_ttl)
}
