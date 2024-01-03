use runtime::cache::Cacheable;

use crate::ExecutionMetadata;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CacheableResponse {
    // transform to base64 for json or just serialize with msgpack
    pub json_body: bytes::Bytes,
    // Empty if coming from the cache, nothing was executed.
    #[serde(skip, default)]
    pub metadata: ExecutionMetadata,
}

/// Cloned for caching, where we ignore metadata anyway. Not super great.
impl Clone for CacheableResponse {
    fn clone(&self) -> Self {
        Self {
            json_body: self.json_body.clone(),
            metadata: ExecutionMetadata::default(),
        }
    }
}

impl From<crate::Response> for CacheableResponse {
    fn from(value: crate::Response) -> Self {
        match serde_json::to_vec(&value) {
            Ok(bytes) => Self {
                json_body: bytes.into(),
                metadata: value.into_metadata(),
            },
            Err(_) => Self {
                json_body: serde_json::to_vec(&serde_json::json!({"errors": [
                    {"message": "Serialization failure"}
                ]}))
                .unwrap()
                .into(),
                metadata: ExecutionMetadata::default(),
            },
        }
    }
}
