use runtime::trusted_documents_service::TrustedDocumentsResult;

pub struct NoopTrustedDocuments;

impl From<NoopTrustedDocuments> for runtime::trusted_documents_service::TrustedDocumentsClient {
    fn from(_: NoopTrustedDocuments) -> Self {
        runtime::trusted_documents_service::TrustedDocumentsClient::new(
            Box::new(NoopTrustedDocuments),
            String::from("irrelevant"),
        )
    }
}

#[async_trait::async_trait]
impl runtime::trusted_documents_service::TrustedDocumentsClientImpl for NoopTrustedDocuments {
    fn is_enabled(&self) -> bool {
        false
    }

    async fn get(&self, _branch_id: &str, _client_name: &str, _document_id: &str) -> TrustedDocumentsResult<String> {
        Err(runtime::trusted_documents_service::TrustedDocumentsError::DocumentNotFound)
    }
}
