#[derive(Debug)]
pub enum TrustedDocumentsError {
    RetrievalError(Box<dyn std::error::Error + Send + Sync + 'static>),
    DocumentNotFound,
}

pub type TrustedDocumentsResult<T> = Result<T, TrustedDocumentsError>;

/// A handle to trusted documents configuration and retrieval.
#[async_trait::async_trait]
pub trait TrustedDocumentsClient: Send + Sync {
    fn is_enabled(&self) -> bool;

    fn bypass_header(&self) -> Option<(&str, &str)> {
        None
    }

    async fn fetch(&self, client_name: &str, document_id: &str) -> TrustedDocumentsResult<String>;
}
