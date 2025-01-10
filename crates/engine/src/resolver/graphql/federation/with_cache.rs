use std::{borrow::Cow, time::Duration};

use futures::future::join_all;
use runtime::{bytes::OwnedOrSharedBytes, hooks::GraphqlResponseStatus};
use serde::{
    de::{DeserializeSeed, Error, IgnoredAny, SeqAccess, Visitor},
    Deserializer,
};
use serde_json::value::RawValue;

use crate::{
    execution::{ExecutionContext, ExecutionError, ExecutionResult},
    resolver::graphql::{
        cache::{calculate_cache_ttl, CacheFetchEntitiesOutcome, EntityCacheHit, EntityCacheMiss},
        deserialize::{EntitiesDataSeed, EntityErrorPathConverter, GraphqlErrorsSeed, GraphqlResponseSeed},
        request::ResponseIngester,
    },
    response::{GraphqlError, SubgraphResponse, SubgraphResponseRefMut},
    Runtime,
};

pub(super) fn ingest_hits(
    ctx: ExecutionContext<'_, impl Runtime>,
    hits: Vec<EntityCacheHit>,
    mut subgraph_response: SubgraphResponse,
) -> ExecutionResult<SubgraphResponse> {
    {
        let subgraph_response = subgraph_response.as_shared_mut();
        for hit in hits {
            subgraph_response
                .seed(&ctx, hit.id)
                .deserialize(&mut serde_json::Deserializer::from_slice(&hit.data))
                .map_err(|err| {
                    tracing::error!("Failed to deserialize subgraph response: {}", err);
                    GraphqlError::invalid_subgraph_response()
                })?;
        }
    }
    Ok(subgraph_response)
}

pub(super) struct PartiallyCachedEntitiesIngester<'ctx, R: Runtime> {
    pub ctx: ExecutionContext<'ctx, R>,
    pub cache_fetch_outcome: CacheFetchEntitiesOutcome,
    pub subgraph_response: SubgraphResponse,
    pub subgraph_default_cache_ttl: Option<Duration>,
}

impl<R> ResponseIngester for PartiallyCachedEntitiesIngester<'_, R>
where
    R: Runtime,
{
    async fn ingest(
        self,
        http_response: http::Response<OwnedOrSharedBytes>,
    ) -> Result<(GraphqlResponseStatus, SubgraphResponse), ExecutionError> {
        let Self {
            ctx,
            cache_fetch_outcome: CacheFetchEntitiesOutcome { hits, misses },
            mut subgraph_response,
            subgraph_default_cache_ttl,
        } = self;

        // New cache values we should update the cache with if everything went fine. Populated
        // while deserializing.
        let mut cache_updates = Vec::with_capacity(misses.len());
        let status = {
            let subgraph_response = subgraph_response.as_shared_mut();

            for hit in hits {
                subgraph_response
                    .seed(&ctx, hit.id)
                    .deserialize(&mut serde_json::Deserializer::from_slice(&hit.data))
                    .map_err(|err| {
                        tracing::error!("Failed to deserialize subgraph response: {}", err);
                        GraphqlError::invalid_subgraph_response()
                    })?;
            }

            // When receiving the subgraph error path, the position in the `_entities` list will
            // match the ordering of the representations we've sent in the variables. And that
            // order is the same as the cache misses. So here we're building a mapping from said
            // position to the InputObjectId which will allow us to generate the full error path on
            // our side.
            let index_to_id = misses.iter().map(|miss| miss.id).collect::<Vec<_>>();
            GraphqlResponseSeed::new(
                EntitiesDataSeed::new(PartiallyCachedEntitiesSeed {
                    ctx,
                    misses,
                    subgraph_response: subgraph_response.clone(),
                    cache_updates: &mut cache_updates,
                }),
                GraphqlErrorsSeed::new(
                    subgraph_response.clone(),
                    EntityErrorPathConverter::new(subgraph_response, |index| index_to_id.get(index).copied()),
                ),
            )
            .deserialize(&mut serde_json::Deserializer::from_slice(http_response.body()))
            .map_err(|err| {
                tracing::error!("Failed to deserialize subgraph response: {}", err);
                GraphqlError::invalid_subgraph_response()
            })?
        };

        if status.is_success() {
            if let Some(cache_ttl) = calculate_cache_ttl(status, http_response.headers(), subgraph_default_cache_ttl) {
                let cache = ctx.engine.runtime.entity_cache();
                join_all(cache_updates.into_iter().map(|(key, value)| async move {
                    cache
                        .put(&key, Cow::Borrowed(value.get().as_bytes()), cache_ttl)
                        .await
                        .inspect_err(|err| tracing::warn!("Failed to write the cache key {key}: {err}"))
                        .ok();
                }))
                .await;
            }
        }

        Ok((status, subgraph_response))
    }
}

struct PartiallyCachedEntitiesSeed<'ctx, 'resp, 'de, 'updates, R: Runtime> {
    ctx: ExecutionContext<'ctx, R>,
    misses: Vec<EntityCacheMiss>,
    subgraph_response: SubgraphResponseRefMut<'resp>,
    cache_updates: &'updates mut Vec<(String, &'de RawValue)>,
}

impl<'ctx, 'resp, 'de, R: Runtime> DeserializeSeed<'de> for PartiallyCachedEntitiesSeed<'ctx, 'resp, 'de, '_, R>
where
    'ctx: 'resp,
    'resp: 'de,
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)
    }
}

impl<'ctx, 'resp, 'de, R: Runtime> Visitor<'de> for PartiallyCachedEntitiesSeed<'ctx, 'resp, 'de, '_, R>
where
    'ctx: 'resp,
    'resp: 'de,
{
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a non null entities list")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let Self {
            ctx,
            misses,
            subgraph_response,
            cache_updates,
        } = self;

        let mut cache_misses = misses.into_iter();

        for EntityCacheMiss { id, key, .. } in cache_misses.by_ref() {
            let raw_value = match seq.next_element::<&RawValue>() {
                Ok(Some(value)) => value,
                Ok(None) => {
                    tracing::error!("Received less entities than expected");
                    subgraph_response.borrow_mut().insert_errors(
                        GraphqlError::invalid_subgraph_response(),
                        cache_misses.by_ref().map(|miss| miss.id),
                    );

                    break;
                }
                Err(err) => {
                    tracing::error!("Subgraph deserialization failed with: {err}");
                    subgraph_response.borrow_mut().insert_errors(
                        GraphqlError::invalid_subgraph_response(),
                        cache_misses.by_ref().map(|miss| miss.id),
                    );

                    // Try discarding the rest of the list, we might be able to use other parts of
                    // the response.
                    while seq.next_element::<IgnoredAny>()?.is_some() {}

                    return Ok(());
                }
            };
            subgraph_response
                .seed(&ctx, id)
                .deserialize(raw_value)
                .map_err(|err| A::Error::custom(err.to_string()))?;
            cache_updates.push((key, raw_value));
        }

        if seq.next_element::<IgnoredAny>().unwrap_or_default().is_some() {
            // Not adding any GraphqlError as from the client perspective we have everything.
            tracing::error!("Received more entities than expected");

            // Try discarding the rest of the list, we might be able to use other parts of
            // the response.
            while seq.next_element::<IgnoredAny>()?.is_some() {}
        }

        Ok(())
    }
}
