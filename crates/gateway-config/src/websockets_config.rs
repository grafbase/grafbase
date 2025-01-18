#[derive(Debug, serde::Deserialize, Clone)]
pub struct WebsocketsConfig {
    #[serde(default)]
    pub forward_connection_init_payload: bool,
}

impl Default for WebsocketsConfig {
    fn default() -> Self {
        WebsocketsConfig {
            forward_connection_init_payload: true,
        }
    }
}
