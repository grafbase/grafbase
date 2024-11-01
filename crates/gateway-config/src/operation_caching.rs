#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct OperationCaching {
    /// If operation caching should be enabled.
    pub enabled: bool,
    /// The maximum number of operations that can be kept in the cache.
    /// 1000 by default.
    pub limit: usize,
}

impl Default for OperationCaching {
    fn default() -> Self {
        Self {
            enabled: true,
            limit: 1000,
        }
    }
}
