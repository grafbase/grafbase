pub use engine_parser::types::OperationType;

use crate::request::Operation;

/// Metadata we provide to the caller on the operation and its execution.
#[derive(Default)]
pub struct ExecutionMetadata {
    pub operation_type: Option<OperationType>,
}

impl ExecutionMetadata {
    pub fn build(operation: &Operation) -> Self {
        Self {
            operation_type: Some(operation.ty),
        }
    }
}
