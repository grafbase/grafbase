use runtime::{bytes::OwnedOrSharedBytes, hooks::GraphqlResponseStatus};
use serde::{
    Deserializer,
    de::{DeserializeSeed, IgnoredAny, SeqAccess, Visitor},
};

use crate::{
    execution::ExecutionError,
    prepare::ConcreteShapeId,
    resolver::graphql::{
        deserialize::{EntitiesDataSeed, EntityErrorPathConverter, GraphqlErrorsSeed, GraphqlResponseSeed},
        request::ResponseIngester,
    },
    response::{GraphqlError, ResponsePartBuilder, SharedResponsePartBuilder},
};

use super::EntityToFetch;

pub(super) struct EntityIngester {
    pub shape_id: ConcreteShapeId,
    pub fetched_entities: Vec<EntityToFetch>,
}

impl ResponseIngester for EntityIngester {
    async fn ingest(
        self,
        http_response: http::Response<OwnedOrSharedBytes>,
        response_part: ResponsePartBuilder<'_>,
    ) -> Result<(GraphqlResponseStatus, ResponsePartBuilder<'_>), ExecutionError> {
        let Self {
            shape_id,
            fetched_entities,
        } = self;

        let part = response_part.into_shared();
        let status = GraphqlResponseSeed::new(
            EntitiesDataSeed::new(EntitiesSeed {
                shape_id,
                response_part: part.clone(),
                fetched_entities: &fetched_entities,
            }),
            GraphqlErrorsSeed::new(
                part.clone(),
                EntityErrorPathConverter::new(part.clone(), |index| {
                    fetched_entities.get(index).map(|entity| entity.id)
                }),
            ),
        )
        .deserialize(&mut sonic_rs::Deserializer::from_slice(http_response.body()))
        .map_err(|err| {
            tracing::error!("Failed to deserialize subgraph response: {}", err);
            GraphqlError::invalid_subgraph_response()
        })?;

        Ok((status, part.unshare().unwrap()))
    }
}

struct EntitiesSeed<'resp, 'parent> {
    shape_id: ConcreteShapeId,
    response_part: SharedResponsePartBuilder<'resp>,
    fetched_entities: &'parent [EntityToFetch],
}

impl<'resp, 'de> DeserializeSeed<'de> for EntitiesSeed<'resp, '_>
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

impl<'resp, 'de> Visitor<'de> for EntitiesSeed<'resp, '_>
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
            shape_id,
            response_part,
            fetched_entities,
        } = self;
        let mut fetched_entities = fetched_entities.iter();
        for EntityToFetch { id, .. } in fetched_entities.by_ref() {
            match seq.next_element_seed(response_part.seed(shape_id, *id)) {
                Ok(Some(())) => continue,
                Ok(None) => {
                    tracing::error!("Received less entities than expected");
                    response_part.borrow_mut().insert_errors(
                        GraphqlError::invalid_subgraph_response(),
                        fetched_entities.by_ref().map(|entity| entity.id),
                    );

                    break;
                }
                Err(err) => {
                    tracing::error!("Subgraph deserialization failed with: {err}");
                    response_part.borrow_mut().insert_errors(
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
            // Not adding any GraphqlError as from the client perspective we have everything.
            tracing::error!("Received more entities than expected");

            // Try discarding the rest of the list, we might be able to use other parts of
            // the response.
            while seq.next_element::<IgnoredAny>()?.is_some() {}
        }

        Ok(())
    }
}
