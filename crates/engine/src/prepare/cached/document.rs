use std::{borrow::Cow, sync::Arc};

use crate::engine::cache::DocumentKey;

use super::CachedOperation;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct OperationDocument<'a> {
    pub(crate) key: DocumentKey<'a>,
    pub(crate) content: Cow<'a, str>,
}

impl OperationDocument<'_> {
    pub fn into_owned(self) -> OperationDocument<'static> {
        OperationDocument {
            key: self.key.into_owned(),
            content: Cow::Owned(self.content.into_owned()),
        }
    }
    pub fn operation_name(&self) -> Option<&str> {
        match &self.key {
            DocumentKey::AutomaticPersistedQuery { operation_name, .. }
            | DocumentKey::TrustedDocumentId { operation_name, .. }
            | DocumentKey::Text { operation_name, .. } => operation_name.as_deref(),
        }
    }
}

impl From<CachedOperation> for OperationDocument<'_> {
    fn from(cached: CachedOperation) -> Self {
        cached.document
    }
}

impl From<Arc<CachedOperation>> for OperationDocument<'_> {
    fn from(cached: Arc<CachedOperation>) -> Self {
        cached.document.clone()
    }
}
