use std::{
    cell::{RefCell, RefMut},
    fmt,
    rc::Rc,
    sync::atomic::{AtomicBool, Ordering},
};

use serde::{
    de::{DeserializeSeed, Visitor},
    Deserializer,
};

use super::{ResponseObjectUpdate, ResponsePart};
use crate::{
    plan::{CollectedField, CollectedSelectionSetId, PlanWalker},
    response::{GraphqlError, ResponseBoundaryItem, ResponseEdge, ResponsePath, ResponseValue},
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

#[derive(Clone)]
pub(crate) struct SeedContext<'ctx>(Rc<SeedContextInner<'ctx>>);

struct SeedContextInner<'ctx> {
    plan: PlanWalker<'ctx>,
    // We could probably avoid the RefCell, but didn't took the time to properly deal with it.
    response_part: RefCell<&'ctx mut ResponsePart>,
    propagating_error: AtomicBool, // using an atomic bool for convenience of fetch_or & fetch_and
    path: RefCell<Vec<ResponseEdge>>,
}

impl<'ctx> SeedContext<'ctx> {
    pub fn new(plan: PlanWalker<'ctx>, response_part: &'ctx mut ResponsePart) -> Self {
        Self(Rc::new(SeedContextInner {
            plan,
            response_part: RefCell::new(response_part),
            propagating_error: AtomicBool::new(false),
            path: RefCell::new(Vec::new()),
        }))
    }

    pub fn create_root_seed(&self, boundary_item: &'ctx ResponseBoundaryItem) -> UpdateSeed<'ctx> {
        UpdateSeed {
            ctx: self.clone(),
            boundary_item,
            id: self.0.plan.collected_selection_set().id(),
        }
    }

    pub fn borrow_mut_response_part(&self) -> RefMut<'_, &'ctx mut ResponsePart> {
        self.0.response_part.borrow_mut()
    }
}

impl<'ctx> SeedContextInner<'ctx> {
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

    fn set_response_path(&self, path: &ResponsePath) {
        let mut current = self.path.borrow_mut();
        current.clear();
        current.extend(path.iter());
    }
}

pub(crate) struct UpdateSeed<'ctx> {
    ctx: SeedContext<'ctx>,
    boundary_item: &'ctx ResponseBoundaryItem,
    id: CollectedSelectionSetId,
}

impl<'de, 'ctx> DeserializeSeed<'de> for UpdateSeed<'ctx> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let ctx = &self.ctx.0;
        ctx.set_response_path(&self.boundary_item.response_path);

        let result = deserializer.deserialize_option(NullableVisitor(
            CollectedSelectionSetSeed::new_from_id(ctx, self.id).fields_seed,
        ));

        let mut response_part = ctx.response_part.borrow_mut();
        match result {
            Ok(Some((_, fields))) => {
                response_part.push_update(ResponseObjectUpdate {
                    id: self.boundary_item.response_object_id,
                    fields,
                });
            }
            Ok(None) => {
                let mut update = ResponseObjectUpdate {
                    id: self.boundary_item.response_object_id,
                    fields: Vec::with_capacity(ctx.plan[self.id].field_ids.len()),
                };
                for field in &ctx.plan[ctx.plan[self.id].field_ids] {
                    if field.wrapping.is_required() {
                        response_part.push_error(GraphqlError {
                            message: ctx.missing_field_error_message(field),
                            path: Some(ctx.response_path().child(field.edge)),
                            ..Default::default()
                        });
                        response_part.push_error_path_to_propagate(self.boundary_item.response_path.clone());
                        return Ok(());
                    } else {
                        update.fields.push((field.edge, ResponseValue::Null));
                    }
                }
                response_part.push_update(update);
            }
            Err(err) => {
                if !ctx.propagating_error.fetch_or(true, Ordering::Relaxed) {
                    response_part.push_error(GraphqlError {
                        message: err.to_string(),
                        path: Some(ctx.response_path()),
                        ..Default::default()
                    });
                }
                response_part.push_error_path_to_propagate(self.boundary_item.response_path.clone());
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
