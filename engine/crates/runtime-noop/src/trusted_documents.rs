use runtime::trusted_documents::TrustedDocumentsResult;

pub struct NoopTrustedDocuments;

impl From<NoopTrustedDocuments> for runtime::trusted_documents::TrustedDocuments {
    fn from(_: NoopTrustedDocuments) -> Self {
        runtime::trusted_documents::TrustedDocuments::new(Box::new(NoopTrustedDocuments), String::from("irrelevant"))
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
