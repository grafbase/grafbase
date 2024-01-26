use std::{
    cell::{RefCell, RefMut},
    collections::BTreeMap,
    fmt,
    rc::Rc,
    sync::atomic::{AtomicBool, Ordering},
};

use serde::{
    de::{DeserializeSeed, Visitor},
    Deserializer,
};

use super::{ExecutorOutput, ResponseObjectUpdate};
use crate::{
    plan::{Attribution, CollectedSelectionSet, ConcreteField, Expectations, PlanOutput},
    request::PlanWalker,
    response::{GraphqlError, ResponseBoundaryItem, ResponseObject, ResponseValue},
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
    walker: PlanWalker<'ctx>,
    expectations: &'ctx Expectations,
    attribution: &'ctx Attribution,
    // We could probably avoid the RefCell, but didn't took the time to properly deal with it.
    data: RefCell<&'ctx mut ExecutorOutput>,
    propagating_error: AtomicBool, // using an atomic bool for convenience of fetch_or & fetch_and
}

impl<'ctx> SeedContext<'ctx> {
    pub fn new(walker: PlanWalker<'ctx>, output: &'ctx mut ExecutorOutput, plan_output: &'ctx PlanOutput) -> Self {
        Self(Rc::new(SeedContextInner {
            walker,
            expectations: &plan_output.expectations,
            attribution: &plan_output.attribution,
            data: RefCell::new(output),
            propagating_error: AtomicBool::new(false),
        }))
    }

    pub fn create_root_seed(&self, boundary_item: &'ctx ResponseBoundaryItem) -> UpdateSeed<'ctx> {
        UpdateSeed {
            ctx: self.clone(),
            boundary_item,
            expected: &self.0.expectations.root_selection_set,
        }
    }

    pub fn borrow_mut_output(&self) -> RefMut<'_, &'ctx mut ExecutorOutput> {
        self.0.data.borrow_mut()
    }
}

impl<'ctx> SeedContextInner<'ctx> {
    fn missing_field_error_message(&self, field: &ConcreteField) -> String {
        let missing_key = field
            .bound_field_id
            .map(|id| self.walker.walk(id).response_key_str())
            .unwrap_or(&field.expected_key);

        if field.expected_key == missing_key {
            format!("Upstream response error: Missing required field named '{missing_key}'")
        } else {
            format!(
                "Upstream response error: Missing required field named '{missing_key}' (expected: '{}')",
                field.expected_key
            )
        }
    }
}

pub(crate) struct UpdateSeed<'ctx> {
    ctx: SeedContext<'ctx>,
    boundary_item: &'ctx ResponseBoundaryItem,
    expected: &'ctx CollectedSelectionSet,
}

impl<'de, 'ctx> DeserializeSeed<'de> for UpdateSeed<'ctx> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let ctx = &self.ctx.0;
        let result = deserializer.deserialize_option(NullableVisitor(CollectedFieldsSeed {
            ctx,
            path: &self.boundary_item.response_path,
            expected: self.expected,
        }));

        let mut data = ctx.data.borrow_mut();
        match result {
            Ok(Some(object)) => {
                data.push_update(ResponseObjectUpdate {
                    id: self.boundary_item.response_object_id,
                    fields: object.fields,
                });
            }
            Ok(None) => {
                let mut update = ResponseObjectUpdate {
                    id: self.boundary_item.response_object_id,
                    fields: BTreeMap::new(),
                };
                for field in &self.expected.fields {
                    if field.wrapping.is_required() {
                        data.push_error(GraphqlError {
                            message: ctx.missing_field_error_message(field),
                            path: Some(self.boundary_item.response_path.child(field.edge)),
                            ..Default::default()
                        });
                        data.push_error_path_to_propagate(self.boundary_item.response_path.clone());
                        return Ok(());
                    } else {
                        update.fields.insert(field.edge, ResponseValue::Null);
                    }
                }
                data.push_update(update);
            }
            Err(err) => {
                if !ctx.propagating_error.fetch_or(true, Ordering::Relaxed) {
                    data.push_error(GraphqlError {
                        message: err.to_string(),
                        path: Some(self.boundary_item.response_path.clone()),
                        ..Default::default()
                    });
                }
                data.push_error_path_to_propagate(self.boundary_item.response_path.clone());
            }
        }
        Ok(())
    }
}

struct NullableVisitor<'ctx, 'parent>(CollectedFieldsSeed<'ctx, 'parent>);

impl<'de, 'ctx, 'parent> Visitor<'de> for NullableVisitor<'ctx, 'parent> {
    type Value = Option<ResponseObject>;

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
