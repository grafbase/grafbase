#[derive(Clone, Debug, serde::Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct ModelControlProtocolConfig {
    /// Whether the MCP service is enabled.
    pub enabled: bool,
    /// The service path in the gateway.
    pub path: String,
    /// Whether mutations are enabled for the MCP service.
    pub execute_mutations: bool,
    /// The transport to use (defaults to streaming-http).
    pub transport: McpTransport,
}

#[derive(serde::Deserialize, Debug, Clone, Copy)]
pub enum McpTransport {
    #[serde(rename = "streaming-http")]
    StreamingHttp,
    #[serde(rename = "sse")]
    Sse,
}

impl Default for ModelControlProtocolConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            path: "/mcp".to_string(),
            execute_mutations: false,
            transport: McpTransport::StreamingHttp,
        }
    }
}
