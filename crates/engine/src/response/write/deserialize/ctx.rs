use std::{
    borrow::Cow,
    cell::{Cell, RefCell, RefMut},
};

use crate::{
    operation::{OperationPlanContext, SolvedOperationContext},
    prepare::PreparedOperation,
    response::{FieldShapeRecord, GraphqlError, ResponseValueId, ResponseWriter},
    ErrorCode,
};
use schema::Schema;
use walker::Walk;

pub(super) struct SeedContext<'ctx> {
    pub schema: &'ctx Schema,
    pub operation: &'ctx PreparedOperation,
    pub writer: ResponseWriter<'ctx>,
    pub propagating_error: Cell<bool>,
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
        self.writer.propagate_null(&self.path())
    }

    pub(super) fn push_field_serde_error<Message: Into<Cow<'static, str>>>(
        &self,
        field_shape: &FieldShapeRecord,
        continue_propagating: bool,
        message: impl FnOnce() -> Message,
    ) {
        let is_propagating = self.propagating_error.get();
        self.propagating_error.set(continue_propagating);
        if !is_propagating && field_shape.key.query_position.is_some() {
            let path = self.path();
            let error = GraphqlError::new(message().into(), ErrorCode::SubgraphInvalidResponseError)
                .with_path(path.as_ref())
                .with_location(field_shape.id.walk(self).location);
            self.writer.propagate_null(&path);
            self.writer.push_error(error);
        }
    }
}
