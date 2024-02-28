const DEFAULT_GRAPHQL_PATH: &str = "/graphql";

#[derive(Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphConfig {
    path: Option<String>,
    introspection: Option<bool>,
}

impl GraphConfig {
    /// Defines the path in URL where the graph is available
    pub fn path(&self) -> &str {
        self.path.as_deref().unwrap_or(DEFAULT_GRAPHQL_PATH)
    }

    /// If enabled, graph responds to introspection queries
    pub fn enable_introspection(&self) -> bool {
        self.introspection.unwrap_or_default()
    }
}
