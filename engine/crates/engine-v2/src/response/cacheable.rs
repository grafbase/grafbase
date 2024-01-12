use crate::{ExecutionMetadata, Response};
use engine_parser::types::OperationType;
use runtime::cache::{CacheMetadata, Cacheable};

#[serde_with::serde_as]
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct CacheableResponse {
    pub bytes: Vec<u8>,
    pub metadata: ExecutionMetadata,
    pub has_errors: bool,
}

impl crate::Response {
    pub fn into_cacheable<F, E>(self, serializer: F) -> Result<CacheableResponse, E>
    where
        F: FnOnce(&Response) -> Result<Vec<u8>, E>,
    {
        let bytes = serializer(&self)?;
        let has_errors = !self.errors().is_empty();
        Ok(CacheableResponse {
            has_errors,
            bytes,
            metadata: self.take_metadata(),
        })
    }
}

impl Cacheable for CacheableResponse {
    fn metadata(&self) -> CacheMetadata {
        CacheMetadata {
            max_age: self
                .metadata
                .cache_config
                .map(|config| config.max_age)
                .unwrap_or_default(),
            stale_while_revalidate: self
                .metadata
                .cache_config
                .map(|config| config.stale_while_revalidate)
                .unwrap_or_default(),
            tags: vec![],
            should_purge_related: false,
            should_cache: !self.has_errors
                && self
                    .metadata
                    .operation_type
                    .map(|operation_type| operation_type == OperationType::Query)
                    .unwrap_or_default(),
        }
    }
}
