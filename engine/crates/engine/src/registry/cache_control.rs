pub use registry_v2::cache_control::*;

#[derive(Debug, thiserror::Error)]
pub enum CacheControlError {
    #[error(transparent)]
    Parse(#[from] crate::parser::Error),
    #[error("Validation Error: {0:?}")]
    Validate(Vec<crate::ServerError>),
}
