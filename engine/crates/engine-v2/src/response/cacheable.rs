use crate::{ExecutionMetadata, Response};
use engine_parser::types::OperationType;
use runtime::cache::{CacheMetadata, Cacheable};
use serde_with::base64::Base64;

#[serde_with::serde_as]
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct CacheableResponse {
    // Currently, we store the response as JSON in the cache. So converting to base64 for more
    // efficient storage as JSON represents it as an array of numbers otherwise.
    #[serde_as(as = "Base64")]
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
        Ok(CacheableResponse {
            has_errors: self.has_errors(),
            bytes,
            metadata: self.take_metadata(),
        })
    }
}

impl Cacheable for CacheableResponse {
    fn metadata(&self) -> CacheMetadata {
        let max_age = self
            .metadata
            .cache_config
            .as_ref()
            .map(|config| config.max_age)
            .unwrap_or_default();
        CacheMetadata {
            max_age,
            stale_while_revalidate: self
                .metadata
                .cache_config
                .as_ref()
                .map(|config| config.stale_while_revalidate)
                .unwrap_or_default(),
            tags: vec![],
            should_purge_related: false,
            should_cache: !self.has_errors
                && self
                    .metadata
                    .operation_type
                    .map(|operation_type| operation_type == OperationType::Query)
                    .unwrap_or_default()
                && !max_age.is_zero(),
        }
    }
}
