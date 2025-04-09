#[derive(Clone, Debug, serde::Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct ModelControlProtocolConfig {
    /// Whether the MCP service is enabled.
    pub enabled: bool,
    /// The service path in the gateway.
    pub path: String,
    /// Whether mutations are enabled for the MCP service.
    pub expose_mutations: bool,
}

impl Default for ModelControlProtocolConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            path: "/mcp".to_string(),
            expose_mutations: false,
        }
    }
}
