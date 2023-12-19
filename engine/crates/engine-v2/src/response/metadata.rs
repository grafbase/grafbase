pub use engine_parser::types::OperationType;
use schema::CacheConfig;

use crate::request::Operation;

/// Metadata we provide to the caller on the operation and its execution.
#[derive(Default, Debug, Clone)]
pub struct ExecutionMetadata {
    pub operation_type: Option<OperationType>,
    pub cache_config: Option<CacheConfig>,
}

impl ExecutionMetadata {
    pub fn build(operation: &Operation) -> Self {
        Self {
            operation_type: Some(operation.ty),
            cache_config: operation.cache_config,
        }
    }
}
