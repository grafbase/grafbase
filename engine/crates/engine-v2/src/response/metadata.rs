pub use engine_parser::types::OperationType;
use schema::CacheConfig;

use crate::request::Operation;

/// Metadata we provide to the caller on the operation and its execution.
/// It's serialized when cached. Ignore anything that isn't relevant for a cached response.
#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutionMetadata {
    pub operation_name: Option<String>,
    pub operation_type: Option<OperationType>,
    #[serde(skip, default)]
    pub cache_config: Option<CacheConfig>,
}

impl ExecutionMetadata {
    pub(crate) fn build(operation: &Operation) -> Self {
        Self {
            operation_name: operation.name.clone(),
            operation_type: Some(operation.ty),
            cache_config: operation.cache_config,
        }
    }
}
