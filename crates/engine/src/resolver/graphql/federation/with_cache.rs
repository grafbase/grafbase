use std::{borrow::Cow, time::Duration};

use bytes::Bytes;
use futures::future::join_all;
use grafbase_telemetry::graphql::GraphqlResponseStatus;
use serde::{
    Deserializer,
    de::{DeserializeSeed, IgnoredAny, SeqAccess, Visitor},
};
use serde_json::value::RawValue;

use crate::{
    Runtime,
    execution::ExecutionContext,
    prepare::RootFieldsShapeId,
    resolver::graphql::{
        cache::{CacheFetchEntitiesOutcome, EntityCacheHit, EntityCacheMiss, calculate_cache_ttl},
        deserialize::{EntitiesDataSeed, EntityErrorPathConverter, GraphqlErrorsSeed, GraphqlResponseSeed},
        request::ResponseIngester,
    },
    response::{Deserializable, GraphqlError, ParentObjectSet, ResponsePartBuilder, SeedState},
};

pub(super) fn ingest_hits<'parent>(
    state: &SeedState<'_, 'parent>,
    parent_objects: &'parent ParentObjectSet,
    hits: Vec<EntityCacheHit>,
) {
    for hit in hits {
        if let Err(error) = state
            .parent_seed(&parent_objects[hit.id])
            .deserialize(&mut sonic_rs::Deserializer::from_slice(&hit.data))
        {
            tracing::error!("Deserialization failure: {error}");
            state.insert_error_update(&parent_objects[hit.id], [GraphqlError::invalid_subgraph_response()]);
        }
    }
}

pub(super) struct PartiallyCachedEntitiesIngester<'ctx, R: Runtime> {
    pub ctx: ExecutionContext<'ctx, R>,
    pub parent_objects: ParentObjectSet,
    pub cache_fetch_outcome: CacheFetchEntitiesOutcome,
    pub shape_id: RootFieldsShapeId,
    pub subgraph_default_cache_ttl: Option<Duration>,
}

impl<R> ResponseIngester for PartiallyCachedEntitiesIngester<'_, R>
where
    R: Runtime,
{
    async fn ingest(
        self,
        result: Result<http::Response<Bytes>, GraphqlError>,
        mut response_part: ResponsePartBuilder<'_>,
    ) -> (Option<GraphqlResponseStatus>, ResponsePartBuilder<'_>) {
        let Self {
            ctx,
            parent_objects,
            cache_fetch_outcome: CacheFetchEntitiesOutcome { hits, misses },
            shape_id,
            subgraph_default_cache_ttl,
        } = self;

        let http_response = match result {
            Ok(http_response) => http_response,
            Err(err) => {
                response_part.insert_error_updates(&parent_objects, shape_id, [err]);
                return (None, response_part);
            }
        };

        // New cache values we should update the cache with if everything went fine. Populated
        // while deserializing.
        let mut cache_updates = Vec::with_capacity(misses.len());
        let (status, response_part) = {
            let state = response_part.into_seed_state(shape_id);

            ingest_hits(&state, &parent_objects, hits);

            // When receiving the subgraph error path, the position in the `_entities` list will
            // match the ordering of the representations we've sent in the variables. And that
            // order is the same as the cache misses. So here we're building a mapping from said
            // position to the InputObjectId which will allow us to generate the full error path on
            // our side.
            let index_to_id = misses.iter().map(|miss| miss.id).collect::<Vec<_>>();
            let mut cache_misses = misses.into_iter();
            let seed = GraphqlResponseSeed::new(
                EntitiesDataSeed::new(PartiallyCachedEntitiesSeed {
                    state: &state,
                    parent_objects: &parent_objects,
                    cache_misses: &mut cache_misses,
                    cache_updates: &mut cache_updates,
                }),
                GraphqlErrorsSeed::new(
                    &state,
                    EntityErrorPathConverter(|index: usize| {
                        let id = index_to_id.get(index).copied()?;
                        Some((&parent_objects[id].path).into())
                    }),
                ),
            );

            // We use RawValue underneath, so can't use sonic_rs. RawValue doesn't do any copies
            // compared to sonic_rs::LazyValue
            let status = match state
                .deserialize_data_with(Deserializable::JsonWithRawValues(http_response.body().as_ref()), seed)
            {
                Ok(status) => Some(status),
                Err(err) => {
                    if let Some(error) = err {
                        state.insert_error_updates(cache_misses.map(|miss| &parent_objects[miss.id]), [error]);
                    }
                    None
                }
            };

            (status, state.into_response_part())
        };

        if let Some(status) = status.filter(|s| s.is_success()) {
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
        (status, response_part)
    }
}

struct PartiallyCachedEntitiesSeed<'ctx, 'parent, 'state, 'de> {
    state: &'state SeedState<'ctx, 'parent>,
    parent_objects: &'parent ParentObjectSet,
    cache_misses: &'state mut std::vec::IntoIter<EntityCacheMiss>,
    cache_updates: &'state mut Vec<(String, &'de RawValue)>,
}

impl<'de> DeserializeSeed<'de> for PartiallyCachedEntitiesSeed<'_, '_, '_, 'de> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)
    }
}

impl<'de> Visitor<'de> for PartiallyCachedEntitiesSeed<'_, '_, '_, 'de> {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a non null entities list")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let Self {
            state,
            parent_objects,
            cache_misses,
            cache_updates,
        } = self;

        let mut result = Ok(());

        for EntityCacheMiss { id, key, .. } in cache_misses.by_ref() {
            let parent_object = &parent_objects[id];
            let raw_value = match seq.next_element::<&RawValue>() {
                Ok(Some(value)) => value,
                Ok(None) => {
                    tracing::error!("Received less entities than expected");
                    state.insert_error_update(parent_object, [GraphqlError::invalid_subgraph_response()]);

                    break;
                }
                Err(err) => {
                    match state.bubbling_up_deser_error.replace(true) {
                        true => state.insert_propagated_empty_update(parent_object),
                        false => {
                            tracing::error!(
                                "Deserialization failure of subgraph response at path '{}': {err}",
                                state.display_path()
                            );
                            state.insert_error_update(parent_object, [GraphqlError::invalid_subgraph_response()]);
                        }
                    }

                    result = Err(err);
                    break;
                }
            };

            if let Err(err) = state.parent_seed(parent_object).deserialize(raw_value) {
                match state.bubbling_up_deser_error.replace(true) {
                    true => state.insert_propagated_empty_update(parent_object),
                    false => {
                        tracing::error!(
                            "Deserialization failure of subgraph response at path '{}': {err}",
                            state.display_path()
                        );
                        state.insert_error_update(parent_object, [GraphqlError::invalid_subgraph_response()]);
                    }
                }

                use serde::de::Error;
                result = Err(A::Error::custom(""));
                break;
            }

            cache_updates.push((key, raw_value));
        }

        if cache_misses.len() > 0 {
            state.insert_empty_updates(cache_misses.map(|EntityCacheMiss { id, .. }| &parent_objects[id]));
        }

        // If de-serialization didn't fail, we finish consuming the sequence if there is anything
        // left.
        if result.is_ok() && seq.next_element::<IgnoredAny>()?.is_some() {
            // Not adding any GraphqlError as from the client perspective we have everything.
            tracing::error!("Received more entities than expected");

            // Try discarding the rest of the list, we might be able to use other parts of
            // the response.
            while seq.next_element::<IgnoredAny>().unwrap_or_default().is_some() {}
        }

        result
    }
}
