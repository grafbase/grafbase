pub enum TrustedDocumentsError {
    RetrievalError(Box<dyn std::error::Error + Send + Sync + 'static>),
    DocumentNotFound,
}

pub type TrustedDocumentsResult<T> = Result<T, TrustedDocumentsError>;

pub struct TrustedDocuments(pub Box<dyn TrustedDocumentsImpl>);

#[async_trait::async_trait]
pub trait TrustedDocumentsImpl: Send + Sync {
    fn trusted_documents_enabled(&self) -> bool;
    async fn get(&self, branch_id: &str, client_name: &str, document_id: &str) -> TrustedDocumentsResult<String>;
}
