use bytes::Bytes;
use runtime::hooks::GraphqlResponseStatus;

use crate::{
    prepare::RootFieldsShapeId,
    resolver::graphql::{
        deserialize::{EntitiesDataSeed, EntityErrorPathConverter, GraphqlErrorsSeed, GraphqlResponseSeed},
        request::ResponseIngester,
    },
    response::{Deserializable, GraphqlError, ParentObjectSet, ResponsePartBuilder},
};

use super::EntityToFetch;

pub(super) struct EntityIngester {
    pub shape_id: RootFieldsShapeId,
    pub parent_objects: ParentObjectSet,
    pub fetched_entities: Vec<EntityToFetch>,
}

impl ResponseIngester for EntityIngester {
    async fn ingest(
        self,
        result: Result<http::Response<Bytes>, GraphqlError>,
        mut response_part: ResponsePartBuilder<'_>,
    ) -> (Option<GraphqlResponseStatus>, ResponsePartBuilder<'_>) {
        let Self {
            shape_id,
            parent_objects,
            fetched_entities,
        } = self;

        let http_response = match result {
            Ok(http_response) => http_response,
            Err(err) => {
                response_part.insert_error_updates(&parent_objects, shape_id, err);
                return (None, response_part);
            }
        };

        let state = response_part.into_seed_state(shape_id);
        let seed = GraphqlResponseSeed::new(
            EntitiesDataSeed::new(
                state.parent_list_seed(fetched_entities.iter().map(|entity| &parent_objects[entity.id])),
            ),
            GraphqlErrorsSeed::new(
                &state,
                EntityErrorPathConverter(|index: usize| {
                    let id = fetched_entities.get(index).map(|entity| entity.id)?;
                    Some((&parent_objects[id].path).into())
                }),
            ),
        );

        let status = match state.deserialize_data_with(Deserializable::Json(http_response.body().as_ref()), seed) {
            Ok(status) => Some(status),
            Err(err) => {
                if let Some(error) = err {
                    state.insert_error_updates(&parent_objects, error);
                }
                None
            }
        };

        (status, state.into_response_part())
    }
}
