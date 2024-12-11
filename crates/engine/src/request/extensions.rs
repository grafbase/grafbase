use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RequestExtensions {
    #[serde(default)]
    pub persisted_query: Option<PersistedQueryRequestExtension>,
}

#[serde_with::serde_as]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PersistedQueryRequestExtension {
    pub version: u32,
    #[serde_as(as = "serde_with::hex::Hex")]
    pub sha256_hash: Vec<u8>,
}
