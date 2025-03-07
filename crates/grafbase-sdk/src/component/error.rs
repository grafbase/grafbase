use crate::wit;

/// Internal SDK error.
#[derive(Debug)]
pub struct SdkError(SdkErrorInner);

impl std::fmt::Display for SdkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for SdkError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum SdkErrorInner {
    #[error("{0}")]
    Message(String),
    #[error("Serialization failed with: {0}")]
    EncodeError(#[from] minicbor_serde::error::EncodeError<std::convert::Infallible>),
    #[error("Deserialization failed with: {0}")]
    DecodeError(#[from] minicbor_serde::error::DecodeError),
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

impl<T> From<T> for SdkError
where
    SdkErrorInner: From<T>,
{
    fn from(err: T) -> Self {
        Self(err.into())
    }
}

impl From<String> for SdkErrorInner {
    fn from(err: String) -> Self {
        Self::Message(err)
    }
}

impl From<SdkError> for wit::Error {
    fn from(err: SdkError) -> Self {
        wit::Error::new(err.to_string())
    }
}
