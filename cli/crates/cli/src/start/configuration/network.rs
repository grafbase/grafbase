use std::net::SocketAddr;

#[derive(Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NetworkConfig {
    listen_address: Option<SocketAddr>,
}

impl NetworkConfig {
    /// The address the HTTP server is receiving requests
    pub fn listen_address(&self) -> Option<SocketAddr> {
        self.listen_address
    }
}
