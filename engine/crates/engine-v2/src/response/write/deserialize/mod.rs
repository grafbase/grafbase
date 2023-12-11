use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap},
    fmt,
    sync::atomic::{AtomicBool, Ordering},
};

use serde::{
    de::{DeserializeSeed, Visitor},
    Deserializer,
};

use super::{ExecutorOutput, ResponseObjectUpdate};
use crate::{
    plan::{Attribution, CollectedSelectionSet, ConcreteField, Expectations},
    request::PlanWalker,
    response::{GraphqlError, ResponseBoundaryItem, ResponseObject, ResponseValue},
};

mod field;
mod list;
mod nullable;
mod scalar;
mod selection_set;

use field::FieldSeed;
use list::ListSeed;
use nullable::NullableSeed;
use scalar::*;
use selection_set::*;

pub(crate) struct SeedContext<'a> {
    pub walker: PlanWalker<'a>,
    pub expectations: &'a Expectations,
    pub attribution: &'a Attribution,
    // We could probably avoid the RefCell, but didn't took the time to properly deal with it.
    pub data: RefCell<&'a mut ExecutorOutput>,
    pub propagating_error: AtomicBool, // using an atomic bool for convenience of fetch_or & fetch_and
}

impl<'a> SeedContext<'a> {
    pub fn missing_field_error_message(&self, field: &ConcreteField) -> String {
        let missing_key = field
            .definition_id
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

pub(crate) struct UpdateSeed<'a> {
    pub ctx: SeedContext<'a>,
    pub boundary_item: &'a ResponseBoundaryItem,
    pub expected: &'a CollectedSelectionSet,
}

impl<'de, 'a> DeserializeSeed<'de> for UpdateSeed<'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let result = deserializer.deserialize_option(NullableVisitor(CollectedFieldsSeed {
            ctx: &self.ctx,
            path: &self.boundary_item.response_path,
            expected: self.expected,
        }));
        match result {
            Ok(Some(object)) => {
                self.ctx.data.borrow_mut().push_update(ResponseObjectUpdate {
                    id: self.boundary_item.response_object_id,
                    fields: object.fields,
                });
            }
            Ok(None) => {
                let mut update = ResponseObjectUpdate {
                    id: self.boundary_item.response_object_id,
                    fields: BTreeMap::new(),
                };
                let mut data = self.ctx.data.borrow_mut();
                for field in &self.expected.fields {
                    if field.wrapping.is_required() {
                        self.ctx.data.borrow_mut().push_error(GraphqlError {
                            message: self.ctx.missing_field_error_message(field),
                            path: Some(self.boundary_item.response_path.child(field.edge)),
                            ..Default::default()
                        });
                        data.push_error_to_propagate(self.boundary_item.response_path.clone());
                        return Ok(());
                    } else {
                        update.fields.insert(field.edge, ResponseValue::Null);
                    }
                }
                self.ctx.data.borrow_mut().push_update(update);
            }
            Err(err) => {
                let mut data = self.ctx.data.borrow_mut();
                if !self.ctx.propagating_error.fetch_or(true, Ordering::Relaxed) {
                    data.push_error(GraphqlError {
                        message: err.to_string(),
                        locations: vec![],
                        path: Some(self.boundary_item.response_path.clone()),
                        extensions: HashMap::with_capacity(0),
                    });
                }
                data.push_error_to_propagate(self.boundary_item.response_path.clone());
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
