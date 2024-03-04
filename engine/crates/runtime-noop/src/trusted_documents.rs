use runtime::trusted_documents::TrustedDocumentsResult;

pub struct NoopTrustedDocuments;

impl NoopTrustedDocuments {
    pub fn runtime() -> runtime::trusted_documents::TrustedDocuments {
        runtime::trusted_documents::TrustedDocuments(Box::new(Self))
    }
}

#[async_trait::async_trait]
impl runtime::trusted_documents::TrustedDocumentsImpl for NoopTrustedDocuments {
    fn trusted_documents_enabled(&self) -> bool {
        false
    }

    async fn get(&self, _branch_id: &str, _client_name: &str, _document_id: &str) -> TrustedDocumentsResult<String> {
        Err(runtime::trusted_documents::TrustedDocumentsError::DocumentNotFound)
    }
}
