use std::cell::{Cell, RefCell};

use crate::{
    execution::ExecutableOperation,
    operation::LogicalPlanId,
    response::{FieldShape, ResponseEdge, ResponsePath, ResponseWriter},
};
use schema::Schema;

pub(super) struct SeedContext<'ctx> {
    pub schema: &'ctx Schema,
    pub operation: &'ctx ExecutableOperation,
    pub logical_plan_id: LogicalPlanId,
    pub writer: ResponseWriter<'ctx>,
    pub propagating_error: Cell<bool>,
    pub path: RefCell<Vec<ResponseEdge>>,
}

impl<'ctx> SeedContext<'ctx> {
    pub(super) fn missing_field_error_message(&self, shape: &FieldShape) -> String {
        let field = self.operation.walker_with(self.schema.walker()).walk(shape.id);
        let response_keys = &self.operation.response_keys;
        if field.response_key() == shape.expected_key.into() {
            format!(
                "Error decoding response from upstream: Missing required field named '{}'",
                &response_keys[shape.expected_key]
            )
        } else {
            format!(
                "Error decoding response from upstream: Missing required field named '{}' (expected: '{}')",
                &response_keys[field.response_key()],
                &response_keys[shape.expected_key]
            )
        }
    }

    pub(super) fn push_edge(&self, edge: ResponseEdge) {
        self.path.borrow_mut().push(edge);
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
