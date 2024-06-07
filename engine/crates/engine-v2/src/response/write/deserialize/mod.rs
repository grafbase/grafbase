use std::{
    cell::RefCell,
    fmt,
    sync::atomic::{AtomicBool, Ordering},
};

use serde::{
    de::{DeserializeSeed, Visitor},
    Deserializer,
};

use crate::{
    plan::{CollectedField, PlanWalker},
    response::{GraphqlError, ResponseEdge, ResponsePath, ResponseWriter},
};

mod field;
mod key;
mod list;
mod nullable;
mod scalar;
mod selection_set;

use field::FieldSeed;
use list::ListSeed;
use nullable::NullableSeed;
use scalar::*;
use selection_set::*;

pub struct SeedContext<'ctx> {
    plan: PlanWalker<'ctx>,
    writer: ResponseWriter<'ctx>,
    propagating_error: AtomicBool, // using an atomic bool for convenience of fetch_or & fetch_and
    path: RefCell<Vec<ResponseEdge>>,
}

impl<'ctx> SeedContext<'ctx> {
    pub fn new(plan: PlanWalker<'ctx>, writer: ResponseWriter<'ctx>) -> Self {
        let path = RefCell::new(writer.root_path().iter().copied().collect());
        Self {
            plan,
            writer,
            propagating_error: AtomicBool::new(false),
            path,
        }
    }
}

impl<'ctx> SeedContext<'ctx> {
    fn missing_field_error_message(&self, collected_field: &CollectedField) -> String {
        let field = &self.plan[collected_field.id];
        let response_keys = self.plan.response_keys();
        if field.response_key() == collected_field.expected_key.into() {
            format!(
                "Error decoding response from upstream: Missing required field named '{}'",
                &response_keys[collected_field.expected_key]
            )
        } else {
            format!(
                "Error decoding response from upstream: Missing required field named '{}' (expected: '{}')",
                &response_keys[field.response_key()],
                &response_keys[collected_field.expected_key]
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
        let selection_set_id = ctx.plan.collected_selection_set().id();
        let result = deserializer.deserialize_option(NullableVisitor(
            CollectedSelectionSetSeed::new_from_id(&ctx, selection_set_id).fields_seed,
        ));

        match result {
            Ok(Some((_, fields))) => {
                ctx.writer.update_root_object_with(fields);
            }
            // Not writing any data is handled at the Coordinator level in all cases, so we can
            // just skip it here.
            Ok(None) => {}
            Err(err) => {
                if !ctx.propagating_error.fetch_or(true, Ordering::Relaxed) {
                    ctx.writer.propagate_error(GraphqlError {
                        message: err.to_string(),
                        path: Some(ctx.response_path()),
                        ..Default::default()
                    });
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
