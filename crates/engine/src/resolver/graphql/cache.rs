use futures::future::join_all;
use grafbase_telemetry::graphql::GraphqlResponseStatus;
use headers::HeaderMapExt;
use http::HeaderMap;
use itertools::Itertools;
use runtime::entity_cache::EntityCache;
use serde_json::value::RawValue;
use std::time::Duration;

use crate::{response::InputObjectId, Runtime};

use super::{EntityToFetch, SubgraphContext};

pub(super) fn calculate_cache_ttl(
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

pub(super) async fn fetch_response<'ctx, R: Runtime>(
    ctx: &mut SubgraphContext<'ctx, R>,
    subgraph_headers: &http::HeaderMap,
    subgraph_request_body: &[u8],
) -> Result<ResponseCacheHit, ResponseCacheMiss> {
    // FIXME: handle cache scopes
    let additional_scopes = Vec::new();

    let key = prepare_key_hasher(ctx.endpoint().subgraph_name(), subgraph_headers, &additional_scopes)
        .update(subgraph_request_body)
        .finalize()
        .to_string();

    ctx.engine()
        .runtime
        .entity_cache()
        .get(&key)
        .await
        .inspect_err(|err| tracing::warn!("Failed to read the cache key {key}: {err}"))
        .ok()
        .flatten()
        .map(|data| Ok(ResponseCacheHit { data }))
        .unwrap_or(Err(ResponseCacheMiss { key }))
}

pub(super) struct ResponseCacheHit {
    pub data: Vec<u8>,
}

pub(super) struct ResponseCacheMiss {
    pub key: String,
}

pub(super) async fn fetch_entities<'ctx, R: Runtime>(
    ctx: &mut SubgraphContext<'ctx, R>,
    subgraph_headers: &http::HeaderMap,
    entities_to_fetch: Vec<EntityToFetch>,
) -> CacheFetchEntitiesOutcome {
    let entity_cache = ctx.engine.runtime.entity_cache();

    // let additional_scopes = plan
    //     .cache_scopes()
    //     .map(|scope| match scope {
    //         CacheScope::Authenticated => "authenticated".into(),
    //         CacheScope::RequiresScopes(scopes) => {
    //             let mut hasher = blake3::Hasher::new();
    //             hasher.update(b"requiresScopes");
    //             hasher.update(&scopes.scopes().len().to_le_bytes());
    //             for scope in scopes.scopes() {
    //                 hasher.update(&scope.len().to_le_bytes());
    //                 hasher.update(scope.as_bytes());
    //             }
    //             hasher.finalize().to_hex().to_string()
    //         }
    //     })
    //     .collect::<Vec<_>>();
    // FIXME: handle cache scopes
    let additional_scopes = Vec::new();

    let hasher = prepare_key_hasher(ctx.endpoint().subgraph_name(), subgraph_headers, &additional_scopes);
    let fetches = entities_to_fetch
        .into_iter()
        .map(|EntityToFetch { id, representation }| {
            let key = hasher
                .clone()
                .update(representation.get().as_bytes())
                .finalize()
                .to_string();
            fetch_entity(entity_cache, id, key, representation)
        });

    let (hits, misses) = join_all(fetches).await.into_iter().partition_result();
    CacheFetchEntitiesOutcome { hits, misses }
}

pub(super) struct CacheFetchEntitiesOutcome {
    pub hits: Vec<EntityCacheHit>,
    pub misses: Vec<EntityCacheMiss>,
}

pub(super) struct EntityCacheHit {
    pub id: InputObjectId,
    pub data: Vec<u8>,
}

pub(super) struct EntityCacheMiss {
    pub id: InputObjectId,
    pub key: String,
    pub representation: Box<RawValue>,
}

async fn fetch_entity(
    entity_cache: &dyn EntityCache,
    id: InputObjectId,
    key: String,
    representation: Box<RawValue>,
) -> Result<EntityCacheHit, EntityCacheMiss> {
    let data = entity_cache
        .get(&key)
        .await
        .inspect_err(|err| tracing::warn!("Failed to read the cache key {key}: {err}"))
        .ok()
        .flatten();

    match data {
        Some(data) => Ok(EntityCacheHit { id, data }),
        None => Err(EntityCacheMiss {
            id,
            key,
            representation,
        }),
    }
}

fn prepare_key_hasher(subgraph_name: &str, headers: &HeaderMap, additional_scopes: &[String]) -> blake3::Hasher {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"v1");
    hasher.update(subgraph_name.as_bytes());
    hasher.update(&headers.len().to_le_bytes());
    for (name, value) in headers {
        hasher.update(&name.as_str().len().to_le_bytes());
        hasher.update(name.as_str().as_bytes());
        hasher.update(&value.len().to_le_bytes());
        hasher.update(value.as_bytes());
    }
    hasher.update(&additional_scopes.len().to_le_bytes());
    for scope in additional_scopes {
        hasher.update(&scope.len().to_le_bytes());
        hasher.update(scope.as_bytes());
    }
    hasher
}
