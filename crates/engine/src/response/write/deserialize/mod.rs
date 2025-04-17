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
use serde::de::DeserializeSeed;
use walker::Walk;

use crate::{
    Runtime,
    execution::ExecutionContext,
    prepare::{ConcreteShapeId, SubgraphField},
    response::{GraphqlError, InputObjectId},
};

use self::r#enum::*;
use ctx::*;
use list::ListSeed;
use scalar::*;

use super::{ObjectUpdate, SubgraphResponseRefMut};

pub(crate) struct UpdateSeed<'ctx> {
    ctx: Rc<SeedContext<'ctx>>,
    shape_id: ConcreteShapeId,
    id: InputObjectId,
}

impl<'ctx> UpdateSeed<'ctx> {
    pub(super) fn new<R: Runtime>(
        ctx: ExecutionContext<'ctx, R>,
        subgraph_response: SubgraphResponseRefMut<'ctx>,
        shape_id: ConcreteShapeId,
        id: InputObjectId,
    ) -> Self {
        let path = RefCell::new(subgraph_response.borrow().input_object_ref(id).path.clone());
        Self {
            ctx: Rc::new(SeedContext {
                schema: ctx.schema(),
                prepared_operation: ctx.operation,
                subgraph_response,
                bubbling_up_serde_error: Cell::new(false),
                path,
            }),
            shape_id,
            id,
        }
    }

    pub fn deserialize_fields(
        self,
        fields: &mut Vec<(SubgraphField<'ctx>, Result<Data, GraphqlError>)>,
    ) -> Result<(), DeserError> {
        let ctx = Rc::clone(&self.ctx);
        self.deserialize(EntityFields { ctx: &ctx, fields })
    }
}

impl<'de> DeserializeSeed<'de> for UpdateSeed<'_> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let UpdateSeed { ctx, shape_id, id } = self;

        let fields_seed = {
            let root_object_ref = Ref::map(ctx.subgraph_response.borrow(), |resp| resp.input_object_ref(id));
            ConcreteShapeFieldsSeed::new(
                &ctx,
                shape_id.walk(ctx.as_ref()),
                root_object_ref.id,
                Some(root_object_ref.definition_id),
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
        ctx.subgraph_response.borrow_mut().insert_update(id, update);

        Ok(())
    }
}
