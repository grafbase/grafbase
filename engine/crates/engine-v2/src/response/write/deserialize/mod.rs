use std::{cell::RefCell, collections::HashMap, sync::atomic::AtomicBool};

use serde::de::DeserializeSeed;

use super::{ExecutorOutput, ResponseObjectUpdate};
use crate::{
    plan::ExpectedSelectionSet,
    request::PlanWalker,
    response::{GraphqlError, ResponseBoundaryItem},
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

pub struct SeedContext<'a> {
    pub walker: PlanWalker<'a>,
    // We could probably avoid the RefCell, but didn't took the time to properly deal with it.
    pub data: RefCell<&'a mut ExecutorOutput>,
    pub propagating_error: AtomicBool, // using an atomic bool for convenience of fetch_or & fetch_and
}

pub struct UpdateSeed<'a> {
    pub ctx: SeedContext<'a>,
    pub boundary_item: &'a ResponseBoundaryItem,
    pub expected: &'a ExpectedSelectionSet,
}

impl<'de, 'ctx> DeserializeSeed<'de> for UpdateSeed<'ctx> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let result = match self.expected {
            ExpectedSelectionSet::Grouped(expected) => ObjectFieldsSeed {
                ctx: &self.ctx,
                path: &self.boundary_item.response_path,
                expected,
            }
            .deserialize(deserializer),
            ExpectedSelectionSet::Arbitrary(_) => unreachable!("Updating an object means knowing its object id."),
        };
        match result {
            Ok(object) => {
                self.ctx.data.borrow_mut().push_update(ResponseObjectUpdate {
                    id: self.boundary_item.response_object_id,
                    fields: object.fields,
                });
            }
            Err(err) => {
                let mut data = self.ctx.data.borrow_mut();
                data.push_error(GraphqlError {
                    message: err.to_string(),
                    // TODO: should include locations & path of all root fields retrieved by
                    // the plan.
                    locations: vec![],
                    path: Some(self.boundary_item.response_path.clone()),
                    extensions: HashMap::with_capacity(0),
                });
                data.push_error_to_propagate(self.boundary_item.response_path.clone());
            }
        };
        Ok(())
    }
}
