use runtime::{bytes::OwnedOrSharedBytes, hooks::GraphqlResponseStatus};
use serde::{
    de::{DeserializeSeed, IgnoredAny, SeqAccess, Visitor},
    Deserializer,
};

use crate::{
    execution::{ExecutionContext, ExecutionError},
    resolver::graphql::{
        deserialize::{EntitiesDataSeed, EntityErrorPathConverter, GraphqlErrorsSeed, GraphqlResponseSeed},
        request::ResponseIngester,
    },
    response::{GraphqlError, SubgraphResponse, SubgraphResponseRefMut},
    Runtime,
};

use super::EntityToFetch;

pub(super) struct EntityIngester<'ctx, R: Runtime> {
    pub ctx: ExecutionContext<'ctx, R>,
    pub subgraph_response: SubgraphResponse,
    pub fetched_entities: Vec<EntityToFetch>,
}

impl<'ctx, R> ResponseIngester for EntityIngester<'ctx, R>
where
    R: Runtime,
{
    async fn ingest(
        self,
        http_response: http::Response<OwnedOrSharedBytes>,
    ) -> Result<(GraphqlResponseStatus, SubgraphResponse), ExecutionError> {
        let Self {
            ctx,
            mut subgraph_response,
            fetched_entities,
        } = self;

        let status = {
            let subgraph_response = subgraph_response.as_shared_mut();
            GraphqlResponseSeed::new(
                EntitiesDataSeed::new(EntitiesSeed {
                    ctx,
                    subgraph_response: subgraph_response.clone(),
                    fetched_entities: &fetched_entities,
                }),
                GraphqlErrorsSeed::new(
                    subgraph_response.clone(),
                    EntityErrorPathConverter::new(subgraph_response, |index| {
                        fetched_entities.get(index).map(|entity| entity.id)
                    }),
                ),
            )
            .deserialize(&mut serde_json::Deserializer::from_slice(http_response.body()))
            .map_err(|err| {
                tracing::error!("Failed to deserialize subgraph response: {}", err);
                GraphqlError::invalid_subgraph_response()
            })?
        };

        Ok((status, subgraph_response))
    }
}

struct EntitiesSeed<'resp, 'parent, R: Runtime> {
    ctx: ExecutionContext<'resp, R>,
    subgraph_response: SubgraphResponseRefMut<'resp>,
    fetched_entities: &'parent [EntityToFetch],
}

impl<'resp, 'de, R: Runtime> DeserializeSeed<'de> for EntitiesSeed<'resp, '_, R>
where
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

impl<'resp, 'de, R: Runtime> Visitor<'de> for EntitiesSeed<'resp, '_, R>
where
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
            subgraph_response,
            fetched_entities,
        } = self;
        let mut fetched_entities = fetched_entities.iter();
        for EntityToFetch { id, .. } in fetched_entities.by_ref() {
            match seq.next_element_seed(subgraph_response.seed(&ctx, *id)) {
                Ok(Some(())) => continue,
                Ok(None) => {
                    tracing::error!("Received less entities than expected");
                    subgraph_response.borrow_mut().insert_errors(
                        GraphqlError::invalid_subgraph_response(),
                        fetched_entities.by_ref().map(|entity| entity.id),
                    );

                    break;
                }
                Err(err) => {
                    tracing::error!("Subgraph deserialization failed with: {err}");
                    subgraph_response.borrow_mut().insert_errors(
                        GraphqlError::invalid_subgraph_response(),
                        fetched_entities.by_ref().map(|miss| miss.id),
                    );

                    // Try discarding the rest of the list, we might be able to use other parts of
                    // the response.
                    while seq.next_element::<IgnoredAny>()?.is_some() {}

                    return Ok(());
                }
            }
        }

        if seq.next_element::<IgnoredAny>().unwrap_or_default().is_some() {
            tracing::error!("Received more entities than expected");
            subgraph_response
                .borrow_mut()
                .push_error(GraphqlError::invalid_subgraph_response());

            // Try discarding the rest of the list, we might be able to use other parts of
            // the response.
            while seq.next_element::<IgnoredAny>()?.is_some() {}
        }

        Ok(())
    }
}
