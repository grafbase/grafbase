#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("{0}")]
    CacheGet(String),
    #[error("{0}")]
    CachePut(String),
    #[error("{0}")]
    CacheDelete(String),
    #[error("{0}")]
    CachePurgeByTags(String),
    #[error("Origin error: {0}")]
    Origin(#[from] worker::Error),
}

impl From<CacheError> for worker::Error {
    fn from(value: CacheError) -> Self {
        worker::Error::RustError(value.to_string())
    }
}
