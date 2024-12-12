#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct OperationCaching {
    /// If operation caching should be enabled.
    pub enabled: bool,
    /// The maximum number of operations that can be kept in the cache.
    /// 1000 by default.
    pub limit: usize,
    /// Whether the cache should be warmed before schema/config reload
    pub warm_on_reload: bool,
}

impl Default for OperationCaching {
    fn default() -> Self {
        Self {
            enabled: true,
            limit: 1000,
            warm_on_reload: false,
        }
    }
}
