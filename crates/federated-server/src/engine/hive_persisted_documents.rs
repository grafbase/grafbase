use hive_console_sdk::persisted_documents::{PersistedDocumentsError, PersistedDocumentsManager};
use runtime::trusted_documents_client::{
    TrustedDocumentsClient, TrustedDocumentsEnforcementMode, TrustedDocumentsError, TrustedDocumentsResult,
};

pub struct HivePersistedDocuments {
    manager: PersistedDocumentsManager,
    enforcement_mode: TrustedDocumentsEnforcementMode,
}

impl HivePersistedDocuments {
    pub fn new(manager: PersistedDocumentsManager, enforcement_mode: TrustedDocumentsEnforcementMode) -> Self {
        Self {
            manager,
            enforcement_mode,
        }
    }
}

#[async_trait::async_trait]
impl TrustedDocumentsClient for HivePersistedDocuments {
    fn enforcement_mode(&self) -> TrustedDocumentsEnforcementMode {
        self.enforcement_mode
    }

    async fn fetch(&self, _client_name: &str, document_id: &str) -> TrustedDocumentsResult<String> {
        self.manager
            .resolve_document(document_id)
            .await
            .map_err(|err| match err {
                PersistedDocumentsError::DocumentNotFound => TrustedDocumentsError::DocumentNotFound,
                err => TrustedDocumentsError::RetrievalError(err.into()),
            })
    }
}
