#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AutomaticPersistedQueries {
    /// If automatic persisted queries should be enabled.
    pub enabled: bool,
}

impl Default for AutomaticPersistedQueries {
    fn default() -> Self {
        Self { enabled: true }
    }
}
