#[derive(Clone, Debug, serde::Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct MCPConfig {
    /// Whether the MCP service is enabled.
    pub enabled: bool,
    /// The service path in the gateway.
    pub path: String,
    /// Whether mutations are enabled for the MCP service.
    #[serde(rename = "execute_mutations")]
    pub can_mutate: bool,
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

impl Default for MCPConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            path: "/mcp".to_string(),
            can_mutate: false,
            transport: McpTransport::StreamingHttp,
        }
    }
}
