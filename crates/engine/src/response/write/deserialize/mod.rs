use std::{
    cell::{Cell, RefCell},
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
    response::{ConcreteShapeId, ResponseWriter},
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

use super::ObjectUpdate;

pub(crate) struct UpdateSeed<'ctx> {
    ctx: SeedContext<'ctx>,
    shape_id: ConcreteShapeId,
}

impl<'ctx> UpdateSeed<'ctx> {
    pub(super) fn new<R: Runtime>(
        ctx: ExecutionContext<'ctx, R>,
        shape_id: ConcreteShapeId,
        writer: ResponseWriter<'ctx>,
    ) -> Self {
        let path = RefCell::new(writer.root_object_ref().path.clone());
        Self {
            ctx: SeedContext {
                schema: ctx.schema(),
                operation: ctx.operation,
                writer,
                propagating_error: Cell::new(false),
                path,
            },
            shape_id,
        }
    }
}

impl<'de> DeserializeSeed<'de> for UpdateSeed<'_> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let UpdateSeed { ctx, shape_id } = self;

        let fields_seed = {
            let root_object_ref = ctx.writer.root_object_ref();
            ConcreteShapeFieldsSeed::new(
                &ctx,
                shape_id.walk(&ctx),
                root_object_ref.id,
                Some(root_object_ref.definition_id),
            )
        };

        let update = match deserializer.deserialize_option(NullableVisitor(fields_seed)) {
            Ok(Some((_, fields))) => ObjectUpdate::Fields(fields),
            Ok(None) => ObjectUpdate::None,
            Err(err) => {
                if let Some(field) = shape_id
                    .walk(&ctx)
                    .fields()
                    .find(|field| field.key.query_position.is_some())
                {
                    ctx.push_field_serde_error(&field, false, || err.to_string());
                }
                ObjectUpdate::Error
            }
        };
        ctx.writer.update_root_object(update);

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
