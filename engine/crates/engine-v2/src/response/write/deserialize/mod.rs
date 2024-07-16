use std::{
    cell::{Cell, RefCell},
    fmt,
};

use object::ConcreteObjectSeed;
use serde::{
    de::{DeserializeSeed, Visitor},
    Deserializer,
};

use crate::{
    execution::{ExecutableOperation, PlanWalker},
    response::{ErrorCode, FieldShape, GraphqlError, ResponseEdge, ResponsePath, ResponseWriter},
};

mod field;
mod key;
mod list;
mod nullable;
mod object;
mod scalar;

use list::ListSeed;
use nullable::NullableSeed;
use scalar::*;

pub struct SeedContext<'ctx> {
    plan: PlanWalker<'ctx, (), ()>,
    operation: &'ctx ExecutableOperation,
    writer: ResponseWriter<'ctx>,
    propagating_error: Cell<bool>,
    path: RefCell<Vec<ResponseEdge>>,
}

impl<'ctx> SeedContext<'ctx> {
    pub fn new(plan: PlanWalker<'ctx>, writer: ResponseWriter<'ctx>) -> Self {
        let path = RefCell::new(writer.root_path().iter().copied().collect());
        Self {
            operation: plan.operation(),
            plan,
            writer,
            propagating_error: Cell::new(false),
            path,
        }
    }
}

impl<'ctx> SeedContext<'ctx> {
    fn missing_field_error_message(&self, shape: &FieldShape) -> String {
        let field = &self.plan.walk_with(shape.id, shape.definition_id);
        let response_keys = self.plan.response_keys();
        if field.response_key() == shape.expected_key.into() {
            format!(
                "Error decoding response from upstream: Missing required field named '{}'",
                &response_keys[shape.expected_key]
            )
        } else {
            format!(
                "Error decoding response from upstream: Missing required field named '{}' (expected: '{}')",
                &response_keys[field.response_key()],
                &response_keys[shape.expected_key]
            )
        }
    }

    fn push_edge(&self, edge: ResponseEdge) {
        self.path.borrow_mut().push(edge);
    }

    fn pop_edge(&self) {
        self.path.borrow_mut().pop();
    }

    fn response_path(&self) -> ResponsePath {
        ResponsePath::from(self.path.borrow().clone())
    }

    fn should_create_new_graphql_error(&self) -> bool {
        let is_propagating = self.propagating_error.get();
        self.propagating_error.set(true);
        !is_propagating
    }

    fn stop_propagating_and_should_create_new_graphql_error(&self) -> bool {
        let is_propagating = self.propagating_error.get();
        self.propagating_error.set(false);
        !is_propagating
    }

    fn propagate_error<V, E: serde::de::Error>(&self) -> Result<V, E> {
        self.propagating_error.set(true);
        Err(serde::de::Error::custom(""))
    }
}

pub(crate) struct UpdateSeed<'ctx> {
    ctx: SeedContext<'ctx>,
}

impl<'ctx> UpdateSeed<'ctx> {
    pub(super) fn new(plan: PlanWalker<'ctx>, writer: ResponseWriter<'ctx>) -> Self {
        Self {
            ctx: SeedContext::new(plan, writer),
        }
    }
}

impl<'de, 'ctx> DeserializeSeed<'de> for UpdateSeed<'ctx> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let UpdateSeed { ctx } = self;
        let result = deserializer.deserialize_option(NullableVisitor(
            ConcreteObjectSeed::new(&ctx, ctx.plan.logical_plan().response_blueprint().concrete_shape_id)
                .into_fields_seed(),
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
