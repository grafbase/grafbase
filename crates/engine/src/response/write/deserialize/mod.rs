use std::{
    cell::{Cell, Ref, RefCell},
    fmt,
};

use object::ConcreteShapeFieldsSeed;
use serde::{
    de::{DeserializeSeed, Visitor},
    Deserializer,
};
use walker::Walk;

use crate::{
    execution::ExecutionContext,
    response::{ConcreteShapeId, GraphqlError, InputObjectId},
    Runtime,
};

mod ctx;
mod r#enum;
mod field;
mod key;
mod list;
mod nullable;
mod object;
mod scalar;

use self::r#enum::*;
use ctx::*;
use list::ListSeed;
use nullable::NullableSeed;
use scalar::*;

use super::{ObjectUpdate, SubgraphResponseRefMut};

pub(crate) struct UpdateSeed<'ctx> {
    ctx: SeedContext<'ctx>,
    shape_id: ConcreteShapeId,
    id: InputObjectId,
}

impl<'ctx> UpdateSeed<'ctx> {
    pub(super) fn new<R: Runtime>(
        ctx: ExecutionContext<'ctx, R>,
        subgraph_response: SubgraphResponseRefMut<'ctx>,
        shape_id: ConcreteShapeId,
        id: InputObjectId,
    ) -> Self {
        let path = RefCell::new(subgraph_response.borrow().input_object_ref(id).path.clone());
        Self {
            ctx: SeedContext {
                schema: ctx.schema(),
                operation: ctx.operation,
                subgraph_response,
                bubbling_up_serde_error: Cell::new(false),
                path,
            },
            shape_id,
            id,
        }
    }
}

impl<'de> DeserializeSeed<'de> for UpdateSeed<'_> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let UpdateSeed { ctx, shape_id, id } = self;

        let fields_seed = {
            let root_object_ref = Ref::map(ctx.subgraph_response.borrow(), |resp| resp.input_object_ref(id));
            ConcreteShapeFieldsSeed::new(
                &ctx,
                shape_id.walk(&ctx),
                root_object_ref.id,
                Some(root_object_ref.definition_id),
            )
        };

        let update = match deserializer.deserialize_option(NullableVisitor(fields_seed)) {
            Ok(Some((_, fields))) => ObjectUpdate::Fields(fields),
            Ok(None) => ObjectUpdate::Missing,
            Err(err) => {
                // if we already handled the GraphQL error and are just bubbling up the serde
                // error, we'll just treat it as an empty fields Vec, a no-op, from here on.
                if ctx.bubbling_up_serde_error.get() {
                    ObjectUpdate::Fields(Vec::new())
                } else {
                    tracing::error!("Deserialization failure of subgraph response: {err}");
                    ObjectUpdate::Error(GraphqlError::invalid_subgraph_response())
                }
            }
        };
        ctx.subgraph_response.borrow_mut().insert_update(id, update);

        Ok(())
    }
}

struct NullableVisitor<Seed>(Seed);

impl<'de, Seed> Visitor<'de> for NullableVisitor<Seed>
where
    Seed: DeserializeSeed<'de>,
{
    type Value = Option<Seed::Value>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a nullable object")
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(None)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(None)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        self.0.deserialize(deserializer).map(Some)
    }
}
