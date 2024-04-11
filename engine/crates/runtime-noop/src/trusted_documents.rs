use runtime::trusted_documents_client::TrustedDocumentsResult;

pub struct NoopTrustedDocuments;

#[async_trait::async_trait]
impl runtime::trusted_documents_client::TrustedDocumentsClient for NoopTrustedDocuments {
    fn is_enabled(&self) -> bool {
        false
    }

    async fn fetch(&self, _client_name: &str, _document_id: &str) -> TrustedDocumentsResult<String> {
        Err(runtime::trusted_documents_client::TrustedDocumentsError::DocumentNotFound)
    }
}
