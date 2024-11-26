use std::{
    cell::{Cell, RefCell},
    fmt,
};

use object::ConcreteShapeSeed;
use serde::{
    de::{DeserializeSeed, Visitor},
    Deserializer,
};

use crate::{
    execution::ExecutionContext,
    response::{ConcreteShapeId, ErrorCode, GraphqlError, ResponseWriter},
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
        let path = RefCell::new(writer.root_path().iter().copied().collect());
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

impl<'de, 'ctx> DeserializeSeed<'de> for UpdateSeed<'ctx> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let UpdateSeed { ctx, shape_id } = self;
        let result = deserializer.deserialize_option(NullableVisitor(
            ConcreteShapeSeed::new(&ctx, shape_id).into_fields_seed(),
        ));

        match result {
            Ok(Some((_, fields))) => {
                ctx.writer.update_root_object_with(fields);
            }
            // Not writing any data is handled at the Coordinator level in all cases, so we can
            // just skip it here.
            Ok(None) => {}
            Err(err) => {
                if ctx.should_create_new_graphql_error() {
                    ctx.writer.propagate_error(
                        GraphqlError::new(err.to_string(), ErrorCode::SubgraphInvalidResponseError)
                            .with_path(ctx.response_path()),
                    );
                } else {
                    ctx.writer.continue_error_propagation();
                }
            }
        }
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
