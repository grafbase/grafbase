use runtime::trusted_documents_client::{TrustedDocumentsEnforcementMode, TrustedDocumentsResult};

pub struct NoopTrustedDocuments;

#[async_trait::async_trait]
impl runtime::trusted_documents_client::TrustedDocumentsClient for NoopTrustedDocuments {
    fn enforcement_mode(&self) -> TrustedDocumentsEnforcementMode {
        TrustedDocumentsEnforcementMode::Ignore
    }

    async fn fetch(&self, _client_name: &str, _document_id: &str) -> TrustedDocumentsResult<String> {
        Err(runtime::trusted_documents_client::TrustedDocumentsError::DocumentNotFound)
    }
}
