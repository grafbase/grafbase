mod batch;
mod ctx;
mod entity;
mod r#enum;
mod field;
mod key;
mod list;
mod object;
mod scalar;

use std::{
    cell::{Cell, Ref, RefCell},
    rc::Rc,
};

use entity::{DeserError, EntityFields};
use object::{ConcreteShapeFieldsSeed, ObjectValue};
use runtime::extension::Data;
use schema::Schema;
use serde::de::DeserializeSeed;
use walker::Walk;

use crate::{
    prepare::{ConcreteShapeId, PreparedOperation, SubgraphField},
    response::{GraphqlError, ParentObjectId},
};

use self::r#enum::*;
pub(crate) use batch::EntitiesSeed;
use ctx::*;
use list::ListSeed;
use scalar::*;

use super::{ObjectUpdate, SharedResponsePartBuilder};

pub(crate) struct EntitySeed<'ctx> {
    ctx: Rc<SeedContext<'ctx>>,
    shape_id: ConcreteShapeId,
    id: ParentObjectId,
}

impl<'ctx> EntitySeed<'ctx> {
    pub(super) fn new(
        response: SharedResponsePartBuilder<'ctx>,
        shape_id: ConcreteShapeId,
        id: ParentObjectId,
    ) -> Self {
        let path = RefCell::new(response.borrow().parent_objects[id].path.clone());
        let schema: &'ctx Schema = response.borrow().schema;
        let prepared_operation: &'ctx PreparedOperation = response.borrow().operation;
        Self {
            ctx: Rc::new(SeedContext {
                schema,
                prepared_operation,
                response,
                bubbling_up_serde_error: Cell::new(false),
                path,
            }),
            shape_id,
            id,
        }
    }

    pub fn deserialize_from_fields(
        self,
        fields: &mut Vec<(SubgraphField<'ctx>, Result<Data, GraphqlError>)>,
    ) -> Result<(), DeserError> {
        let ctx = Rc::clone(&self.ctx);
        self.deserialize(EntityFields { ctx: &ctx, fields })
    }
}

impl<'de> DeserializeSeed<'de> for EntitySeed<'_> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let EntitySeed { ctx, shape_id, id } = self;

        let fields_seed = {
            let parent_object = Ref::map(ctx.response.borrow(), |resp| &resp.parent_objects[id]);
            ConcreteShapeFieldsSeed::new(
                &ctx,
                shape_id.walk(ctx.as_ref()),
                parent_object.id,
                Some(parent_object.definition_id),
            )
        };

        let update = match deserializer.deserialize_any(fields_seed) {
            Ok(ObjectValue::Some { fields, .. }) => ObjectUpdate::Fields(fields),
            Ok(ObjectValue::Null) => ObjectUpdate::Missing,
            // Errors have already been handled.
            Ok(ObjectValue::Unexpected) => ObjectUpdate::Fields(Vec::new()),
            Ok(ObjectValue::Error(error)) => ObjectUpdate::Error(error),
            Err(err) => {
                // if we already handled the GraphQL error and are just bubbling up the serde
                // error, we'll just treat it as an empty fields Vec, a no-op, from here on.
                if ctx.bubbling_up_serde_error.get() {
                    ObjectUpdate::Fields(Vec::new())
                } else {
                    tracing::error!(
                        "Deserialization failure of subgraph response at path '{}': {err}",
                        ctx.display_path()
                    );
                    ObjectUpdate::Error(GraphqlError::invalid_subgraph_response())
                }
            }
        };
        ctx.response.borrow_mut().insert_update(id, update);

        Ok(())
    }
}
