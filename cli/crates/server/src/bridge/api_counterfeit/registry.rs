#![allow(clippy::module_name_repetitions)]
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct VersionedRegistry {
    pub registry: Registry,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registry {
    #[serde(default)]
    pub search_config: super::search::runtime::Config,
    #[serde(default)]
    pub enable_kv: bool,
}
