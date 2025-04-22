use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::Arc,
};

use error::GraphqlError;
use runtime::extension::Data;
use schema::Schema;
use serde::{
    Deserializer,
    de::{DeserializeSeed, IgnoredAny, SeqAccess, Visitor},
};

use crate::{
    Runtime,
    execution::ExecutionContext,
    prepare::{ConcreteShapeId, PreparedOperation},
    response::{InputResponseObjectSet, SubgraphResponseRefMut},
};

use super::{EntitySeed, SeedContext, entity::DeserError};

pub(crate) struct EntitiesSeed<'ctx> {
    schema: &'ctx Schema,
    prepared_operation: &'ctx PreparedOperation,
    subgraph_response: SubgraphResponseRefMut<'ctx>,
    parent_objects: Arc<InputResponseObjectSet>,
    shape_id: ConcreteShapeId,
}
impl<'ctx> EntitiesSeed<'ctx> {
    pub fn new<R: Runtime>(
        ctx: ExecutionContext<'ctx, R>,
        subgraph_response: SubgraphResponseRefMut<'ctx>,
        parent_objects: Arc<InputResponseObjectSet>,
        shape_id: ConcreteShapeId,
    ) -> Self {
        Self {
            schema: ctx.schema(),
            prepared_operation: ctx.operation,
            subgraph_response,
            parent_objects,
            shape_id,
        }
    }

    pub fn ingest(self, result: Result<Data, GraphqlError>) {
        if let Err(err) = ingest(self, result) {
            todo!("Handle error... {err}");
        }
    }
}

fn ingest<Seed: for<'de> DeserializeSeed<'de>>(
    seed: Seed,
    result: Result<Data, GraphqlError>,
) -> Result<(), DeserError> {
    let data = result?;
    match data {
        Data::Json(bytes) => {
            seed.deserialize(&mut sonic_rs::Deserializer::from_slice(&bytes))
                .inspect_err(|err| tracing::error!("Deserialization failure: {err}"))?;
        }
        Data::Cbor(bytes) => {
            seed.deserialize(&mut minicbor_serde::Deserializer::new(&bytes))
                .inspect_err(|err| tracing::error!("Deserialization failure: {err}"))?;
        }
    }
    Ok(())
}

impl<'de> DeserializeSeed<'de> for EntitiesSeed<'_> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)
    }
}

impl<'de> Visitor<'de> for EntitiesSeed<'_> {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a non null entities list")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let Self {
            schema,
            prepared_operation,
            subgraph_response,
            parent_objects,
            shape_id,
        } = self;
        let mut parent_objects = parent_objects.iter_with_id();
        for (id, parent_object) in parent_objects.by_ref() {
            let ctx = Rc::new(SeedContext {
                schema,
                prepared_operation,
                subgraph_response: subgraph_response.clone(),
                bubbling_up_serde_error: Cell::new(false),
                path: RefCell::new(parent_object.path.clone()),
            });
            let entity_seed = EntitySeed {
                ctx: ctx.clone(),
                shape_id,
                id,
            };
            match seq.next_element_seed(entity_seed) {
                Ok(Some(())) => continue,
                Ok(None) => {
                    tracing::error!("Received less entities than expected");
                    subgraph_response.borrow_mut().insert_errors(
                        GraphqlError::invalid_subgraph_response(),
                        parent_objects.by_ref().map(|(id, _)| id),
                    );

                    break;
                }
                Err(err) => {
                    tracing::error!("Subgraph deserialization failed with: {err}");
                    subgraph_response.borrow_mut().insert_errors(
                        GraphqlError::invalid_subgraph_response(),
                        parent_objects.by_ref().map(|(id, _)| id),
                    );

                    return Ok(());
                }
            }
        }

        if seq.next_element::<IgnoredAny>().unwrap_or_default().is_some() {
            // Not adding any GraphqlError as from the client perspective we have everything.
            tracing::error!("Received more entities than expected");

            // Try discarding the rest of the list, we might be able to use other parts of
            // the response.
            while seq.next_element::<IgnoredAny>().unwrap_or_default().is_some() {}
        }

        Ok(())
    }
}
