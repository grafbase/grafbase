#[derive(Debug)]
pub enum TrustedDocumentsError {
    RetrievalError(Box<dyn std::error::Error + Send + Sync + 'static>),
    DocumentNotFound,
}

pub type TrustedDocumentsResult<T> = Result<T, TrustedDocumentsError>;

/// A handle to trusted documents configuration and retrieval.
pub struct TrustedDocumentsClient {
    inner: Box<dyn TrustedDocumentsClientImpl>,
    branch_id: String,
}

impl TrustedDocumentsClient {
    pub fn new(inner: Box<dyn TrustedDocumentsClientImpl>, branch_id: String) -> Self {
        TrustedDocumentsClient { inner, branch_id }
    }

    pub fn is_enabled(&self) -> bool {
        self.inner.is_enabled()
    }

    pub async fn fetch(&self, client_name: &str, document_id: &str) -> TrustedDocumentsResult<String> {
        self.inner.get(&self.branch_id, client_name, document_id).await
    }
}

#[async_trait::async_trait]
pub trait TrustedDocumentsClientImpl: Send + Sync {
    fn is_enabled(&self) -> bool;
    async fn get(&self, branch_id: &str, client_name: &str, document_id: &str) -> TrustedDocumentsResult<String>;
}
