use runtime::trusted_documents::{TrustedDocumentsError, TrustedDocumentsResult};

#[derive(Debug, Clone)]
pub struct TestTrustedDocument {
    pub branch_id: &'static str,
    pub client_name: &'static str,
    pub document_id: &'static str,
    pub document_text: &'static str,
}

pub(super) struct MockTrustedDocuments {
    pub(super) documents: Vec<TestTrustedDocument>,
    pub(super) branch_id: String,
}

impl From<MockTrustedDocuments> for runtime::trusted_documents::TrustedDocuments {
    fn from(value: MockTrustedDocuments) -> Self {
        let branch_name = value.branch_id.clone();
        runtime::trusted_documents::TrustedDocuments::new(Box::new(value), branch_name)
    }
}

#[async_trait::async_trait]
impl runtime::trusted_documents::TrustedDocumentsImpl for MockTrustedDocuments {
    fn trusted_documents_enabled(&self) -> bool {
        !self.documents.is_empty()
    }

    async fn get(&self, branch_id: &str, client_name: &str, document_id: &str) -> TrustedDocumentsResult<String> {
        self.documents
            .iter()
            .find(|doc| doc.branch_id == branch_id && doc.client_name == client_name && doc.document_id == document_id)
            .map(|doc| Ok(doc.document_text.to_owned()))
            .unwrap_or(Err(TrustedDocumentsError::DocumentNotFound))
    }
}
