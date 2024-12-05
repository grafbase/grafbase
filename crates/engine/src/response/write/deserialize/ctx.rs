use std::cell::{Cell, RefCell, RefMut};

use crate::{
    operation::{OperationPlanContext, SolvedOperationContext},
    prepare::PreparedOperation,
    response::{FieldShapeRecord, GraphqlError, ResponseValueId, SubgraphResponseRefMut},
};
use schema::Schema;
use walker::Walk;

pub(super) struct SeedContext<'ctx> {
    pub schema: &'ctx Schema,
    pub operation: &'ctx PreparedOperation,
    pub subgraph_response: SubgraphResponseRefMut<'ctx>,
    pub bubbling_up_serde_error: Cell<bool>,
    pub path: RefCell<Vec<ResponseValueId>>,
}

impl<'ctx> From<&SeedContext<'ctx>> for SolvedOperationContext<'ctx> {
    fn from(ctx: &SeedContext<'ctx>) -> Self {
        SolvedOperationContext {
            schema: ctx.schema,
            operation: &ctx.operation.cached.solved,
        }
    }
}

impl<'ctx> From<&SeedContext<'ctx>> for OperationPlanContext<'ctx> {
    fn from(ctx: &SeedContext<'ctx>) -> Self {
        OperationPlanContext {
            schema: ctx.schema,
            solved_operation: &ctx.operation.cached.solved,
            operation_plan: &ctx.operation.plan,
        }
    }
}

impl SeedContext<'_> {
    pub(super) fn path(&self) -> RefMut<'_, Vec<ResponseValueId>> {
        self.path.borrow_mut()
    }

    pub(super) fn propagate_null(&self) {
        self.subgraph_response.borrow_mut().propagate_null(&self.path())
    }

    pub(super) fn push_field_deserialization_error_if_not_bubbling_up(
        &self,
        field_shape: &FieldShapeRecord,
        continue_bubbling_up: bool,
        message: impl std::fmt::Display,
    ) {
        let is_propagating = self.bubbling_up_serde_error.get();
        self.bubbling_up_serde_error.set(continue_bubbling_up);
        if !is_propagating && field_shape.key.query_position.is_some() {
            tracing::error!("Deserialization failure of subgraph response: {message}");
            let path = self.path();
            let mut resp = self.subgraph_response.borrow_mut();
            resp.propagate_null(&path);
            resp.push_error(
                GraphqlError::invalid_subgraph_response()
                    .with_path(path.as_ref())
                    .with_location(field_shape.id.walk(self).location),
            );
        }
    }
}
