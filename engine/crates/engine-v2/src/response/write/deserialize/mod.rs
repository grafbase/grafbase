use std::{cell::RefCell, collections::HashMap, sync::atomic::AtomicBool};

use schema::SchemaWalker;
use serde::de::DeserializeSeed;

use super::{ResponseObjectUpdate, ResponsePartBuilder};
use crate::{
    plan::ExpectedSelectionSet,
    request::{BoundAnyFieldDefinitionId, BoundAnyFieldDefinitionWalker, Operation},
    response::{GraphqlError, ResponseObjectRoot},
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
    pub schema_walker: SchemaWalker<'a, ()>,
    pub operation: &'a Operation,
    // We could probably avoid the RefCell, but didn't took the time to properly deal with it.
    pub data: RefCell<&'a mut ResponsePartBuilder>,
    pub propagating_error: AtomicBool, // using an atomic bool for convenience of fetch_or & fetch_and
}

impl<'a> SeedContext<'a> {
    fn walk(&self, definition_id: BoundAnyFieldDefinitionId) -> BoundAnyFieldDefinitionWalker<'a> {
        self.operation.walk_definition(self.schema_walker, definition_id)
    }
}

pub struct UpdateSeed<'a> {
    pub ctx: SeedContext<'a>,
    pub root: ResponseObjectRoot,
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
                path: &self.root.path,
                expected,
            }
            .deserialize(deserializer),
            ExpectedSelectionSet::Arbitrary(_) => unreachable!("Updating an object means knowing its object id."),
        };
        match result {
            Ok(object) => {
                self.ctx.data.borrow_mut().push_update(ResponseObjectUpdate {
                    id: self.root.id,
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
                    path: Some(self.root.path.clone()),
                    extensions: HashMap::with_capacity(0),
                });
                data.push_error_to_propagate(self.root.path.clone());
            }
        };
        Ok(())
    }
}
