use std::cell::{Cell, RefCell};

use crate::{
    operation::{OperationPlanContext, SolvedOperationContext},
    prepare::PreparedOperation,
    response::{FieldShape, ResponseEdge, ResponsePath, ResponseWriter},
};
use schema::Schema;
use walker::Walk;

pub(super) struct SeedContext<'ctx> {
    pub schema: &'ctx Schema,
    pub operation: &'ctx PreparedOperation,
    pub writer: ResponseWriter<'ctx>,
    pub propagating_error: Cell<bool>,
    pub path: RefCell<Vec<ResponseEdge>>,
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

impl<'ctx> SeedContext<'ctx> {
    pub(super) fn missing_field_error_message(&self, field_shape: FieldShape<'ctx>) -> String {
        let field = field_shape.id.walk(self);
        let response_keys = &self.operation.cached.solved.response_keys;
        if field.key.response_key == field_shape.expected_key {
            format!(
                "Error decoding response from upstream: Missing required field named '{}'",
                &response_keys[field_shape.expected_key]
            )
        } else {
            format!(
                "Error decoding response from upstream: Missing required field named '{}' (expected: '{}')",
                field.response_key_str(),
                &response_keys[field_shape.expected_key]
            )
        }
    }

    pub(super) fn push_edge(&self, edge: impl Into<ResponseEdge>) {
        self.path.borrow_mut().push(edge.into());
    }

    pub(super) fn pop_edge(&self) {
        self.path.borrow_mut().pop();
    }

    pub(super) fn response_path(&self) -> ResponsePath {
        ResponsePath::from(self.path.borrow().clone())
    }

    pub(super) fn should_create_new_graphql_error(&self) -> bool {
        let is_propagating = self.propagating_error.get();
        self.propagating_error.set(true);
        !is_propagating
    }

    pub(super) fn stop_propagating_and_should_create_new_graphql_error(&self) -> bool {
        let is_propagating = self.propagating_error.get();
        self.propagating_error.set(false);
        !is_propagating
    }

    pub(super) fn propagate_error<V, E: serde::de::Error>(&self) -> Result<V, E> {
        self.propagating_error.set(true);
        Err(serde::de::Error::custom(""))
    }
}
