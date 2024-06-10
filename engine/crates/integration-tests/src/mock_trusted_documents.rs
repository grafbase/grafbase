use runtime::trusted_documents_client::{TrustedDocumentsError, TrustedDocumentsResult};

#[derive(Debug, Clone)]
pub struct TestTrustedDocument {
    pub branch_id: &'static str,
    pub client_name: &'static str,
    pub document_id: &'static str,
    pub document_text: &'static str,
}

#[derive(Default)]
pub(super) struct MockTrustedDocumentsClient {
    pub(crate) documents: Vec<TestTrustedDocument>,
    pub(crate) _branch_id: String,
}

#[async_trait::async_trait]
impl runtime::trusted_documents_client::TrustedDocumentsClient for MockTrustedDocumentsClient {
    fn is_enabled(&self) -> bool {
        !self.documents.is_empty()
    }

    fn bypass_header(&self) -> Option<(&str, &str)> {
        Some(("test-bypass-header", "test-bypass-value"))
    }

    async fn fetch(&self, client_name: &str, document_id: &str) -> TrustedDocumentsResult<String> {
        self.documents
            .iter()
            .find(|doc| doc.client_name == client_name && doc.document_id == document_id)
            .map(|doc| Ok(doc.document_text.to_owned()))
            .unwrap_or(Err(TrustedDocumentsError::DocumentNotFound))
    }
}
