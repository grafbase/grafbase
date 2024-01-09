use crate::ExecutionMetadata;
use engine_parser::types::OperationType;
use runtime::cache::Cacheable;
use std::time::Duration;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CacheableResponse {
    pub json_bytes: bytes::Bytes,
    // Empty if coming from the cache, nothing was executed.
    #[serde(skip, default)]
    pub metadata: ExecutionMetadata,
    pub has_errors: bool,
}

impl crate::Response {
    pub fn into_cacheable(self) -> Result<CacheableResponse, serde_json::Error> {
        let bytes = serde_json::to_vec(&self)?;
        let has_errors = !self.errors().is_empty();
        Ok(CacheableResponse {
            has_errors,
            json_bytes: bytes::Bytes::from(bytes),
            metadata: self.take_metadata(),
        })
    }
}

impl Cacheable for CacheableResponse {
    fn max_age(&self) -> Duration {
        self.metadata
            .cache_config
            .map(|config| config.max_age)
            .unwrap_or_default()
    }

    fn stale_while_revalidate(&self) -> Duration {
        self.metadata
            .cache_config
            .map(|config| config.stale_while_revalidate)
            .unwrap_or_default()
    }

    fn cache_tags(&self) -> Vec<String> {
        vec![] // to be added when mutation invalidation is supported in v2
    }

    fn should_purge_related(&self) -> bool {
        false // to be added when mutation invalidation is supported in v2
    }

    fn should_cache(&self) -> bool {
        !self.has_errors
            && self
                .metadata
                .operation_type
                .map(|operation_type| {
                    operation_type == OperationType::Query
                })
                .unwrap_or_default()
    }
}
