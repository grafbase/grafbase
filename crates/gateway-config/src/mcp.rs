use serde_dynamic_string::DynamicString;

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelControlProtocolConfig {
    /// Whether the MCP service is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// The name of the service.
    pub name: String,
    /// The instructions for the LLM how to use the service.
    pub instructions: Option<String>,
    /// The service path in the gateway.
    #[serde(default = "default_mcp_path")]
    pub path: String,
    /// Whether mutations are enabled for the MCP service.
    #[serde(default)]
    pub enable_mutations: bool,
    /// Pass the downstream auth header here until the MCP service
    /// supports passing headers.
    #[serde(default)]
    pub authentication: Option<DynamicString<String>>,
}

fn default_mcp_path() -> String {
    "/mcp".to_string()
}
