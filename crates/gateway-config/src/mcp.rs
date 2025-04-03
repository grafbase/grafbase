#[derive(Clone, Debug, serde::Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct ModelControlProtocolConfig {
    /// Whether the MCP service is enabled.
    pub enabled: bool,
    /// The name of the service.
    pub name: String,
    /// The instructions for the LLM how to use the service.
    pub instructions: Option<String>,
    /// The service path in the gateway.
    pub path: String,
    /// Whether mutations are enabled for the MCP service.
    pub enable_mutations: bool,
}

impl Default for ModelControlProtocolConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            name: "Grafbase GraphQL Model Control Protocol Server".to_string(),
            instructions: None,
            path: "/mcp".to_string(),
            enable_mutations: false,
        }
    }
}
