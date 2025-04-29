use std::{borrow::Cow, time::Duration};

use futures::future::join_all;
use runtime::{bytes::OwnedOrSharedBytes, hooks::GraphqlResponseStatus};
use serde::{
    Deserializer,
    de::{DeserializeSeed, Error, IgnoredAny, SeqAccess, Visitor},
};
use serde_json::value::RawValue;

use crate::{
    Runtime,
    execution::{ExecutionContext, ExecutionError, ExecutionResult},
    prepare::ConcreteShapeId,
    resolver::graphql::{
        cache::{CacheFetchEntitiesOutcome, EntityCacheHit, EntityCacheMiss, calculate_cache_ttl},
        deserialize::{EntitiesDataSeed, EntityErrorPathConverter, GraphqlErrorsSeed, GraphqlResponseSeed},
        request::ResponseIngester,
    },
    response::{GraphqlError, ResponsePartBuilder, SharedResponsePartBuilder},
};

pub(super) fn ingest_hits(
    shape_id: ConcreteShapeId,
    hits: Vec<EntityCacheHit>,
    response_part: ResponsePartBuilder<'_>,
) -> ExecutionResult<ResponsePartBuilder<'_>> {
    let response_part = response_part.into_shared();
    for hit in hits {
        response_part
            .seed(shape_id, hit.id)
            .deserialize(&mut sonic_rs::Deserializer::from_slice(&hit.data))
            .map_err(|err| {
                tracing::error!("Failed to deserialize subgraph response: {}", err);
                GraphqlError::invalid_subgraph_response()
            })?;
    }
    Ok(response_part.unshare().unwrap())
}

pub(super) struct PartiallyCachedEntitiesIngester<'ctx, R: Runtime> {
    pub ctx: ExecutionContext<'ctx, R>,
    pub cache_fetch_outcome: CacheFetchEntitiesOutcome,
    pub shape_id: ConcreteShapeId,
    pub subgraph_default_cache_ttl: Option<Duration>,
}

impl<R> ResponseIngester for PartiallyCachedEntitiesIngester<'_, R>
where
    R: Runtime,
{
    async fn ingest(
        self,
        http_response: http::Response<OwnedOrSharedBytes>,
        response_part: ResponsePartBuilder<'_>,
    ) -> Result<(GraphqlResponseStatus, ResponsePartBuilder<'_>), ExecutionError> {
        let Self {
            ctx,
            cache_fetch_outcome: CacheFetchEntitiesOutcome { hits, misses },
            shape_id,
            subgraph_default_cache_ttl,
        } = self;

        // New cache values we should update the cache with if everything went fine. Populated
        // while deserializing.
        let mut cache_updates = Vec::with_capacity(misses.len());
        let (status, response_part) = {
            let response_part = response_part.into_shared();

            for hit in hits {
                response_part
                    .seed(shape_id, hit.id)
                    .deserialize(&mut sonic_rs::Deserializer::from_slice(&hit.data))
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
            let status = GraphqlResponseSeed::new(
                EntitiesDataSeed::new(PartiallyCachedEntitiesSeed {
                    misses,
                    shape_id,
                    response_part: response_part.clone(),
                    cache_updates: &mut cache_updates,
                }),
                GraphqlErrorsSeed::new(
                    response_part.clone(),
                    EntityErrorPathConverter::new(response_part.clone(), |index| index_to_id.get(index).copied()),
                ),
            )
            // We use RawValue underneath, so can't use sonic_rs. RwaValue doesn't do any copies
            // compared to sonic_rs::LazyValue
            .deserialize(&mut serde_json::Deserializer::from_slice(http_response.body()))
            .map_err(|err| {
                tracing::error!("Failed to deserialize subgraph response: {}", err);
                GraphqlError::invalid_subgraph_response()
            })?;
            (status, response_part.unshare().unwrap())
        };

        if status.is_success() {
            if let Some(cache_ttl) = calculate_cache_ttl(status, http_response.headers(), subgraph_default_cache_ttl) {
                let cache = ctx.runtime().entity_cache();
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

        Ok((status, response_part))
    }
}

struct PartiallyCachedEntitiesSeed<'ctx, 'de, 'updates> {
    misses: Vec<EntityCacheMiss>,
    shape_id: ConcreteShapeId,
    response_part: SharedResponsePartBuilder<'ctx>,
    cache_updates: &'updates mut Vec<(String, &'de RawValue)>,
}

impl<'ctx, 'de> DeserializeSeed<'de> for PartiallyCachedEntitiesSeed<'ctx, 'de, '_>
where
    'ctx: 'de,
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)
    }
}

impl<'ctx, 'de> Visitor<'de> for PartiallyCachedEntitiesSeed<'ctx, 'de, '_>
where
    'ctx: 'de,
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
            misses,
            shape_id,
            response_part,
            cache_updates,
        } = self;

        let mut cache_misses = misses.into_iter();

        for EntityCacheMiss { id, key, .. } in cache_misses.by_ref() {
            let raw_value = match seq.next_element::<&RawValue>() {
                Ok(Some(value)) => value,
                Ok(None) => {
                    tracing::error!("Received less entities than expected");
                    response_part.borrow_mut().insert_errors(
                        GraphqlError::invalid_subgraph_response(),
                        cache_misses.by_ref().map(|miss| miss.id),
                    );

                    break;
                }
                Err(err) => {
                    tracing::error!("Subgraph deserialization failed with: {err}");
                    response_part.borrow_mut().insert_errors(
                        GraphqlError::invalid_subgraph_response(),
                        cache_misses.by_ref().map(|miss| miss.id),
                    );

                    // Try discarding the rest of the list, we might be able to use other parts of
                    // the response.
                    while seq.next_element::<IgnoredAny>()?.is_some() {}

                    return Ok(());
                }
            };
            response_part
                .seed(shape_id, id)
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
