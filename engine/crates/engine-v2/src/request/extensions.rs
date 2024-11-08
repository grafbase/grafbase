use std::collections::HashMap;

use engine_value::Value;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestExtensions {
    #[serde(default)]
    pub persisted_query: Option<PersistedQueryRequestExtension>,
    #[serde(flatten)]
    pub custom: HashMap<String, Value>,
}

#[serde_with::serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistedQueryRequestExtension {
    pub version: u32,
    #[serde_as(as = "serde_with::hex::Hex")]
    pub sha256_hash: Vec<u8>,
}
