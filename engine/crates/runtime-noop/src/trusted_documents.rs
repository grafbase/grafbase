use runtime::trusted_documents::TrustedDocumentsResult;

struct NoopTrustedDocuments;

#[async_trait::async_trait]
impl runtime::trusted_documents::TrustedDocuments for NoopTrustedDocuments {
    async fn get(&self, _branch_id: &str, _client_name: &str, _document_id: &str) -> TrustedDocumentsResult<String> {
        Err(runtime::trusted_documents::TrustedDocumentsError::DocumentNotFound)
    }
}
