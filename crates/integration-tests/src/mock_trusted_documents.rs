use runtime::trusted_documents_client::{
    TrustedDocumentsEnforcementMode, TrustedDocumentsError, TrustedDocumentsResult,
};

#[derive(Debug, Clone)]
pub struct TestTrustedDocument {
    pub branch_id: &'static str,
    pub client_name: &'static str,
    pub document_id: &'static str,
    pub document_text: &'static str,
}

pub(super) struct MockTrustedDocumentsClient {
    pub(crate) documents: Vec<TestTrustedDocument>,
    pub(crate) enforcement_mode: TrustedDocumentsEnforcementMode,
}

impl Default for MockTrustedDocumentsClient {
    fn default() -> Self {
        MockTrustedDocumentsClient {
            documents: Vec::new(),
            enforcement_mode: TrustedDocumentsEnforcementMode::Ignore,
        }
    }
}

#[async_trait::async_trait]
impl runtime::trusted_documents_client::TrustedDocumentsClient for MockTrustedDocumentsClient {
    fn enforcement_mode(&self) -> TrustedDocumentsEnforcementMode {
        self.enforcement_mode
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
