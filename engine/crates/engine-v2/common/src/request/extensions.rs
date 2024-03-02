use serde::{Deserialize, Serialize};

pub trait HasPersistedQueryExtension {
    fn persisted_query(&self) -> Option<&PersistedQueryRequestExtension>;
}

#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistedQueryRequestExtension {
    pub version: u32,
    #[serde_as(as = "serde_with::hex::Hex")]
    pub sha256_hash: Vec<u8>,
}
