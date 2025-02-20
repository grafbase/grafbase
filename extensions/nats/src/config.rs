#[derive(Debug, serde::Deserialize)]
pub struct NatsConfig {
    #[serde(rename = "endpoint")]
    pub endpoints: Vec<Endpoint>,
}

#[derive(Debug, serde::Deserialize)]
pub struct Endpoint {
    pub name: String,
    pub servers: Vec<String>,
    pub authentication: Option<AuthConfig>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
pub enum AuthConfig {
    UsernamePassword { username: String, password: String },
    Token { token: String },
    Credentials { credentials: String },
}
